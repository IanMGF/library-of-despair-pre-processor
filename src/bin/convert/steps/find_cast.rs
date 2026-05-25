use std::rc::Rc;

use backend::cast::{Cast, CastMember};

use crate::steps::PreProcessingStep;

pub struct FindCastMemberResult {
    pub text: Rc<str>,
    pub cast_member: Option<Rc<CastMember>>,
}

pub struct FindCastMember;
impl PreProcessingStep<(Rc<str>, &Cast), Option<Rc<CastMember>>> for FindCastMember {
    fn apply(
        (line, Cast(cast_set)): (Rc<str>, &Cast),
        _ctx: &super::PreProcessingCtx,
    ) -> Option<Rc<CastMember>> {
        let speaker: Option<Rc<backend::cast::CastMember>> = cast_set
            .iter()
            .filter(|&member| member.aliases.contains(&line.to_string()))
            .next()
            .map(|member_ref| Rc::new(member_ref.clone()));

        speaker
    }
}
