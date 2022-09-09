use crate::Args;
use feroxfuzz::actions::Action;
use feroxfuzz::deciders::{Decider, DeciderHooks};
use feroxfuzz::observers::{Observers, ResponseObserver};
use feroxfuzz::responses::Response;
use feroxfuzz::state::SharedState;
use feroxfuzz::Metadata;
use std::any::Any;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub(crate) struct MetadataStruct {
    pub(crate) length: usize,
    pub(crate) words: usize,
    pub(crate) lines: usize,
}

#[typetag::serde(name = "metadata")]
impl Metadata for MetadataStruct {
    fn is_equal(&self, _other: &dyn Any) -> bool {
        false
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FilterDecider<'a, F>
where
    F: Fn(&'a Args, u16, usize, &SharedState) -> Action,
{
    comparator: F,
    args: &'a Args,
}

impl<'a, F> FilterDecider<'a, F>
where
    F: Fn(&Args, u16, usize, &SharedState) -> Action,
{
    pub const fn new(args: &'a Args, comparator: F) -> Self {
        Self { comparator, args }
    }
}

impl<'a, O, R, F> DeciderHooks<O, R> for FilterDecider<'a, F>
where
    O: Observers<R>,
    R: Response,
    F: Fn(&'a Args, u16, usize, &SharedState) -> Action,
{
}

impl<'a, O, R, F> Decider<O, R> for FilterDecider<'a, F>
where
    O: Observers<R>,
    R: Response,
    F: Fn(&'a Args, u16, usize, &SharedState) -> Action,
{
    fn decide_with_observers(&mut self, state: &SharedState, observers: &O) -> Option<Action> {
        if let Some(observer) = observers.match_name::<ResponseObserver<R>>("ResponseObserver") {
            let observed_response = observer;
            return Some((self.comparator)(
                &self.args,
                observed_response.status_code(),
                observed_response.content_length(),
                state,
            ));
        }
        None
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub(crate) struct CalibrateDecider<'a, F>
where
    F: Fn(&'a Vec<MetadataStruct>, usize, usize, usize, &SharedState) -> Action,
{
    comparator: F,
    metadata: &'a Vec<MetadataStruct>,
}

impl<'a, F> CalibrateDecider<'a, F>
where
    F: Fn(&Vec<MetadataStruct>, usize, usize, usize, &SharedState) -> Action,
{
    pub const fn new(metadata: &'a Vec<MetadataStruct>, comparator: F) -> Self {
        Self {
            comparator,
            metadata,
        }
    }
}

impl<'a, O, R, F> DeciderHooks<O, R> for CalibrateDecider<'a, F>
where
    O: Observers<R>,
    R: Response,
    F: Fn(&'a Vec<MetadataStruct>, usize, usize, usize, &SharedState) -> Action,
{
}

impl<'a, O, R, F> Decider<O, R> for CalibrateDecider<'a, F>
where
    O: Observers<R>,
    R: Response,
    F: Fn(&'a Vec<MetadataStruct>, usize, usize, usize, &SharedState) -> Action,
{
    fn decide_with_observers(&mut self, state: &SharedState, observers: &O) -> Option<Action> {
        if let Some(observer) = observers.match_name::<ResponseObserver<R>>("ResponseObserver") {
            let observed_response = observer;
            return Some((self.comparator)(
                &self.metadata,
                observed_response.content_length(),
                observed_response.word_count(),
                observed_response.line_count(),
                state,
            ));
        }
        None
    }
}
