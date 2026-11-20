//! The one seam for power and session events; each platform has a module, the rules live elsewhere.

#[cfg(not(windows))]
mod unsupported;
#[cfg(windows)]
mod windows;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};
use std::time::Duration;

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum SystemEvent {
    /// Sleep or hibernate; Windows gives one signal for both.
    Sleep,
    Wake,
    /// Sign-out, shutdown or restart.
    SessionEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum EventSupport {
    Available,
    /// This system's source does not report sleep, wake and session end yet.
    NotOnThisSystem,
}

/// Takes the events of a source; the source decides how long it waits for each [`Done`].
pub trait EventSink: Send + Sync + 'static {
    fn event(&self, event: SystemEvent) -> Done;
}

pub trait EventSource {
    fn support(&self) -> EventSupport;

    /// Starts delivering events to `sink` until the process ends.
    fn start(self: Box<Self>, sink: Arc<dyn EventSink>);
}

/// Finishes when the work for one event finished or gave up at its own limit.
pub struct Done(Receiver<()>);

impl Done {
    pub fn pair() -> (SyncSender<()>, Self) {
        let (sender, receiver) = sync_channel(1);
        (sender, Self(receiver))
    }

    /// Blocks the event thread for at most `limit`; true when the work finished in time.
    pub fn wait(self, limit: Duration) -> bool {
        self.0.recv_timeout(limit).is_ok()
    }
}

type Work = Pin<Box<dyn Future<Output = ()> + Send>>;

/// Runs each event's work on the async runtime, so the event thread only waits for it.
pub struct Dispatcher {
    runtime: tokio::runtime::Handle,
    work: Box<dyn Fn(SystemEvent) -> Work + Send + Sync>,
}

impl Dispatcher {
    pub fn new(
        runtime: tokio::runtime::Handle,
        work: impl Fn(SystemEvent) -> Work + Send + Sync + 'static,
    ) -> Self {
        Self {
            runtime,
            work: Box::new(work),
        }
    }
}

impl EventSink for Dispatcher {
    fn event(&self, event: SystemEvent) -> Done {
        let (finished, done) = Done::pair();
        let work = (self.work)(event);
        self.runtime.spawn(async move {
            work.await;
            let _ = finished.send(());
        });
        done
    }
}

/// The source of this platform; only Windows reports events for now.
pub fn platform_source() -> Box<dyn EventSource> {
    #[cfg(windows)]
    return Box::new(windows::WindowsSource);
    #[cfg(not(windows))]
    Box::new(unsupported::Unsupported)
}
