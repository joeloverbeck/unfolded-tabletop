//! The `compare-captures` binary on pictures each test makes. Every expected count,
//! difference and box is worked by hand from the pixels the test changes.

use image::{Rgba, RgbaImage};
use std::{path::Path, process::Command};

/// An 8 by 6 picture of one colour.
fn plain() -> RgbaImage {
    RgbaImage::from_pixel(8, 6, Rgba([10, 20, 30, 255]))
}

/// `plain` with blue raised by 100 at (2, 1) and red by 5 at (5, 4).
fn changed() -> RgbaImage {
    let mut picture = plain();
    picture.put_pixel(2, 1, Rgba([10, 20, 130, 255]));
    picture.put_pixel(5, 4, Rgba([15, 20, 30, 255]));
    picture
}

struct Run {
    code: Option<i32>,
    stdout: String,
    stderr: String,
}

/// Write `before` and `after` as PNGs and compare them with `options`.
fn compare(before: &RgbaImage, after: &RgbaImage, options: &[&str]) -> Run {
    let directory = tempfile::tempdir().unwrap();
    let (first, second) = (
        directory.path().join("a.png"),
        directory.path().join("b.png"),
    );
    before.save(&first).unwrap();
    after.save(&second).unwrap();
    run_on(&first, &second, options)
}

fn run_on(first: &Path, second: &Path, options: &[&str]) -> Run {
    let output = Command::new(env!("CARGO_BIN_EXE_compare-captures"))
        .arg(first)
        .arg(second)
        .args(options)
        .output()
        .unwrap();
    Run {
        code: output.status.code(),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

#[test]
fn identical_pictures_differ_nowhere() {
    let run = compare(&plain(), &plain(), &[]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert_eq!(
        run.stdout,
        "threshold: 0, limit: 0, ignored: none\n\
         differing pixels: 0\n\
         largest difference: 0\n\
         bounding box: none\n"
    );
}

#[test]
fn a_known_change_reports_its_count_largest_difference_and_box() {
    let run = compare(&plain(), &changed(), &[]);
    assert_eq!(run.code, Some(1), "{}", run.stderr);
    // Columns 2 to 5 and rows 1 to 4.
    assert_eq!(
        run.stdout,
        "threshold: 0, limit: 0, ignored: none\n\
         differing pixels: 2\n\
         largest difference: 100\n\
         bounding box: 2,1,4,4\n"
    );
}

#[test]
fn a_change_inside_the_ignored_rectangle_is_not_counted() {
    let run = compare(&plain(), &changed(), &["--ignore", "1,0,2,2"]);
    assert_eq!(run.code, Some(1), "{}", run.stderr);
    assert_eq!(
        run.stdout,
        "threshold: 0, limit: 0, ignored: 1,0,2,2\n\
         differing pixels: 1\n\
         largest difference: 5\n\
         bounding box: 5,4,1,1\n"
    );
}

#[test]
fn a_difference_at_the_threshold_does_not_count() {
    let run = compare(&plain(), &changed(), &["--threshold", "5"]);
    assert_eq!(run.code, Some(1), "{}", run.stderr);
    assert!(
        run.stdout
            .ends_with("differing pixels: 1\nlargest difference: 100\nbounding box: 2,1,1,1\n"),
        "{}",
        run.stdout
    );
    let above = compare(&plain(), &changed(), &["--threshold", "100"]);
    assert_eq!(above.code, Some(0), "{}", above.stderr);
    assert!(
        above.stdout.contains("differing pixels: 0\n"),
        "{}",
        above.stdout
    );
}

#[test]
fn a_count_within_the_limit_passes() {
    let at = compare(&plain(), &changed(), &["--limit", "2"]);
    assert_eq!(at.code, Some(0), "{}", at.stderr);
    let below = compare(&plain(), &changed(), &["--limit", "1"]);
    assert_eq!(below.code, Some(1), "{}", below.stderr);
}

#[test]
fn a_change_in_transparency_alone_differs() {
    let mut after = plain();
    after.put_pixel(7, 5, Rgba([10, 20, 30, 0]));
    let run = compare(&plain(), &after, &[]);
    assert_eq!(run.code, Some(1), "{}", run.stderr);
    assert!(
        run.stdout
            .ends_with("differing pixels: 1\nlargest difference: 255\nbounding box: 7,5,1,1\n"),
        "{}",
        run.stdout
    );
}

#[test]
fn pictures_of_different_sizes_are_an_error() {
    let run = compare(&plain(), &RgbaImage::new(8, 5), &[]);
    assert_eq!(run.code, Some(2), "{}", run.stdout);
    assert!(
        run.stderr.contains("8x6") && run.stderr.contains("8x5"),
        "{}",
        run.stderr
    );
    assert!(run.stdout.is_empty(), "{}", run.stdout);
}

#[test]
fn a_file_that_is_not_a_png_is_an_error() {
    let directory = tempfile::tempdir().unwrap();
    let picture = directory.path().join("a.png");
    plain().save(&picture).unwrap();
    let text = directory.path().join("b.png");
    std::fs::write(&text, "not a picture").unwrap();
    let run = run_on(&picture, &text, &[]);
    assert_eq!(run.code, Some(2), "{}", run.stdout);
    assert!(run.stderr.contains("b.png is not a PNG"), "{}", run.stderr);
    let missing = run_on(&picture, &directory.path().join("none.png"), &[]);
    assert_eq!(missing.code, Some(2), "{}", missing.stdout);
}

#[test]
fn the_printed_box_can_be_ignored_as_printed() {
    // The box `a_known_change_…` prints, handed back as `--ignore`, leaves nothing.
    let run = compare(&plain(), &changed(), &["--ignore", "2,1,4,4"]);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert!(
        run.stdout.contains("differing pixels: 0\n"),
        "{}",
        run.stdout
    );
}

#[test]
fn a_malformed_rectangle_is_an_error() {
    let run = compare(&plain(), &plain(), &["--ignore", "1,2,3"]);
    assert_eq!(run.code, Some(2), "{}", run.stdout);
}

#[test]
fn help_names_the_exit_statuses() {
    let output = Command::new(env!("CARGO_BIN_EXE_compare-captures"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let text = String::from_utf8(output.stdout).unwrap();
    let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        words.contains(
            "Exit status: 0 when the count is within the limit, 1 when it exceeds it, and 2 on an error"
        ),
        "{text}"
    );
    assert!(text.contains("--threshold <THRESHOLD>"), "{text}");
}

/// Two captures as `held-start` writes them, `before.table.png` and `after.table.png`,
/// each beside its case's record naming the executable that made it.
fn compare_recorded(before_executable: &str, after_executable: &str) -> Run {
    let directory = tempfile::tempdir().unwrap();
    for (case, executable) in [("before", before_executable), ("after", after_executable)] {
        plain()
            .save(directory.path().join(format!("{case}.table.png")))
            .unwrap();
        std::fs::write(
            directory.path().join(format!("{case}.record")),
            format!("command: game --socket x\n{executable}\nfirst status: ready\n"),
        )
        .unwrap();
    }
    run_on(
        &directory.path().join("before.table.png"),
        &directory.path().join("after.table.png"),
        &[],
    )
}

const ONE: &str = "executable: /games/one sha256 1111111111111111111111111111111111111111111111111111111111111111";
const TWO: &str = "executable: /games/two sha256 2222222222222222222222222222222222222222222222222222222222222222";

#[test]
fn captures_recorded_from_one_executable_are_said_to_show_no_change_between_builds() {
    let run = compare_recorded(ONE, ONE);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert!(
        run.stdout.ends_with(
            "executables: the same, sha256 1111111111111111111111111111111111111111111111111111111111111111; \
             these captures cannot show a change between builds\n"
        ),
        "{}",
        run.stdout
    );
}

#[test]
fn captures_recorded_from_different_executables_say_so() {
    let run = compare_recorded(ONE, TWO);
    assert_eq!(run.code, Some(0), "{}", run.stderr);
    assert!(
        run.stdout
            .ends_with("bounding box: none\nexecutables: different\n"),
        "{}",
        run.stdout
    );

    // A record from before executables were recorded says nothing of them.
    let run = compare_recorded(ONE, "revision: none");
    assert!(
        run.stdout.ends_with("bounding box: none\n"),
        "{}",
        run.stdout
    );
}
