use std::rc::Rc;

use crate::steps::{PreProcessingCtx, PreProcessingStep};
use pre_processor::parse_cast::{SpeakerParseResult, parse_speaker};

type SpeakerAlias = Rc<str>;
type Line = Rc<str>;

pub struct Parser;
impl PreProcessingStep<Line, (Option<SpeakerAlias>, Line)> for Parser {
    fn apply(line: Line, ctx: &PreProcessingCtx) -> (Option<SpeakerAlias>, Line) {
        match parse_speaker(line) {
            SpeakerParseResult::Expected(alias, line) => (Some(alias), line),
            SpeakerParseResult::ColonWithoutSpeech(line) => {
                log::info!(
                    "Empty Colon:\t\tTimestamp:{}  Line {:>5}  Text:{}",
                    ctx.subtitle_entry.timespan.start,
                    ctx.line_number + 1,
                    line
                );
                (None, line)
            }
            SpeakerParseResult::NoColon(line) => (None, line),
        }
    }
}

// Cellbit: Muito boa noite
// Speaker: "Cellbit"
// Texto: "Muito boa noite"
