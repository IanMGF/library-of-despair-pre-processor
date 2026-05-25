//! Conversion code to turn original `.srt` files into a (`cast.yaml`,  `assignments.yaml`, `content.txt`) triple
pub mod steps;

use backend::assignments::{AssignmentSet, OwnedAssignmentSet};
use backend::cast::{Cast, CastMember};
use pre_processor::setup_logging;
use serde::Serialize;
use std::ffi::OsStr;
use std::io::Write;
use std::{
    collections::HashSet,
    env,
    fs::File,
    io::{BufReader, BufWriter, Read},
    iter::Map,
    path::PathBuf,
    rc::Rc,
};
use subparse::{SrtFile, SubtitleEntry, SubtitleFileInterface};

use crate::steps::enclosers::{Encloser, RemoveEnclosers, RemovedEncloserResult};
use crate::steps::find_cast::FindCastMember;
use crate::steps::parse::Parser;
use crate::steps::{PreProcessingCtx, PreProcessingStep};

fn main() {
    let args: Vec<PathBuf> = env::args().skip(1).map(PathBuf::from).collect();
    setup_logging();

    let srt_filepath = args.get(0).unwrap();
    let cast_filepath = args.get(1).unwrap();

    if srt_filepath.extension().unwrap_or(OsStr::new("")) != OsStr::new("srt") {
        log::warn!(
            "Filepath '{}' does not have the '.srt' file format extension - Found extension: {}",
            srt_filepath.to_string_lossy(),
            srt_filepath
                .extension()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap()
        );
    }
    if cast_filepath.extension().unwrap_or(OsStr::new("")) != OsStr::new("yaml") {
        log::warn!(
            "Filepath '{}' does not have the '.yaml' file format extension - Found extension: {}",
            cast_filepath.to_string_lossy(),
            cast_filepath
                .extension()
                .unwrap_or(OsStr::new(""))
                .to_str()
                .unwrap(),
        );
    }

    // Load SRT file
    let subtitle_file: File = File::open(srt_filepath).unwrap();
    let mut subtitle_content: String = String::new();
    let mut subtitle_reader = BufReader::new(subtitle_file);
    subtitle_reader
        .read_to_string(&mut subtitle_content)
        .unwrap();
    let srt_file: SrtFile = SrtFile::parse(&subtitle_content).unwrap();

    // Load YAML Cast file
    let cast: Cast = yaml_serde::from_reader(File::open(cast_filepath).unwrap()).unwrap();

    let mut content: String = String::new();
    let mut assignments: AssignmentSet = AssignmentSet(Vec::new());

    // Store member who was previously speaking,
    // so that it can recognize
    // "Member: Line 1 \n Line2 \n Line3"
    let mut prev_speaker: Option<Rc<CastMember>> = None;

    let line_entry_iter = srt_file
        .get_subtitle_entries()
        .unwrap()
        .into_iter()
        .flat_map(flatten_subtitles)
        .enumerate();

    for (i, entry) in line_entry_iter {
        let encloser_opt: Option<Encloser>;
        let ctx: PreProcessingCtx;
        let og_line_rc: Rc<str>;
        let line_rc: Rc<str>;

        // Store original line, for later retrieval in case the processing goes wrong
        let original_line = entry.line.clone().unwrap();

        ctx = PreProcessingCtx {
            subtitle_entry: &entry,
            line_number: i,
        };

        og_line_rc = entry.line.as_ref().unwrap().as_str().into();

        // Remove enclosers and reassign text
        RemovedEncloserResult {
            encloser: encloser_opt,
            text: line_rc,
        } = RemoveEnclosers::apply(og_line_rc, &ctx);

        if let Some(ref encloser) = encloser_opt {
            content.push(encloser.first_char());
        }

        // Parse speaker
        let (curr_speaker_tag, line) = Parser::apply(line_rc.clone(), &ctx);

        // If current text does not have a proper speaker, assign the previous speaker
        let curr_speaker_tag = match (curr_speaker_tag, &prev_speaker) {
            (Some(s), _) => s,
            (None, Some(s)) => s.aliases[0].as_str().into(),
            (None, None) => {
                log::warn!("Speaker undetected for line {}", i + 1);
                content.push_str(line_rc.trim());
                content.push('\n');

                let timestamp = entry.timespan.start.msecs();
                assignments.0.push((timestamp, HashSet::new()));
                continue;
            }
        };

        // Find the speaker
        let speaker = FindCastMember::apply((curr_speaker_tag.clone(), &cast), &ctx);

        // Guarantee the alias given was found in the cast file
        let Some(speaker) = speaker else {
            log::warn!(
                "Alias not found:\tTimestamp:{:>16}  Line {:>5}  Alias:{}",
                entry.timespan.start,
                i + 1,
                curr_speaker_tag
            );
            content.push_str(original_line.trim());
            content.push('\n');
            let timestamp = entry.timespan.start.msecs();
            assignments.0.push((timestamp, HashSet::new()));
            continue;
        };

        // Insert final processed content into `content`
        content.push_str(line.trim());
        if let Some(ref encloser) = encloser_opt {
            content.push(encloser.last_char());
        }
        content.push('\n');

        // Insert speaker into `prev_speaker`
        prev_speaker = Some(speaker.clone());

        // Insert speaker into `assignments`
        match assignments.0.get_mut(i) {
            Some((_t, a)) => a.insert(speaker.id.as_str().into()),
            None => {
                let mut set = HashSet::new();
                let timestamp = entry.timespan.start.msecs();
                set.insert(speaker.id.as_str().into());
                assignments.0.push((timestamp, set));
                false
            }
        };
    }

    let content_file: File = File::create("content.txt").unwrap();
    let mut content_writer: BufWriter<File> = BufWriter::new(content_file);
    write!(content_writer, "{}", content).unwrap();

    let assignment_file: File = File::create("assignment.csv").unwrap();
    let mut csv_writer = csv::WriterBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_writer(assignment_file);

    for line_assignment in OwnedAssignmentSet::from(assignments).0 {
        #[derive(Serialize)]
        struct Record {
            time: i64,
            assignments: HashSet<String>
        }
        let record = Record {
            time: line_assignment.0,
            assignments: line_assignment.1,
        };
        csv_writer.serialize(record).unwrap();
    }
    // yaml_serde::to_writer(assignment_file, &OwnedAssignmentSet::from(assignments)).unwrap();
}

fn flatten_subtitles(
    e: SubtitleEntry,
) -> Map<std::vec::IntoIter<String>, impl FnMut(String) -> SubtitleEntry> {
    let lines: Vec<String> = e.line.unwrap().split('\n').map(str::to_string).collect();
    lines.into_iter().map(move |line| SubtitleEntry {
        timespan: e.timespan,
        line: Some(line),
    })
}
