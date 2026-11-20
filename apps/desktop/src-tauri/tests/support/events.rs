use std::sync::{Arc, Mutex};

use prismforge_desktop::{Done, EventSink, EventSource, EventSupport, SystemEvent};

/// Stands in for the Windows window: the test fires the events it would deliver.
#[derive(Clone, Default)]
pub struct FakeEvents(Arc<Mutex<Option<Arc<dyn EventSink>>>>);

pub struct FakeSource(FakeEvents);

impl FakeEvents {
    pub fn source(&self) -> Box<dyn EventSource> {
        Box::new(FakeSource(self.clone()))
    }

    pub fn fire(&self, event: SystemEvent) -> Done {
        let sink = self
            .0
            .lock()
            .expect("sink")
            .clone()
            .expect("source started");
        sink.event(event)
    }
}

impl EventSource for FakeSource {
    fn support(&self) -> EventSupport {
        EventSupport::Available
    }

    fn start(self: Box<Self>, sink: Arc<dyn EventSink>) {
        *self.0.0.lock().expect("sink") = Some(sink);
    }
}
