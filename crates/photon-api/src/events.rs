use crate::Event;

/// A client tick is about to run (Fabric's `ClientTickEvents.START_CLIENT_TICK`).
#[derive(Debug, Clone, Copy)]
pub struct TickStart;

/// A client tick has run (`ClientTickEvents.END_CLIENT_TICK`).
#[derive(Debug, Clone, Copy)]
pub struct TickEnd;

/// The events the client fires, one field each. The play loop owns the one value.
#[derive(Default)]
pub struct ClientEvents {
    pub tick_start: Event<TickStart>,
    pub tick_end: Event<TickEnd>,
}
