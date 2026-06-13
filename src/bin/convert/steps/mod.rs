use srtlib::Subtitle;

pub trait PreProcessingStep<T, U> {
    fn apply(input: T, ctx: &PreProcessingCtx) -> U;
}

#[derive(Clone, Copy)]
pub struct PreProcessingCtx<'a> {
    pub subtitle: &'a Subtitle,
    pub line_number: usize,
}

pub mod enclosers;
pub mod find_cast;
pub mod parse;
