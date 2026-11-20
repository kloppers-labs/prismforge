use std::sync::Arc;

use super::{EventSink, EventSource, EventSupport};

/// macOS and Linux: PrismForge does not receive sleep, wake or session end there yet.
pub struct Unsupported;

impl EventSource for Unsupported {
    fn support(&self) -> EventSupport {
        EventSupport::NotOnThisSystem
    }

    fn start(self: Box<Self>, _sink: Arc<dyn EventSink>) {}
}
