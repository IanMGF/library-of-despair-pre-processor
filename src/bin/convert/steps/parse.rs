use std::sync::Arc;

use crate::steps::{PreProcessingCtx, PreProcessingStep};
use pre_processor::parse_cast::{SpeakerParseResult, parse_speaker};

type SpeakerAlias = Arc<str>;
type Line = Arc<str>;

pub struct Parser;
impl PreProcessingStep<Line, (Option<SpeakerAlias>, Line)> for Parser {
    fn apply(line: Line, ctx: &PreProcessingCtx) -> (Option<SpeakerAlias>, Line) {
        match parse_speaker(line) {
            SpeakerParseResult::Expected(alias, line) => (Some(alias), line),
            SpeakerParseResult::ColonWithoutSpeech(line) => {
                log::info!(
                    "Empty Colon:\t\tTimestamp:{}  Line {:>5}  Text:{}",
                    ctx.subtitle.start_time,
                    ctx.line_number + 1,
                    line
                );
                (None, line)
            }
            SpeakerParseResult::NoColon(line) => (None, line),
        }
    }
}
