#![allow(unsafe_code)]

use std::sync::{Arc, OnceLock};
use std::thread;

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

use super::{EventSink, EventSource, EventSupport, SystemEvent};
use crate::automation::{SESSION_END_LIMIT, SLEEP_LIMIT};

/// In the "application first" range, so Windows closes PrismForge before an OpenRGB at the default 0x280.
const SHUTDOWN_LEVEL: u32 = 0x300;
const CLASS: PCWSTR = w!("PrismForgeSystemEvents");

static SINK: OnceLock<Arc<dyn EventSink>> = OnceLock::new();

/// A hidden window on its own thread with its own message loop; it never shows and never blocks a shutdown.
pub struct WindowsSource;

impl EventSource for WindowsSource {
    fn support(&self) -> EventSupport {
        EventSupport::Available
    }

    fn start(self: Box<Self>, sink: Arc<dyn EventSink>) {
        if SINK.set(sink).is_err() {
            return;
        }
        unsafe {
            let _ = SetProcessShutdownParameters(SHUTDOWN_LEVEL, 0);
        }
        let _ = thread::Builder::new()
            .name("system-events".into())
            .spawn(pump);
    }
}

fn pump() {
    unsafe {
        let Ok(module) = GetModuleHandleW(None) else {
            return;
        };
        let class = WNDCLASSW {
            lpfnWndProc: Some(window_proc),
            hInstance: module.into(),
            lpszClassName: CLASS,
            ..Default::default()
        };
        if RegisterClassW(&class) == 0 {
            return;
        }
        let Ok(window) = CreateWindowExW(
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
        ) else {
            return;
        };
        let _registration =
            RegisterSuspendResumeNotification(HANDLE(window.0), DEVICE_NOTIFY_WINDOW_HANDLE);
        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
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
