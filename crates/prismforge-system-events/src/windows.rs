#![allow(unsafe_code)]

use std::sync::mpsc::{SyncSender, sync_channel};
use std::sync::{Arc, OnceLock};
use std::thread;
use std::time::Duration;

use windows::Win32::Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Power::RegisterSuspendResumeNotification;
use windows::Win32::System::Threading::SetProcessShutdownParameters;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DEVICE_NOTIFY_WINDOW_HANDLE, DefWindowProcW, DispatchMessageW, GetMessageW,
    MSG, PBT_APMRESUMEAUTOMATIC, PBT_APMSUSPEND, RegisterClassW, TranslateMessage, WM_ENDSESSION,
    WM_POWERBROADCAST, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_POPUP,
};
use windows::core::{PCWSTR, w};

use super::{EventSink, EventSource, EventSupport, SESSION_END_LIMIT, SLEEP_LIMIT, SystemEvent};

/// In the "application first" range, so Windows closes PrismForge before an OpenRGB at the default 0x280.
const SHUTDOWN_LEVEL: u32 = 0x300;
/// How long the app waits for the event window to come up at start.
const START_LIMIT: Duration = Duration::from_secs(2);
const CLASS: PCWSTR = w!("PrismForgeSystemEvents");

static SINK: OnceLock<Arc<dyn EventSink>> = OnceLock::new();

/// What the event window reports once it is up: `None`, or a part that failed and what it costs.
type Started = Result<Option<String>, String>;

/// A hidden window on its own thread with its own message loop; it never shows and never blocks a shutdown.
pub struct WindowsSource;

impl EventSource for WindowsSource {
    fn start(self: Box<Self>, sink: Arc<dyn EventSink>) -> EventSupport {
        if SINK.set(sink).is_err() {
            return EventSupport::Failed {
                detail: "the event window was already started".into(),
            };
        }
        let level = unsafe { SetProcessShutdownParameters(SHUTDOWN_LEVEL, 0) }.err();
        let (report, started) = sync_channel(1);
        let spawned = thread::Builder::new()
            .name("system-events".into())
            .spawn(move || pump(&report));
        if let Err(error) = spawned {
            return EventSupport::Failed {
                detail: format!("the event thread could not start: {error}"),
            };
        }
        let window = started
            .recv_timeout(START_LIMIT)
            .unwrap_or_else(|_| Err("the event window did not start in time".into()));
        let level = level.map(|error| {
            format!("Windows may close OpenRGB before PrismForge at shutdown: {error}")
        });
        match window {
            Err(detail) => EventSupport::Failed { detail },
            Ok(limited) => match limited
                .into_iter()
                .chain(level)
                .reduce(|a, b| format!("{a} {b}"))
            {
                Some(detail) => EventSupport::Limited { detail },
                None => EventSupport::Available,
            },
        }
    }
}

fn pump(report: &SyncSender<Started>) {
    let window = match create_window() {
        Ok(window) => window,
        Err(detail) => {
            let _ = report.send(Err(detail));
            return;
        }
    };
    let registered =
        unsafe { RegisterSuspendResumeNotification(HANDLE(window.0), DEVICE_NOTIFY_WINDOW_HANDLE) };
    let limited = registered
        .err()
        .map(|error| format!("Sleep may not be reported on PCs with Modern Standby: {error}"));
    let _ = report.send(Ok(limited));
    let mut message = MSG::default();
    while unsafe { GetMessageW(&mut message, None, 0, 0) }.0 > 0 {
        unsafe {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

fn create_window() -> Result<HWND, String> {
    unsafe {
        let module =
            GetModuleHandleW(None).map_err(|error| format!("no module handle: {error}"))?;
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: module.into(),
            lpszClassName: CLASS,
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            return Err(format!(
                "the window class could not be registered: {}",
                windows::core::Error::from_thread()
            ));
        }
        CreateWindowExW(
            WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
            CLASS,
            w!("PrismForge system events"),
            WS_POPUP,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(module.into()),
            None,
        )
        .map_err(|error| format!("the event window could not be created: {error}"))
    }
}

unsafe extern "system" fn window_proc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match (message, wparam.0 as u32) {
        (WM_POWERBROADCAST, PBT_APMSUSPEND) => {
            deliver(SystemEvent::Sleep, |done| {
                done.wait(SLEEP_LIMIT);
            });
            LRESULT(1)
        }
        (WM_POWERBROADCAST, PBT_APMRESUMEAUTOMATIC) => {
            deliver(SystemEvent::Wake, drop);
            LRESULT(1)
        }
        (WM_ENDSESSION, ending) if ending != 0 => {
            deliver(SystemEvent::SessionEnd, |done| {
                done.wait(SESSION_END_LIMIT);
            });
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window, message, wparam, lparam) },
    }
}

fn deliver(event: SystemEvent, then: impl FnOnce(super::Done)) {
    if let Some(sink) = SINK.get() {
        then(sink.event(event));
    }
}
