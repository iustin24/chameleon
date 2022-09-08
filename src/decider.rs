use feroxfuzz::{Decider, DeciderHooks};
use feroxfuzz::actions::Action;
use feroxfuzz::observers::{Observers, ResponseObserver};
use feroxfuzz::responses::Response;
use feroxfuzz::state::SharedState;
use crate::Args;

pub struct StatusCodeDecider<F>
    where
        F: Fn(&Args, &ResponseObserver<R>, &SharedState) -> Action,
{
    comparator: F,
    args: &Args,
}

impl<F> StatusCodeDecider<F>
    where
        F: Fn(&Args, &ResponseObserver<R>, &SharedState) -> Action,
{
    /// create a new `StatusCodeDecider` that calls `comparator` in its
    /// `post_send_hook` method
    pub const fn new(args: &Args, comparator: F) -> Self {
        Self {
            comparator,
            args,
        }
    }
}

impl<O, R, F> DeciderHooks<O, R> for StatusCodeDecider<F>
    where
        O: Observers<R>,
        R: Response,
        F: Fn(&Args, &ResponseObserver<R>, &SharedState) -> Action,
{
}

impl<O, R, F> Decider<O, R> for StatusCodeDecider<F>
    where
        O: Observers<R>,
        R: Response,
        F: Fn(&Args, &ResponseObserver<R>, &SharedState) -> Action,

{
    fn decide_with_observers(&mut self, state: &SharedState, observers: &O) -> Option<Action> {
        // there's an implicit expectation that there is only a single ResponseObserver in the
        // list of given Observers
        if let Some(observer) = observers.match_name::<ResponseObserver<R>>("ResponseObserver") {
            // get the observed status code
            let observed_response = observer;

            // call the comparator to arrive at a decided action
            return Some((self.comparator)(self.status_code, observed_response, state));
        }

        None
    }
}
