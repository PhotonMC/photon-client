use crate::Owner;

type Handler<A> = Box<dyn FnMut(&A) + Send>;

/// An event with the handlers listening to it, run in the order they were registered.
pub struct Event<A> {
    handlers: Vec<(Owner, Handler<A>)>,
}

impl<A> Default for Event<A> {
    fn default() -> Self {
        Self {
            handlers: Vec::new(),
        }
    }
}

impl<A> Event<A> {
    /// Listen to the event as `owner`; the handler runs after those registered before it.
    pub fn register(&mut self, owner: Owner, handler: impl FnMut(&A) + Send + 'static) {
        self.handlers.push((owner, Box::new(handler)));
    }

    /// Run every handler on `argument`, in registration order.
    pub fn fire(&mut self, argument: &A) {
        for (_, handler) in &mut self.handlers {
            handler(argument);
        }
    }

    /// Remove every handler `owner` registered; how many there were.
    pub fn remove_owner(&mut self, owner: Owner) -> usize {
        let before = self.handlers.len();
        self.handlers
            .retain(|(handler_owner, _)| *handler_owner != owner);
        before - self.handlers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.handlers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    const FIRST: Owner = Owner::new("first");
    const SECOND: Owner = Owner::new("second");

    fn recording(
        log: &Arc<Mutex<Vec<String>>>,
        name: &'static str,
    ) -> impl FnMut(&u32) + Send + 'static {
        let log = Arc::clone(log);
        move |value| log.lock().unwrap().push(format!("{name}{value}"))
    }

    #[test]
    fn handlers_run_in_the_order_they_were_registered() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut event = Event::default();
        event.register(FIRST, recording(&log, "a"));
        event.register(SECOND, recording(&log, "b"));
        event.register(FIRST, recording(&log, "c"));
        event.fire(&1);
        assert_eq!(*log.lock().unwrap(), ["a1", "b1", "c1"]);
    }

    #[test]
    fn removing_an_owner_takes_all_of_its_handlers_and_only_those() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let mut event = Event::default();
        event.register(FIRST, recording(&log, "a"));
        event.register(SECOND, recording(&log, "b"));
        event.register(FIRST, recording(&log, "c"));
        assert_eq!(event.remove_owner(FIRST), 2);
        event.fire(&2);
        assert_eq!(*log.lock().unwrap(), ["b2"]);
        assert_eq!(event.remove_owner(FIRST), 0);
        assert_eq!(event.remove_owner(SECOND), 1);
        assert!(event.is_empty());
    }
}
