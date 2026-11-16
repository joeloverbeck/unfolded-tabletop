//! Compare two captures of the same table, pixel by pixel. `ABOUT`, which
//! `compare-captures --help` prints, describes the comparison and its exit statuses.

use anyhow::{Context, Result, bail};
use clap::Parser;
use image::RgbaImage;
use std::{
    fmt,
    path::{Path, PathBuf},
    process::ExitCode,
    str::FromStr,
};

/// What `--help` prints above the options.
const ABOUT: &str = "\
Compare two captures of the same table, pixel by pixel.

A pixel differs when its largest difference in red, green, blue or transparency
exceeds the threshold. Pixels inside the ignored rectangle are left out. It prints its
options, the count of differing pixels, the largest difference outside the ignored
rectangle and the box around the differing pixels, written as x,y,width,height so it
can be handed back as --ignore.

When each picture sits beside the `held-start` record of its case (`<case>.record`
for `<case>.<name>`), and both records name their executable's SHA-256, it also says
whether the executables were the same. Two captures from one executable cannot show
a change between builds, however they compare.

Exit status: 0 when the count is within the limit, 1 when it exceeds it, and 2 on an
error: an unreadable file, one whose contents are not PNG, pictures of different sizes
or malformed options.

Two held starts of one unchanged table can differ by rendering noise; raise
--threshold to the noise a table is known to have, or --ignore a region that differs
on purpose.";

#[derive(Parser)]
#[command(
    name = "compare-captures",
    about = "Compare two captures of the same table, pixel by pixel.",
    long_about = ABOUT
)]
struct Args {
    before: PathBuf,
    after: PathBuf,
    /// Leave out the pixels of this rectangle: `x,y,width,height`.
    #[arg(long)]
    ignore: Option<Rectangle>,
    /// The largest channel difference a pixel may have and still not differ.
    #[arg(long, default_value_t = 0)]
    threshold: u8,
    /// The most differing pixels that still pass.
    #[arg(long, default_value_t = 0)]
    limit: u64,
}

/// A rectangle of pixels; written and read as `x,y,width,height`, so a printed box
/// can be handed back as `--ignore`.
#[derive(Clone, Copy)]
struct Rectangle {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl Rectangle {
    fn holds(self, x: u32, y: u32) -> bool {
        x >= self.x && x - self.x < self.width && y >= self.y && y - self.y < self.height
    }
}

impl FromStr for Rectangle {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, String> {
        let numbers = text
            .split(',')
            .map(|number| number.trim().parse::<u32>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| format!("{text:?}: {error}"))?;
        let [x, y, width, height] = numbers[..] else {
            return Err(format!("{text:?} is not x,y,width,height"));
        };
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }
}

impl fmt::Display for Rectangle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{},{},{},{}",
            self.x, self.y, self.width, self.height
        )
    }
}

fn main() -> ExitCode {
    let args = Args::parse();
    match compare(&args) {
        Ok(count) if count <= args.limit => ExitCode::SUCCESS,
        Ok(_) => ExitCode::from(1),
        Err(error) => {
            eprintln!("compare-captures: {error:#}");
            ExitCode::from(2)
        }
    }
}

/// Print the comparison and return the count of differing pixels.
fn compare(args: &Args) -> Result<u64> {
    let before = picture(&args.before)?;
    let after = picture(&args.after)?;
    if before.dimensions() != after.dimensions() {
        bail!(
            "{} is {}x{} and {} is {}x{}",
            args.before.display(),
            before.width(),
            before.height(),
            args.after.display(),
            after.width(),
            after.height()
        );
    }
    let mut count = 0;
    let mut largest = 0;
    let mut bounds: Option<Rectangle> = None;
    for (x, y, pixel) in before.enumerate_pixels() {
        if args.ignore.is_some_and(|ignored| ignored.holds(x, y)) {
            continue;
        }
        let difference = pixel
            .0
            .iter()
            .zip(after.get_pixel(x, y).0)
            .map(|(&first, second)| first.abs_diff(second))
            .max()
            .unwrap_or(0);
        largest = largest.max(difference);
        if difference > args.threshold {
            count += 1;
            bounds = Some(match bounds {
                None => Rectangle {
                    x,
                    y,
                    width: 1,
                    height: 1,
                },
                Some(seen) => {
                    let (left, top) = (seen.x.min(x), seen.y.min(y));
                    let right = (seen.x + seen.width).max(x + 1);
                    let bottom = (seen.y + seen.height).max(y + 1);
                    Rectangle {
                        x: left,
                        y: top,
                        width: right - left,
                        height: bottom - top,
                    }
                }
            });
        }
    }
    let ignored = args
        .ignore
        .map_or_else(|| "none".to_owned(), |ignored| ignored.to_string());
    println!(
        "threshold: {}, limit: {}, ignored: {ignored}",
        args.threshold, args.limit
    );
    println!("differing pixels: {count}");
    println!("largest difference: {largest}");
    match bounds {
        Some(bounds) => println!("bounding box: {bounds}"),
        None => println!("bounding box: none"),
    }
    if let (Some(before), Some(after)) = (built(&args.before), built(&args.after)) {
        if before == after {
            println!(
                "executables: the same, sha256 {before}; these captures cannot show a change between builds"
            );
        } else {
            println!("executables: different");
        }
    }
    Ok(count)
}

/// The SHA-256 of the executable that made a capture, from the `held-start` record
/// of its case: `<case>.record` beside `<case>.<name>`, the longest such case first.
fn built(capture: &Path) -> Option<String> {
    let directory = capture.parent()?;
    let name = capture.file_name()?.to_str()?;
    let record = name
        .match_indices('.')
        .rev()
        .map(|(end, _)| directory.join(format!("{}.record", &name[..end])))
        .find(|record| record.is_file())?;
    let text = std::fs::read_to_string(record).ok()?;
    text.lines()
        .find_map(|line| line.strip_prefix("executable: "))
        .and_then(|executable| executable.rsplit_once(" sha256 "))
        .map(|(_, digest)| digest.to_owned())
}

fn picture(path: &PathBuf) -> Result<RgbaImage> {
    let bytes = std::fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
    // Judged by its contents, not its name.
    if image::guess_format(&bytes).ok() != Some(image::ImageFormat::Png) {
        bail!("{} is not a PNG", path.display());
    }
    Ok(
        image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
            .with_context(|| format!("cannot decode {}", path.display()))?
            .into_rgba8(),
    )
}
