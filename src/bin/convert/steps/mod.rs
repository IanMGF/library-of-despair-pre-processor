use subparse::SubtitleEntry;

pub trait PreProcessingStep<T, U> {
    fn apply(input: T, ctx: &PreProcessingCtx) -> U;
}

#[derive(Clone, Copy)]
pub struct PreProcessingCtx<'a> {
    pub subtitle_entry: &'a SubtitleEntry,
    pub line_number: usize,
}

pub mod enclosers;
pub mod find_cast;
pub mod parse;
