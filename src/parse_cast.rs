use std::sync::Arc;

pub enum SpeakerParseResult {
    Expected(Arc<str>, Arc<str>),
    ColonWithoutSpeech(Arc<str>),
    NoColon(Arc<str>),
}

/// Processes a line by splitting off the first `:` character.
/// Returns '(Some(speaker), line)` if a speaker is present and identified,
/// and `(None, line)` otherwise
pub fn parse_speaker(line: Arc<str>) -> SpeakerParseResult {
    let (speaker, processed_line) = match line.split_once(":") {
        Some((a, b)) => (Arc::<str>::from(a.trim()), Arc::<str>::from(b.trim())),
        None => return SpeakerParseResult::NoColon(line),
    };

    // A subtitle should not be in the format
    // 'Character Speaks:{empty}'
    if processed_line.is_empty() {
        return SpeakerParseResult::ColonWithoutSpeech(line);
    }

    return SpeakerParseResult::Expected(speaker, processed_line);
}
