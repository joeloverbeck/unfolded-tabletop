use crate::support;
use std::{fs, path::Path};

fn search(path: &Path, args: &[&str]) -> std::process::Output {
    support::tool()
        .arg("search-context")
        .arg(path)
        .args(args)
        .output()
        .unwrap()
}

fn success(output: std::process::Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
    String::from_utf8(output.stdout).unwrap()
}

fn rows(text: &str) -> Vec<(usize, &str)> {
    text.lines()
        .filter_map(|line| line.split_once(" | "))
        .map(|(number, content)| (number.trim().parse().unwrap(), content))
        .collect()
}

fn literal_search_stays_within_the_selected_range_and_preserves_its_source() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("source.txt");
    let source = "a.b outside\naxb is not literal\na.b café\nA.B is different\na.b outside\n";
    fs::write(&path, source).unwrap();
    let text = success(search(
        &path,
        &["--find", "a.b", "--start", "2", "--end", "4"],
    ));
    assert_eq!(rows(&text), [(3, "a.b café")]);
    assert!(text.contains("MATCHES 1"));
    assert!(text.ends_with("\nEND\n"));
    assert_eq!(fs::read_to_string(path).unwrap(), source);
}

fn pages_preserve_scoped_context_once_under_one_complete_byte_budget() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("source.txt");
    let source: String = (1..=20)
        .map(|number| {
            let word = if [3, 4, 10, 19].contains(&number) {
                "hit"
            } else {
                "plain"
            };
            format!("{word} {number}: café 漢字 {}\n", "x".repeat(70))
        })
        .collect();
    fs::write(&path, &source).unwrap();
    let query = [
        "--find",
        "hit",
        "--start",
        "3",
        "--end",
        "19",
        "--context",
        "1",
    ];
    let mut continuation = "--max-bytes 512".to_owned();
    let mut observed = Vec::new();
    for page in 0..20 {
        let args: Vec<_> = query
            .into_iter()
            .chain(continuation.split_whitespace())
            .collect();
        let result = search(&path, &args);
        assert!(result.stdout.len() <= 512);
        let text = success(result);
        assert!(text.contains("MATCHES 4"));
        observed.extend(rows(&text).into_iter().map(|(number, _)| number));
        if let Some(next) = text.lines().find_map(|line| line.strip_prefix("NEXT ")) {
            continuation = next.to_owned();
        } else {
            assert!(page > 0, "fixture must exercise continuation");
            assert!(text.ends_with("\nEND\n"));
            assert_eq!(observed, [3, 4, 5, 9, 10, 11, 18, 19]);
            assert_eq!(fs::read_to_string(path).unwrap(), source);
            return;
        }
    }
    panic!("search did not finish");
}

fn heading_scope_includes_children_and_ignores_headings_inside_fenced_examples() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("sections.md");
    fs::write(&path, "# Archive\n```markdown\n## Selected\nneedle fake\n```\n## Selected\nlead\nneedle one\n### Child\nneedle two\n~~~text\n# False boundary\n~~~\nneedle three\n## Other\nneedle outside\n").unwrap();
    let text = success(search(
        &path,
        &[
            "--find",
            "needle",
            "--heading",
            "## Selected",
            "--context",
            "20",
        ],
    ));
    assert!(text.contains("SELECTED 6..14 of 16 lines"));
    assert!(text.contains("MATCHES 3"));
    assert_eq!(
        rows(&text)
            .iter()
            .map(|(number, _)| *number)
            .collect::<Vec<_>>(),
        [6, 7, 8, 9, 10, 11, 12, 13, 14]
    );
    assert!(!text.contains("needle outside"));
    assert!(!text.contains("needle fake"));
}

fn empty_sources_and_no_matches_are_explicit_successful_results() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("empty.txt");
    for source in ["unrelated\n", ""] {
        fs::write(&path, source).unwrap();
        let text = success(search(&path, &["--find", "needle"]));
        assert!(text.contains("MATCHES 0\nNO MATCHES\n"));
        assert!(rows(&text).is_empty());
        assert!(text.ends_with("\nEND\n"));
        if source.is_empty() {
            assert!(text.contains("EMPTY FILE"));
        }
    }
}

fn continuation_refuses_source_query_scope_and_context_changes_before_output() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("sections.md");
    let source = format!(
        "## Alpha\n{}## Beta\nneedle elsewhere\n",
        "needle common café\n".repeat(40)
    );
    fs::write(&path, &source).unwrap();
    let query = ["--find", "needle", "--heading", "## Alpha"];
    let first = success(search(
        &path,
        &[&query[..], &["--max-bytes", "512"]].concat(),
    ));
    let next: Vec<_> = first
        .lines()
        .find_map(|line| line.strip_prefix("NEXT "))
        .unwrap()
        .split_whitespace()
        .collect();
    for changed in [
        &["--find", "common", "--heading", "## Alpha"][..],
        &["--find", "needle", "--heading", "## Beta"],
        &[
            "--find",
            "needle",
            "--heading",
            "## Alpha",
            "--context",
            "1",
        ],
        &["--find", "needle", "--heading", "## Alpha", "--end", "40"],
        &["--find", "needle", "--heading", "## Alpha", "--start", "2"],
    ] {
        let result = search(&path, &[changed, &next].concat());
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("source or search options changed")
        );
    }
    fs::write(&path, format!("{source}changed outside selected section\n")).unwrap();
    let result = search(&path, &[&query[..], &next].concat());
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("source or search options changed"));
}

fn ambiguous_headings_require_a_range_and_missing_headings_are_not_no_matches() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("sections.md");
    fs::write(
        &path,
        "## Repeated\nneedle first\n## Repeated\nneedle second\n",
    )
    .unwrap();
    for heading in ["## Repeated", "## Missing", "not an ATX heading"] {
        let result = search(&path, &["--find", "needle", "--heading", heading]);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
    }
    let text = success(search(
        &path,
        &[
            "--find",
            "needle",
            "--heading",
            "## Repeated",
            "--start",
            "3",
        ],
    ));
    assert_eq!(rows(&text), [(4, "needle second")]);
}

fn invalid_inputs_and_oversized_matches_emit_no_source() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("source.txt");
    let missing = search(&path, &["--find", "needle"]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(missing.stdout.is_empty());
    for source in [
        b"needle\n\xff".to_vec(),
        format!("needle {}", "x".repeat(13_000)).into_bytes(),
    ] {
        fs::write(&path, source).unwrap();
        let result = search(&path, &["--find", "needle", "--max-bytes", "12000"]);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
    }
    fs::write(&path, "needle\n").unwrap();
    for args in [
        &["--find", ""][..],
        &["--find", "one\ntwo"],
        &["--find", "needle", "--start", "0"],
        &["--find", "needle", "--end", "2"],
        &["--find", "needle", "--max-bytes", "511"],
        &["--find", "needle", "--max-bytes", "12001"],
        &["--find", "needle", "--next-line", "1"],
    ] {
        let result = search(&path, args);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
    }
}

fn crlf_missing_final_newline_and_shell_characters_remain_literal() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("source.txt");
    fs::write(&path, "needle\r\nneedle '$(literal)` value").unwrap();
    let text = success(search(&path, &["--find", "needle"]));
    assert!(text.contains("     1 | needle\r\n"));
    assert!(
        text.ends_with("     2 | needle '$(literal)` value\nEND (source has no final newline)\n")
    );
    let literal = success(search(&path, &["--find", "'$(literal)`"]));
    assert_eq!(rows(&literal), [(2, "needle '$(literal)` value")]);
}

fn multiple_files_and_terms_share_a_budget_and_never_duplicate_overlapping_lines() {
    let directory = tempfile::tempdir().unwrap();
    let paths: Vec<_> = ["one", "two", "empty"]
        .map(|name| directory.path().join(name))
        .into();
    for (index, path) in paths.iter().enumerate() {
        let source = if index == 2 {
            "unrelated\n".into()
        } else {
            (1..=12)
                .map(|n| {
                    format!(
                        "file{index} line{n} {} {}\n",
                        if n % 2 == 0 { "alpha beta" } else { "beta" },
                        "x".repeat(35)
                    )
                })
                .collect::<String>()
        };
        fs::write(path, source).unwrap();
    }
    let mut next = "--max-bytes 512".to_owned();
    let mut observed = Vec::new();
    let mut no_matches = false;
    for _ in 0..50 {
        let result = support::tool()
            .arg("search-context")
            .args(&paths)
            .args(["--find", "alpha", "--find", "beta", "--context", "1"])
            .args(next.split_whitespace())
            .output()
            .unwrap();
        assert!(result.stdout.len() <= 512);
        let text = success(result);
        observed.extend(rows(&text).into_iter().map(|(_, line)| line.to_owned()));
        no_matches |= text.contains("NO MATCHES");
        if let Some(more) = text.lines().find_map(|line| line.strip_prefix("NEXT ")) {
            next = more.into();
        } else {
            assert!(text.ends_with("\nEND\n"));
            assert_eq!(observed.len(), 24);
            assert_eq!(
                observed
                    .iter()
                    .collect::<std::collections::HashSet<_>>()
                    .len(),
                24
            );
            assert!(observed[0].starts_with("file0 line1 "));
            assert!(observed[23].starts_with("file1 line12 "));
            assert!(no_matches);
            return;
        }
    }
    panic!("multi-file search did not finish");
}

fn multi_file_continuation_binds_unread_sources_path_order_and_every_term() {
    let directory = tempfile::tempdir().unwrap();
    let a = directory.path().join("a");
    let b = directory.path().join("b");
    fs::write(&a, "alpha beta\n".repeat(50)).unwrap();
    fs::write(&b, "alpha\n").unwrap();
    let invoke = |paths: &[&Path], terms: &[&str], next: &str| {
        support::tool()
            .arg("search-context")
            .args(paths)
            .args(terms)
            .args(next.split_whitespace())
            .output()
            .unwrap()
    };
    let terms = ["--find", "alpha", "--find", "beta"];
    let first = success(invoke(&[&a, &b], &terms, "--max-bytes 512"));
    let next = first
        .lines()
        .find_map(|line| line.strip_prefix("NEXT "))
        .unwrap();
    for result in [
        invoke(&[&b, &a], &terms, next),
        invoke(&[&a, &b], &["--find", "alpha"], next),
    ] {
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
    }
    fs::write(&b, "changed unread source\n").unwrap();
    let result = invoke(&[&a, &b], &terms, next);
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
}

fn multiple_no_matches_and_oversized_following_file_remain_bounded() {
    let root = tempfile::tempdir().unwrap();
    let a = root.path().join("a");
    let b = root.path().join("b");
    fs::write(&a, "unrelated\n").unwrap();
    fs::write(&b, "").unwrap();
    let invoke = |next: &str| {
        support::tool()
            .arg("search-context")
            .args([&a, &b])
            .args(["--find", "hit"])
            .args(next.split_whitespace())
            .output()
            .unwrap()
    };
    let text = success(invoke("--max-bytes 6000"));
    assert_eq!(text.matches("NO MATCHES").count(), 2);
    fs::write(&b, format!("hit {}", "x".repeat(13000))).unwrap();
    let first = success(invoke("--max-bytes 512"));
    assert!(first.len() <= 512);
    let next = first
        .lines()
        .find_map(|line| line.strip_prefix("NEXT "))
        .unwrap();
    let refused = invoke(next);
    assert_eq!(refused.status.code(), Some(2));
    assert!(refused.stdout.is_empty());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("cannot fit"));
}

pub const CASES: &[crate::Case] = &[
    (
        "search::literal_search_stays_within_the_selected_range_and_preserves_its_source",
        literal_search_stays_within_the_selected_range_and_preserves_its_source,
    ),
    (
        "search::pages_preserve_scoped_context_once_under_one_complete_byte_budget",
        pages_preserve_scoped_context_once_under_one_complete_byte_budget,
    ),
    (
        "search::heading_scope_includes_children_and_ignores_headings_inside_fenced_examples",
        heading_scope_includes_children_and_ignores_headings_inside_fenced_examples,
    ),
    (
        "search::empty_sources_and_no_matches_are_explicit_successful_results",
        empty_sources_and_no_matches_are_explicit_successful_results,
    ),
    (
        "search::continuation_refuses_source_query_scope_and_context_changes_before_output",
        continuation_refuses_source_query_scope_and_context_changes_before_output,
    ),
    (
        "search::ambiguous_headings_require_a_range_and_missing_headings_are_not_no_matches",
        ambiguous_headings_require_a_range_and_missing_headings_are_not_no_matches,
    ),
    (
        "search::invalid_inputs_and_oversized_matches_emit_no_source",
        invalid_inputs_and_oversized_matches_emit_no_source,
    ),
    (
        "search::crlf_missing_final_newline_and_shell_characters_remain_literal",
        crlf_missing_final_newline_and_shell_characters_remain_literal,
    ),
    (
        "search::multiple_files_and_terms_share_a_budget_and_never_duplicate_overlapping_lines",
        multiple_files_and_terms_share_a_budget_and_never_duplicate_overlapping_lines,
    ),
    (
        "search::multi_file_continuation_binds_unread_sources_path_order_and_every_term",
        multi_file_continuation_binds_unread_sources_path_order_and_every_term,
    ),
    (
        "search::multiple_no_matches_and_oversized_following_file_remain_bounded",
        multiple_no_matches_and_oversized_following_file_remain_bounded,
    ),
];
