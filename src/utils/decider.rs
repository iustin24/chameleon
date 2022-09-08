use crate::Args;
use feroxfuzz::actions::Action;
use feroxfuzz::deciders::{Decider, DeciderHooks};
use feroxfuzz::observers::{Observers, ResponseObserver};
use feroxfuzz::responses::Response;
use feroxfuzz::state::SharedState;

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
