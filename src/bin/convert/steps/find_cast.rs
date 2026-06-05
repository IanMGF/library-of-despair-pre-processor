use std::{rc::Rc, sync::Arc};

use backend::archive::cast::{Cast, CastMember};

use crate::steps::PreProcessingStep;

pub struct FindCastMemberResult {
    pub text: Rc<str>,
    pub cast_member: Option<Arc<CastMember>>,
}

pub struct FindCastMember;
impl PreProcessingStep<(Arc<str>, &Cast), Option<Arc<CastMember>>> for FindCastMember {
    fn apply(
        (line, Cast(cast_set)): (Arc<str>, &Cast),
        _ctx: &super::PreProcessingCtx,
    ) -> Option<Arc<CastMember>> {
        let speaker: Option<Arc<backend::archive::cast::CastMember>> = cast_set
            .iter()
            .filter(|&member| member.aliases.iter().any(|s| s.as_ref() == line.as_ref()))
            .next()
            .map(|member_ref| Arc::new(member_ref.clone()));

        speaker
    }
}
