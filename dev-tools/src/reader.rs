use clap::Args as ClapArgs;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{fs, num::NonZeroUsize, path::PathBuf};

mod search;
pub use search::{Args as SearchArgs, page as search};

#[derive(ClapArgs)]
pub struct Args {
    /// UTF-8 file to read; never modified.
    path: PathBuf,
    #[arg(long, default_value_t = 1)]
    start: usize,
    /// Inclusive last line (default: EOF).
    #[arg(long)]
    end: Option<usize>,
    /// Read up to this many lines from --start, stopping at EOF.
    #[arg(long, conflicts_with = "end")]
    lines: Option<NonZeroUsize>,
    /// Complete output limit in bytes (512..=12000), including metadata.
    #[arg(long, default_value_t = 6000, value_parser = clap::value_parser!(u32).range(512..=12000))]
    max_bytes: u32,
    /// Expected source hash from the preceding page.
    #[arg(long)]
    sha256: Option<String>,
}

pub fn page(args: Args) -> Result<String, String> {
    let data = fs::read(&args.path).map_err(|e| format!("{}: {e}", args.path.display()))?;
    let digest = format!("{:x}", Sha256::digest(&data));
    if args.sha256.as_ref().is_some_and(|hash| *hash != digest) {
        return Err("source changed; restart the read from the intended revision".into());
    }
    let text = std::str::from_utf8(&data).map_err(|e| format!("invalid UTF-8: {e}"))?;
    // Like rg -n: LF-delimited, preserving CRLF and a missing final newline.
    let lines: Vec<_> = text.split_inclusive('\n').collect();
    let end = range_end(args.start, args.end, args.lines, lines.len())?;
    let mut output = format!("SHA256 {digest}\n");
    if lines.is_empty() {
        output.push_str("EMPTY FILE\nEND\n");
        return Ok(output);
    }
    output.push_str(&format!(
        "SELECTED {}..{end} of {} lines\n",
        args.start,
        lines.len()
    ));
    let continuation = |next| {
        format!(
            "\nNEXT --start {next} --end {end} --max-bytes {} --sha256 {digest}\n",
            args.max_bytes
        )
    };
    let reserved = continuation(end + 1).len();
    let mut next = args.start;
    for number in args.start..=end {
        let row = format!("{number:6} | {}", lines[number - 1]);
        if output.len() + row.len() + reserved > args.max_bytes as usize {
            break;
        }
        output.push_str(&row);
        next = number + 1;
    }
    if next == args.start {
        return Err(format!(
            "line {} cannot fit this page; raise --max-bytes (up to 12000) or inspect that line separately; no source text was emitted",
            args.start
        ));
    }
    if next <= end {
        output.push_str(&continuation(next));
    } else if lines[end - 1].ends_with('\n') {
        output.push_str("\nEND\n");
    } else {
        output.push_str("\nEND (source has no final newline)\n");
    }
    Ok(output)
}

fn range_end(
    start: usize,
    end: Option<usize>,
    count: Option<NonZeroUsize>,
    total: usize,
) -> Result<usize, String> {
    if end.is_some() && count.is_some() {
        return Err("select either end or lines, not both".into());
    }
    if total == 0 && start == 1 && end.is_none() {
        return Ok(0);
    }
    let end = count.map_or_else(
        || end.unwrap_or(total),
        |count| start.saturating_add(count.get() - 1).min(total),
    );
    if start == 0 || start > end || end > total {
        return Err(format!(
            "select an inclusive range within 1..{total}; use --lines for an EOF-safe window or omit --end to read through EOF"
        ));
    }
    Ok(end)
}

#[derive(ClapArgs)]
pub struct BatchArgs {
    /// Path to a JSON file containing an array of {path, start?, end?} or {path, start?, lines?}; paths are relative to the working directory.
    #[arg(value_name = "REQUESTS_FILE")]
    requests: PathBuf,
    /// Complete output limit in bytes (512..=12000), shared by all excerpts including metadata.
    #[arg(long, default_value_t = 6000, value_parser = clap::value_parser!(u32).range(512..=12000))]
    max_bytes: u32,
    /// One-based request number from NEXT.
    #[arg(long, default_value_t = 1)]
    request: usize,
    /// First unread line in that request, from NEXT.
    #[arg(long)]
    start: Option<usize>,
    /// Expected hash of the request list and all sources, from NEXT.
    #[arg(long)]
    sha256: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Excerpt {
    path: PathBuf,
    #[serde(default = "first_line")]
    start: usize,
    end: Option<usize>,
    lines: Option<NonZeroUsize>,
}

fn first_line() -> usize {
    1
}

struct Source {
    label: String,
    lines: Vec<String>,
    start: usize,
    end: usize,
}

pub fn batch(args: BatchArgs) -> Result<String, String> {
    let manifest = fs::read(&args.requests)
        .map_err(|e| format!("cannot read requests file {}: {e}", args.requests.display()))?;
    let requests: Vec<Excerpt> = serde_json::from_slice(&manifest).map_err(|e| e.to_string())?;
    if !(1..=32).contains(&requests.len()) || !(1..=requests.len()).contains(&args.request) {
        return Err("select 1–32 excerpts and a request number within that list".into());
    }
    let mut digest = Sha256::new();
    digest.update((manifest.len() as u64).to_le_bytes());
    digest.update(&manifest);
    // Validate and bind every source before emitting any part of the batch.
    let sources: Vec<_> = requests
        .into_iter()
        .map(|request| {
            let data =
                fs::read(&request.path).map_err(|e| format!("{}: {e}", request.path.display()))?;
            digest.update((data.len() as u64).to_le_bytes());
            digest.update(&data);
            let text = std::str::from_utf8(&data).map_err(|e| format!("invalid UTF-8: {e}"))?;
            let lines: Vec<_> = text.split_inclusive('\n').map(str::to_owned).collect();
            let end = range_end(request.start, request.end, request.lines, lines.len())
                .map_err(|error| format!("{}: {error}", request.path.display()))?;
            Ok(Source {
                label: serde_json::to_string(&request.path).map_err(|e| e.to_string())?,
                lines,
                start: request.start,
                end,
            })
        })
        .collect::<Result<_, String>>()?;
    let digest = format!("{:x}", digest.finalize());
    if args
        .sha256
        .as_ref()
        .is_some_and(|expected| *expected != digest)
    {
        return Err(
            "request list or source changed; restart the batch from the intended revision".into(),
        );
    }
    let selected = &sources[args.request - 1];
    let first = args.start.unwrap_or(selected.start);
    if first < selected.start || first > selected.end.max(1) {
        return Err("continuation line lies outside the selected excerpt".into());
    }
    let next = |request, line| {
        format!(
            "\nNEXT --request {request} --start {line} --max-bytes {} --sha256 {digest}\n",
            args.max_bytes
        )
    };
    let reserve = next(
        sources.len(),
        sources.iter().map(|source| source.end).max().unwrap_or(1),
    )
    .len();
    let mut output = format!("BATCH SHA256 {digest}\n");
    let mut progressed = false;
    for (index, source) in sources.iter().enumerate().skip(args.request - 1) {
        let start = if index == args.request - 1 {
            first
        } else {
            source.start
        };
        let header = format!(
            "\nREQUEST {} FILE {}\nSELECTED {}..{} of {} lines\n",
            index + 1,
            source.label,
            source.start,
            source.end,
            source.lines.len()
        );
        let end_marker = if source
            .lines
            .last()
            .is_some_and(|line| !line.ends_with('\n'))
            && source.end == source.lines.len()
        {
            "\nEND REQUEST (source has no final newline)\n"
        } else {
            "\nEND REQUEST\n"
        };
        if source.lines.is_empty() {
            let block = format!("{header}EMPTY FILE{end_marker}");
            if output.len() + block.len() + reserve > args.max_bytes as usize {
                return finish_batch_page(output, progressed, next(index + 1, start));
            }
            output.push_str(&block);
            progressed = true;
            continue;
        }
        for number in start..=source.end {
            let row = format!("{number:6} | {}", source.lines[number - 1]);
            let prefix = if number == start { header.as_str() } else { "" };
            if output.len() + prefix.len() + row.len() + end_marker.len() + reserve
                > args.max_bytes as usize
            {
                return finish_batch_page(output, progressed, next(index + 1, number));
            }
            output.push_str(prefix);
            output.push_str(&row);
            progressed = true;
        }
        output.push_str(end_marker);
    }
    output.push_str("\nEND\n");
    Ok(output)
}

fn finish_batch_page(mut output: String, progressed: bool, next: String) -> Result<String, String> {
    if !progressed {
        return Err("excerpt header or next whole line cannot fit; raise --max-bytes (up to 12000); no source text was emitted".into());
    }
    output.push_str(&next);
    Ok(output)
}
