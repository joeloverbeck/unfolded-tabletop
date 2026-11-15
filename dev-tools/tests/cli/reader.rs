use crate::support;
use std::fs;

fn read(path: &std::path::Path, args: &[&str]) -> std::process::Output {
    support::tool()
        .arg("read-context")
        .arg(path)
        .args(args)
        .output()
        .unwrap()
}

fn line_windows_stop_at_eof_without_weakening_explicit_ranges() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("short.txt");
    fs::write(&path, "first\nsecond\nthird\n").unwrap();
    let output = read(&path, &["--start", "2", "--lines", "100"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("SELECTED 2..3 of 3 lines"));
    assert!(text.contains("     2 | second\n     3 | third\n"));
    assert!(text.ends_with("\nEND\n"));

    for args in [
        vec!["--start", "2", "--end", "100"],
        vec!["--lines", "2", "--end", "3"],
        vec!["--lines", "0"],
        vec!["--start", "4", "--lines", "2"],
    ] {
        let invalid = read(&path, &args);
        assert_eq!(invalid.status.code(), Some(2));
        assert!(invalid.stdout.is_empty());
    }
}

fn help_identifies_the_batch_file_and_both_byte_limits() {
    for command in ["read-context", "read-context-batch", "search-context"] {
        let result = support::tool().args([command, "--help"]).output().unwrap();
        assert!(result.status.success());
        assert!(result.stderr.is_empty());
        let help = String::from_utf8(result.stdout).unwrap();
        assert!(help.contains("--max-bytes <MAX_BYTES>"), "{help}");
        assert!(help.contains("512..=12000"), "{help}");
        if command == "read-context-batch" {
            assert!(help.contains("<REQUESTS_FILE>"), "{help}");
            assert!(help.contains("Path to a JSON file containing"), "{help}");
            assert!(help.contains("{path, start?, end?}"), "{help}");
            assert!(help.contains("{path, start?, lines?}"), "{help}");
        }
        if command == "read-context" {
            assert!(help.contains("--lines <LINES>"), "{help}");
        }
        if command == "search-context" {
            assert!(help.contains("--find <FIND>"), "{help}");
            assert!(help.contains("--heading <HEADING>"), "{help}");
            assert!(
                help.contains("repeat the same path and query/scope options"),
                "{help}"
            );
        }
    }
}

fn continuations_cover_selected_unicode_lines_once_within_the_complete_byte_limit() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("source.txt");
    let lines: Vec<_> = (1..=80)
        .map(|n| format!("  source {n}: café 漢字 — {}\n", "x".repeat(n % 37)))
        .collect();
    let source = lines.concat();
    fs::write(&path, &source).unwrap();
    for selection in ["--start 3 --end 79", "--start 3 --lines 77"] {
        let mut args = format!("{selection} --max-bytes 512");
        let mut observed = Vec::new();
        let mut pages = 0;
        loop {
            let output = read(&path, &args.split_whitespace().collect::<Vec<_>>());
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(output.stdout.len() <= 512);
            let text = String::from_utf8(output.stdout).unwrap();
            for row in text.lines() {
                if let Some((number, content)) = row.split_once(" | ") {
                    observed.push((
                        number.trim().parse::<usize>().unwrap(),
                        format!("{content}\n"),
                    ));
                }
            }
            pages += 1;
            assert!(pages <= 80, "continuation did not advance");
            match text.lines().find_map(|line| line.strip_prefix("NEXT ")) {
                Some(next) => args = next.to_owned(),
                None => {
                    assert!(text.ends_with("\nEND\n"));
                    break;
                }
            }
        }
        assert!(pages > 1);
        assert_eq!(
            observed,
            lines
                .iter()
                .cloned()
                .enumerate()
                .map(|(n, line)| (n + 1, line))
                .skip(2)
                .take(77)
                .collect::<Vec<_>>()
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), source);
    }
}

fn changed_source_refuses_continuation_without_emitting_content() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("source.txt");
    fs::write(&path, "original\n".repeat(100)).unwrap();
    let first =
        String::from_utf8(read(&path, &["--lines", "50", "--max-bytes", "512"]).stdout).unwrap();
    let continuation = first
        .lines()
        .find_map(|line| line.strip_prefix("NEXT "))
        .unwrap();
    fs::write(&path, "changed\n".repeat(100)).unwrap();
    let result = read(&path, &continuation.split_whitespace().collect::<Vec<_>>());
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("source changed"));
}

fn invalid_ranges_limits_files_and_oversized_lines_emit_no_source() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("source.txt");
    let missing = read(&path, &[]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(missing.stdout.is_empty());
    for data in [b"before\n\xff\nafter\n".to_vec(), vec![b'x'; 13_000]] {
        fs::write(&path, data).unwrap();
        let result = read(&path, &["--max-bytes", "12000"]);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
    }
    fs::write(&path, "one\ntwo\n").unwrap();
    for args in [
        &["--start", "0"][..],
        &["--end", "3"],
        &["--start", "2", "--end", "1"],
        &["--max-bytes", "511"],
        &["--max-bytes", "12001"],
    ] {
        let result = read(&path, args);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
    }
}

fn empty_crlf_and_missing_final_newline_are_explicit_and_preserved() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("source.txt");
    fs::write(&path, "").unwrap();
    let empty = read(&path, &[]);
    assert!(empty.status.success());
    assert_eq!(
        String::from_utf8(empty.stdout).unwrap(),
        "SHA256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\nEMPTY FILE\nEND\n"
    );
    fs::write(&path, "first\r\nlast without newline").unwrap();
    let result = read(&path, &[]);
    assert!(result.status.success());
    let output = String::from_utf8(result.stdout).unwrap();
    assert!(output.contains("     1 | first\r\n"));
    assert!(output.ends_with("     2 | last without newline\nEND (source has no final newline)\n"));
}

fn batch(path: &std::path::Path, args: &[&str]) -> std::process::Output {
    support::tool()
        .arg("read-context-batch")
        .arg(path)
        .args(args)
        .output()
        .unwrap()
}

fn missing_batch_manifest_reports_the_requested_file_without_source_output() {
    let directory = tempfile::tempdir().unwrap();
    let requests = directory.path().join("missing requests.json");
    let result = batch(&requests, &[]);
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
    let diagnostic = String::from_utf8(result.stderr).unwrap();
    assert!(
        diagnostic.contains("cannot read requests file"),
        "{diagnostic}"
    );
    assert!(
        diagnostic.contains(requests.to_str().unwrap()),
        "{diagnostic}"
    );
}

fn batch_continuations_share_one_budget_and_deliver_each_selected_line_once() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first file.txt");
    let second = directory.path().join("second.txt");
    fs::write(&first, "first café\n".repeat(45)).unwrap();
    fs::write(&second, "second 漢字\n".repeat(30)).unwrap();
    let requests = directory.path().join("requests.json");
    fs::write(
        &requests,
        serde_json::to_vec(&serde_json::json!([
            {"path":first,"start":3,"end":40},
            {"path":second,"start":2,"lines":100}
        ]))
        .unwrap(),
    )
    .unwrap();
    let mut args = "--max-bytes 600".to_owned();
    let mut observed = Vec::new();
    for page in 0..100 {
        let output = batch(&requests, &args.split_whitespace().collect::<Vec<_>>());
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            output.stdout.len() <= 600,
            "combined output exceeded its budget"
        );
        let text = String::from_utf8(output.stdout).unwrap();
        for row in text.lines() {
            if let Some((number, content)) = row.split_once(" | ") {
                observed.push((number.trim().parse::<usize>().unwrap(), content.to_owned()));
            }
        }
        if let Some(next) = text.lines().find_map(|line| line.strip_prefix("NEXT ")) {
            args = next.to_owned();
        } else {
            assert!(text.ends_with("\nEND\n"));
            assert!(page > 1, "the fixture must exercise continuation");
            let expected: Vec<_> = (3..=40)
                .map(|n| (n, "first café".to_owned()))
                .chain((2..=30).map(|n| (n, "second 漢字".to_owned())))
                .collect();
            assert_eq!(observed, expected);
            return;
        }
    }
    panic!("batch continuation did not finish");
}

fn batch_continuation_binds_even_unread_sources_and_the_request_list() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first.txt");
    let second = directory.path().join("second.txt");
    let requests = directory.path().join("requests.json");
    fs::write(&first, "first\n".repeat(100)).unwrap();
    fs::write(&second, "second\n").unwrap();
    let manifest = serde_json::to_vec(
        &serde_json::json!([{ "path": first, "lines":80 }, { "path": second, "lines":100 }]),
    )
    .unwrap();
    fs::write(&requests, &manifest).unwrap();
    let first_page = batch(&requests, &["--max-bytes", "512"]);
    assert!(first_page.status.success());
    let text = String::from_utf8(first_page.stdout).unwrap();
    let next = text
        .lines()
        .find_map(|line| line.strip_prefix("NEXT "))
        .unwrap();
    for change_manifest in [false, true] {
        fs::write(
            &second,
            if change_manifest {
                "second\n"
            } else {
                "changed\n"
            },
        )
        .unwrap();
        if change_manifest {
            fs::write(
                &requests,
                serde_json::to_vec(&serde_json::json!([
                    { "path": first, "lines":50 }, { "path": second, "lines":100 }
                ]))
                .unwrap(),
            )
            .unwrap();
        }
        let result = batch(&requests, &next.split_whitespace().collect::<Vec<_>>());
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("changed"));
    }
}

fn batch_validates_all_excerpts_before_output_and_preserves_empty_and_crlf_sources() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("empty.txt");
    let second = directory.path().join("second.txt");
    let requests = directory.path().join("requests.json");
    fs::write(&first, "").unwrap();
    let manifest = serde_json::to_vec(
        &serde_json::json!([{ "path": first, "lines":100 }, { "path": second }]),
    )
    .unwrap();
    fs::write(&requests, &manifest).unwrap();
    let missing = batch(&requests, &[]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(missing.stdout.is_empty());
    fs::write(&second, "first\r\nlast without newline").unwrap();
    let result = batch(&requests, &[]);
    assert!(result.status.success());
    let text = String::from_utf8(result.stdout).unwrap();
    assert!(text.contains("EMPTY FILE"));
    assert!(text.contains("     1 | first\r\n"));
    assert!(
        text.contains("     2 | last without newline\nEND REQUEST (source has no final newline)")
    );
    for bad_manifest in [
        serde_json::json!([]),
        serde_json::json!([{ "path":second, "start":0 }]),
        serde_json::json!([{ "path":second, "end":3 }]),
        serde_json::json!([{ "path":second, "lines":0 }]),
        serde_json::json!([{ "path":second, "lines":1, "end":2 }]),
        serde_json::json!([{ "path":second, "start":3, "lines":1 }]),
        serde_json::json!([{ "path":second, "strat":2 }]),
    ] {
        fs::write(&requests, serde_json::to_vec(&bad_manifest).unwrap()).unwrap();
        let result = batch(&requests, &[]);
        assert_eq!(result.status.code(), Some(2));
        assert!(result.stdout.is_empty());
    }
    fs::write(
        &requests,
        serde_json::to_vec(&serde_json::json!([{ "path":second }])).unwrap(),
    )
    .unwrap();
    fs::write(&second, "x".repeat(13000)).unwrap();
    let result = batch(&requests, &[]);
    assert_eq!(result.status.code(), Some(2));
    assert!(result.stdout.is_empty());
}

pub const CASES: &[crate::Case] = &[
    (
        "reader::line_windows_stop_at_eof_without_weakening_explicit_ranges",
        line_windows_stop_at_eof_without_weakening_explicit_ranges,
    ),
    (
        "reader::help_identifies_the_batch_file_and_both_byte_limits",
        help_identifies_the_batch_file_and_both_byte_limits,
    ),
    (
        "reader::continuations_cover_selected_unicode_lines_once_within_the_complete_byte_limit",
        continuations_cover_selected_unicode_lines_once_within_the_complete_byte_limit,
    ),
    (
        "reader::changed_source_refuses_continuation_without_emitting_content",
        changed_source_refuses_continuation_without_emitting_content,
    ),
    (
        "reader::invalid_ranges_limits_files_and_oversized_lines_emit_no_source",
        invalid_ranges_limits_files_and_oversized_lines_emit_no_source,
    ),
    (
        "reader::empty_crlf_and_missing_final_newline_are_explicit_and_preserved",
        empty_crlf_and_missing_final_newline_are_explicit_and_preserved,
    ),
    (
        "reader::missing_batch_manifest_reports_the_requested_file_without_source_output",
        missing_batch_manifest_reports_the_requested_file_without_source_output,
    ),
    (
        "reader::batch_continuations_share_one_budget_and_deliver_each_selected_line_once",
        batch_continuations_share_one_budget_and_deliver_each_selected_line_once,
    ),
    (
        "reader::batch_continuation_binds_even_unread_sources_and_the_request_list",
        batch_continuation_binds_even_unread_sources_and_the_request_list,
    ),
    (
        "reader::batch_validates_all_excerpts_before_output_and_preserves_empty_and_crlf_sources",
        batch_validates_all_excerpts_before_output_and_preserves_empty_and_crlf_sources,
    ),
];
