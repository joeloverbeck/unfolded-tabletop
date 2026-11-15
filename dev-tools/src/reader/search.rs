use clap::Args as ClapArgs;
use sha2::{Digest, Sha256};
use std::{fs, path::PathBuf};

#[derive(ClapArgs)]
#[command(
    after_help = "For NEXT, repeat the same path and query/scope options; replace only the previous continuation arguments with those printed after NEXT."
)]
pub struct Args {
    /// Explicit UTF-8 files to search in order; never modified (at most 32).
    #[arg(required = true, num_args = 1..=32)]
    path: Vec<PathBuf>,
    /// Nonempty, case-sensitive literal text on one line (not a regular expression).
    #[arg(long, required = true, allow_hyphen_values = true)]
    find: Vec<String>,
    /// Inclusive first line of the search scope.
    #[arg(long, default_value_t = 1)]
    start: usize,
    /// Inclusive last line of the search scope (default: EOF).
    #[arg(long)]
    end: Option<usize>,
    /// Exact ATX heading line, including # prefix; scopes through its child sections.
    #[arg(long)]
    heading: Option<String>,
    /// Lines of context around each match, clipped to the selected scope.
    #[arg(long, default_value_t = 0)]
    context: usize,
    /// Complete output limit in bytes (512..=12000), including metadata.
    #[arg(long, default_value_t = 6000, value_parser = clap::value_parser!(u32).range(512..=12000))]
    max_bytes: u32,
    /// First unread result line from NEXT; repeat the original search options.
    #[arg(long, requires = "sha256")]
    next_line: Option<usize>,
    /// One-based file index from NEXT. Line zero starts that file's results.
    #[arg(long, requires = "next_line")]
    next_file: Option<usize>,
    /// Expected hash of the source, literal query and scope from the preceding page.
    #[arg(long)]
    sha256: Option<String>,
}

pub fn page(args: Args) -> Result<String, String> {
    if args
        .find
        .iter()
        .any(|term| term.is_empty() || term.contains(['\r', '\n']))
    {
        return Err("--find must contain nonempty literal text on one line".into());
    }
    let sources: Vec<_> = args
        .path
        .iter()
        .map(|path| fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display())))
        .collect::<Result<_, _>>()?;
    let identities: Vec<_> = args
        .path
        .iter()
        .zip(&sources)
        .map(|(path, source)| {
            Ok((
                path,
                path.canonicalize().map_err(|e| e.to_string())?,
                format!("{:x}", Sha256::digest(source.as_bytes())),
            ))
        })
        .collect::<Result<_, String>>()?;
    let identity = serde_json::to_vec(&(
        identities,
        args.start,
        args.end,
        &args.find,
        &args.heading,
        args.context,
    ))
    .map_err(|e| e.to_string())?;
    let digest = format!("{:x}", Sha256::digest(identity));
    if args
        .sha256
        .as_ref()
        .is_some_and(|expected| *expected != digest)
    {
        return Err("source or search options changed; restart the search".into());
    }
    // Validate every scope before emitting even the first file's results.
    let files: Vec<_> = sources
        .iter()
        .map(|source| selected(source, &args))
        .collect::<Result<_, _>>()?;
    render(&args, &files, &digest)
}

struct Selected<'a> {
    lines: Vec<&'a str>,
    numbers: Vec<usize>,
    header: String,
}

fn selected<'a>(source: &'a str, args: &Args) -> Result<Selected<'a>, String> {
    let lines: Vec<_> = source.split_inclusive('\n').collect();
    let end = args.end.unwrap_or(lines.len());
    let empty = lines.is_empty() && args.start == 1 && args.end.is_none();
    if !empty && (args.start == 0 || args.start > end || end > lines.len()) {
        return Err(format!(
            "select an inclusive range within 1..{}",
            lines.len()
        ));
    }
    let (start, end) = match args.heading.as_deref() {
        Some(heading) => heading_scope(&lines, heading, args.start, end)?,
        None => (args.start, end),
    };
    let matches: Vec<_> = (start..=end)
        .filter(|number| {
            args.find
                .iter()
                .any(|term| lines[number - 1].contains(term))
        })
        .collect();
    let numbers = context_lines(&matches, args.context, start, end);
    let scope = if empty {
        "EMPTY FILE".to_owned()
    } else {
        format!("SELECTED {start}..{end} of {} lines", lines.len())
    };
    let header = format!(
        "{scope}\nMATCHES {}\n{}",
        matches.len(),
        if matches.is_empty() {
            "NO MATCHES\n"
        } else {
            ""
        }
    );
    Ok(Selected {
        lines,
        numbers,
        header,
    })
}

fn render(args: &Args, files: &[Selected<'_>], digest: &str) -> Result<String, String> {
    let multiple = files.len() > 1;
    let first_file = args
        .next_file
        .unwrap_or(1)
        .checked_sub(1)
        .filter(|index| *index < files.len())
        .ok_or("continuation file is outside the search results")?;
    let first_row = match args.next_line {
        Some(0) if multiple => 0,
        Some(number) => files[first_file]
            .numbers
            .binary_search(&number)
            .map_err(|_| "continuation line is outside the search results")?,
        None => 0,
    };
    let next = |file: usize, number: usize| {
        let file = if multiple {
            format!("--next-file {} ", file + 1)
        } else {
            String::new()
        };
        format!(
            "\nNEXT {file}--next-line {number} --max-bytes {} --sha256 {digest}\n",
            args.max_bytes
        )
    };
    let reserved = next(
        files.len() - 1,
        files.iter().map(|file| file.lines.len()).max().unwrap_or(0),
    )
    .len()
    .max(40);
    let mut output = format!("SEARCH SHA256 {digest}\n");
    let mut progressed = false;
    for (index, file) in files.iter().enumerate().skip(first_file) {
        let first = if index == first_file { first_row } else { 0 };
        let mut header = if multiple {
            format!(
                "FILE {} {}\n",
                index + 1,
                serde_json::to_string(&args.path[index]).map_err(|e| e.to_string())?
            )
        } else {
            String::new()
        };
        header.push_str(&file.header);
        let minimum = file.numbers.get(first).map_or(0, |number| {
            format!("{number:6} | {}", file.lines[number - 1]).len()
        });
        if output.len() + header.len() + minimum + reserved > args.max_bytes as usize {
            if !progressed {
                return Err("next file header or line cannot fit; raise --max-bytes (up to 12000); no source text was emitted".into());
            }
            output.push_str(&next(index, 0));
            return Ok(output);
        }
        output.push_str(&header);
        let mut previous = None;
        for &number in &file.numbers[first..] {
            let gap = if previous.is_some_and(|last| number != last + 1) {
                "...\n"
            } else {
                ""
            };
            let row = format!("{gap}{number:6} | {}", file.lines[number - 1]);
            if output.len() + row.len() + reserved > args.max_bytes as usize {
                output.push_str(&next(index, number));
                return Ok(output);
            }
            output.push_str(&row);
            previous = Some(number);
        }
        let missing_newline = previous.is_some_and(|n| !file.lines[n - 1].ends_with('\n'));
        if multiple {
            if missing_newline {
                output.push('\n');
            }
            output.push_str("END FILE\n");
        } else if missing_newline {
            output.push_str("\nEND (source has no final newline)\n");
            return Ok(output);
        }
        progressed = true;
    }
    output.push_str("\nEND\n");
    Ok(output)
}

fn heading_scope(
    lines: &[&str],
    heading: &str,
    start: usize,
    end: usize,
) -> Result<(usize, usize), String> {
    if heading.contains(['\r', '\n']) || heading_level(heading).is_none() {
        return Err("--heading must be an exact ATX heading line, including its # prefix".into());
    }
    let mut fence = None;
    let mut headings = Vec::new();
    for (index, source) in lines.iter().enumerate() {
        let Some(line) = markdown_line(source) else {
            continue;
        };
        let marker = fence_marker(line);
        if let Some((character, length)) = fence {
            if marker
                .is_some_and(|(c, n, tail)| c == character && n >= length && tail.trim().is_empty())
            {
                fence = None;
            }
            continue;
        }
        if let Some((character, length, tail)) = marker
            && (character != b'`' || !tail.contains('`'))
        {
            fence = Some((character, length));
            continue;
        }
        if let Some(level) = heading_level(source) {
            headings.push((index + 1, level));
        }
    }
    let mut found = headings.iter().filter(|(number, _)| {
        (start..=end).contains(number)
            && lines[number - 1].trim_end_matches(['\r', '\n']) == heading
    });
    let &(first, level) = found
        .next()
        .ok_or("heading not found in the selected range")?;
    if found.next().is_some() {
        return Err("heading is ambiguous in the selected range; narrow --start/--end".into());
    }
    let last = headings
        .iter()
        .find(|(number, depth)| *number > first && *depth <= level)
        .map_or(end, |(number, _)| end.min(number - 1));
    Ok((first, last))
}

fn markdown_line(line: &str) -> Option<&str> {
    let line = line.trim_end_matches(['\r', '\n']);
    let indent = line.bytes().take_while(|byte| *byte == b' ').count();
    (indent <= 3).then_some(&line[indent..])
}

fn heading_level(line: &str) -> Option<usize> {
    let line = markdown_line(line)?;
    let level = line.bytes().take_while(|byte| *byte == b'#').count();
    ((1..=6).contains(&level)
        && line
            .as_bytes()
            .get(level)
            .is_none_or(|b| matches!(b, b' ' | b'\t')))
    .then_some(level)
}

fn fence_marker(line: &str) -> Option<(u8, usize, &str)> {
    let character = *line.as_bytes().first()?;
    if !matches!(character, b'`' | b'~') {
        return None;
    }
    let length = line.bytes().take_while(|byte| *byte == character).count();
    (length >= 3).then_some((character, length, &line[length..]))
}

fn context_lines(matches: &[usize], context: usize, start: usize, end: usize) -> Vec<usize> {
    let mut ranges: Vec<(usize, usize)> = Vec::new();
    for &number in matches {
        let first = number.saturating_sub(context).max(start);
        let last = number.saturating_add(context).min(end);
        if let Some(range) = ranges.last_mut()
            && first <= range.1.saturating_add(1)
        {
            range.1 = range.1.max(last);
        } else {
            ranges.push((first, last));
        }
    }
    ranges
        .into_iter()
        .flat_map(|(first, last)| first..=last)
        .collect()
}
