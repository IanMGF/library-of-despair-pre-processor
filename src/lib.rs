use log4rs::encode::pattern::PatternEncoder;

pub mod flatten_subtitles;
pub mod parse_cast;

pub fn setup_logging() {
    use log::LevelFilter;
    use log4rs::{
        Config,
        append::console::ConsoleAppender,
        config::{Appender, Root},
    };

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{h([{d(%H:%M:%S)} - {l}])} {m} {n}",
        )))
        .build();
    let log_config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .build(Root::builder().appender("stdout").build(LevelFilter::Info))
        .unwrap();
    let _ = log4rs::init_config(log_config);
}
