use std::sync::{Arc, Mutex};

use prismforge_desktop::{Done, EventSink, EventSource, EventSupport, SystemEvent};

/// Stands in for the Windows window: the test fires the events it would deliver.
#[derive(Clone, Default)]
pub struct FakeEvents(Arc<Mutex<Option<Arc<dyn EventSink>>>>, Option<String>);

pub struct FakeSource(FakeEvents);

impl FakeEvents {
    /// A source whose start fails with `detail`, as a window that could not be created.
    pub fn failing(detail: &str) -> Self {
        Self(Arc::default(), Some(detail.into()))
    }

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
    fn start(self: Box<Self>, sink: Arc<dyn EventSink>) -> EventSupport {
        if let Some(detail) = &self.0.1 {
            return EventSupport::Failed {
                detail: detail.clone(),
            };
        }
        *self.0.0.lock().expect("sink") = Some(sink);
        EventSupport::Available
    }
}
