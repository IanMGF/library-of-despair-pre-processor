//! Conversion code to turn original `.srt` files into a (`cast.yaml`,  `assignments.yaml`, `content.txt`) triple
pub mod steps;

use common_types::archive::assignments::AssignmentUnit;
use common_types::archive::cast::{Cast, CastMember};
use pre_processor::flatten_subtitles::flatten_subtitles;
use pre_processor::setup_logging;
use srtlib::Subtitles;
use std::ffi::OsStr;
use std::io::Write;
use std::sync::Arc;
use std::{env, fs::File, io::BufWriter, path::PathBuf};

use crate::steps::enclosers::{Encloser, RemoveEnclosers, RemovedEncloserResult};
use crate::steps::find_cast::FindCastMember;
use crate::steps::parse::Parser;
use crate::steps::{PreProcessingCtx, PreProcessingStep};

fn main() {
    let args: Vec<PathBuf> = env::args().skip(1).map(PathBuf::from).collect();
    setup_logging();

    let base_filepath = args.first().unwrap();

    let srt_filepath = base_filepath.join("subtitles.srt");
    let cast_filepath = base_filepath.join("cast.yaml");
    let content_filepath = base_filepath.join("content.txt");
    let assignment_filepath = base_filepath.join("assignment.csv");

    // Warn if wrong extensions are detected
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
    let subtitles: Subtitles = Subtitles::parse_from_file(&srt_filepath, Some("utf8")).unwrap();

    // Load YAML Cast file
    let cast: Cast = yaml_serde::from_reader(File::open(cast_filepath).unwrap()).unwrap();

    let mut content: String = String::new();
    let mut assignments_vec: Vec<AssignmentUnit> = vec![];

    // Store member who was previously speaking,
    // so that it can recognize
    // "Member: Line 1 \n Line2 \n Line3"
    let mut prev_speaker: Option<Arc<CastMember>> = None;

    let line_entry_iter = subtitles
        .into_iter()
        .flat_map(flatten_subtitles)
        .enumerate();

    for (i, entry) in line_entry_iter {
        let encloser_opt: Option<Encloser>;
        let line_rc: Arc<str>;
        let og_line_rc: Arc<str> = entry.text.clone().into();
        let ctx: PreProcessingCtx = PreProcessingCtx {
            subtitle: &entry,
            line_number: i,
        };

        // Store original line, for later retrieval in case the processing goes wrong
        let original_line = entry.text.clone();

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
            (Some(s), _) => vec![s],
            (None, Some(s)) => s.aliases.clone(),
            (None, None) => {
                log::warn!("Speaker undetected for line {}", i + 1);
                content.push_str(line_rc.trim());
                content.push('\n');

                let timestamp = {
                    let t = entry.start_time.get();
                    t.0 as i64 * 3600 + t.1 as i64 * 60 + t.2 as i64
                };
                let assignment_unit = AssignmentUnit {
                    time: timestamp,
                    assignments: Arc::new([]),
                };
                assignments_vec.push(assignment_unit);
                continue;
            }
        };

        // Find the speaker
        let speaker = FindCastMember::apply((curr_speaker_tag[0].clone(), &cast), &ctx);

        // Guarantee the alias given was found in the cast file
        let Some(speaker) = speaker else {
            log::warn!(
                "Aliases not found:\tTimestamp:{:>16}  Line {:>5}  Aliases: {:?}",
                entry.start_time,
                i + 1,
                curr_speaker_tag
            );
            content.push_str(original_line.trim());
            content.push('\n');
            let timestamp = {
                let t = entry.start_time.get();
                t.0 as i64 * 3600 + t.1 as i64 * 60 + t.2 as i64
            };
            let assignment_unit = AssignmentUnit {
                time: timestamp,
                assignments: Arc::new([]),
            };
            assignments_vec.push(assignment_unit);
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
        let mut set = Vec::new();
        let timestamp = {
            let t = entry.start_time.get();
            t.0 as i64 * 3600 + t.1 as i64 * 60 + t.2 as i64
        };
        set.push(speaker.id.clone());
        let assignment_unit = AssignmentUnit {
            time: timestamp,
            assignments: set.into(),
        };
        assignments_vec.push(assignment_unit);
    }

    let content_file: File = File::create(content_filepath).unwrap();
    let mut content_writer: BufWriter<File> = BufWriter::new(content_file);
    write!(content_writer, "{}", content).unwrap();

    let assignment_file: File = File::create(assignment_filepath).unwrap();
    let mut csv_writer = csv::WriterBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_writer(assignment_file);

    for line_assignment in assignments_vec {
        csv_writer.serialize(line_assignment).unwrap();
    }
}
