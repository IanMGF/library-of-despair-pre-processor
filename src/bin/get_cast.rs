//! Outputs all identified speakers in a `.srt` file.
//!
//! A "speaker" is identified when a line in the subtitles has a format
//! `A:B`
//! With `A` and `B` being arbitrary strings.
//! This code has the merely auxiliary purpose of allowing
//! the developers to quickly see all mentioned "speakers" (also referred to as "cast members")

use std::{
    collections::HashSet,
    env,
    fs::File,
    io::{BufReader, Read},
    os::unix::fs::MetadataExt,
    path::PathBuf,
    sync::Arc,
};

use backend::archive::cast::Cast;
use pre_processor::{
    parse_cast::{self, parse_speaker},
    setup_logging,
};
use subparse::{SrtFile, SubtitleFileInterface};

pub fn main() {
    setup_logging();

    let filepath = PathBuf::from(env::args().skip(1).next().unwrap());
    let comparing_cast: Cast = env::args()
        .skip(2)
        .next()
        .map(PathBuf::from)
        .map(File::open)
        .map(Result::unwrap)
        .map(yaml_serde::from_reader)
        .map(Result::unwrap)
        .unwrap_or_else(|| Cast(HashSet::new()));

    let subtitle_file: File = File::open(filepath).unwrap();
    let mut subtitle_content: String =
        String::with_capacity(subtitle_file.metadata().unwrap().size() as usize);
    let mut subtitle_reader = BufReader::new(subtitle_file);

    subtitle_reader
        .read_to_string(&mut subtitle_content)
        .unwrap();

    let srt_file: SrtFile = SrtFile::parse(&subtitle_content).unwrap();
    let mut unknown_cast: HashSet<String> = HashSet::new();

    let mut prev_speaker: Option<Arc<str>> = None;
    for entry in srt_file
        .get_subtitle_entries()
        .unwrap()
        .into_iter()
        .flat_map(pre_processor::flatten_subtitles::flatten_subtitles)
    {
        let mut line: &str = entry.line.as_ref().unwrap().as_str();
        if line.starts_with('(') {
            line = line.trim_start_matches('(');
        }

        let speaker = match parse_speaker(line.into()) {
            parse_cast::SpeakerParseResult::Expected(sp, _) => Some(sp),
            _ => prev_speaker.clone(),
        };

        let member_alias_opt = speaker.as_ref().map(|s| s.to_string());
        if let Some(member_alias) = member_alias_opt {
            let member_exists = comparing_cast.get_member_by_id(&member_alias).is_some();

            let inserted_in_unknown = unknown_cast.insert(member_alias.clone());
            if inserted_in_unknown && member_exists {
                log::info!(
                    "Existe em arquivo:     {} -  [ ] {:?}",
                    entry.timespan.start,
                    speaker
                );
            } else if inserted_in_unknown {
                log::warn!(
                    "Não existe em arquivo: {} -  [X] {:?}",
                    entry.timespan.start,
                    speaker
                );
                unknown_cast.insert(member_alias);
            }
        } else {
            if prev_speaker.is_none() {
                log::error!(
                    "Falante inexistente  : {} - {:?}",
                    entry.timespan.start,
                    speaker
                );
            }
        }

        prev_speaker = speaker;
    }
}
