use srtlib::Subtitle;

/// Returns an iterator that yields each line of the subtitle entry as a separate entry
pub fn flatten_subtitles(e: Subtitle) -> impl Iterator<Item = Subtitle> {
    let lines: Vec<String> = e.text.split('\n').map(str::to_string).collect();
    lines.into_iter().map(move |line| Subtitle {
        num: e.num,
        start_time: e.start_time,
        end_time: e.end_time,
        text: line,
    })
}
