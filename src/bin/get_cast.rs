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
    rc::Rc,
};

use pre_processor::{
    parse_cast::{self, parse_speaker},
    setup_logging,
};
use subparse::{SrtFile, SubtitleFileInterface};

pub fn main() {
    let paths: Vec<PathBuf> = env::args().skip(1).map(PathBuf::from).collect();
    setup_logging();

    for filepath in paths {
        let subtitle_file: File = File::open(filepath).unwrap();
        let mut subtitle_content: String =
            String::with_capacity(subtitle_file.metadata().unwrap().size() as usize);
        let mut subtitle_reader = BufReader::new(subtitle_file);

        subtitle_reader
            .read_to_string(&mut subtitle_content)
            .unwrap();

        let srt_file: SrtFile = SrtFile::parse(&subtitle_content).unwrap();

        let mut cast: HashSet<Option<String>> = HashSet::new();

        let mut prev_speaker: Option<Rc<str>> = None;
        for entry in srt_file
            .get_subtitle_entries()
            .unwrap()
            .into_iter()
            .flat_map(pre_processor::flatten_subtitles::flatten_subtitles)
        {
            let speaker = match parse_speaker(entry.line.unwrap().as_str().into()) {
                parse_cast::SpeakerParseResult::Expected(sp, _) => Some(sp),
                _ => prev_speaker.clone(),
            };

            if cast.insert(speaker.as_ref().map(|s| s.to_string())) {
                println!("Adicionando: {:?}", speaker);
            }
            prev_speaker = speaker;
        }
    }
}
