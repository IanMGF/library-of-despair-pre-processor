use subparse::SubtitleEntry;

/// Returns an iterator that yields each line of the subtitle entry as a separate entry
pub fn flatten_subtitles(e: SubtitleEntry) -> impl Iterator<Item = SubtitleEntry> {
    let lines: Vec<String> = e.line.unwrap().split('\n').map(str::to_string).collect();
    lines.into_iter().map(move |line| SubtitleEntry {
        timespan: e.timespan,
        line: Some(line),
    })
}
