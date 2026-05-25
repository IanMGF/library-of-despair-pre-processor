//! Step to remove enclosing characters,
//! such as parethenses, brackets, etc.

use std::rc::Rc;

use crate::steps::PreProcessingStep;

pub struct RemoveEnclosers;

pub enum Encloser {
    Parenthesis,
}

impl Encloser {
    pub fn first_char(&self) -> char {
        match self {
            Encloser::Parenthesis => '(',
        }
    }

    pub fn last_char(&self) -> char {
        match self {
            Encloser::Parenthesis => ')',
        }
    }
}

pub struct RemovedEncloserResult {
    pub encloser: Option<Encloser>,
    pub text: Rc<str>,
}

impl PreProcessingStep<Rc<str>, RemovedEncloserResult> for RemoveEnclosers {
    fn apply(input: Rc<str>, _ctx: &super::PreProcessingCtx) -> RemovedEncloserResult {
        if input.starts_with('(') && input.ends_with(')') {
            RemovedEncloserResult {
                encloser: Some(Encloser::Parenthesis),
                text: input[1..input.len() - 1].into(),
            }
        } else {
            RemovedEncloserResult {
                encloser: None,
                text: input,
            }
        }
    }
}
