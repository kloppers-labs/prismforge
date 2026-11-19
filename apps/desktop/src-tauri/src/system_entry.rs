use crate::lifecycle::{AUTOSTART_ARG, StartEntry};

const ENTRY_NAME: &str = "PrismForge";

/// The command the Windows `Run` value holds: the quoted program path, then the start argument.
pub fn run_command(exe: &str) -> String {
    format!("\"{exe}\" {AUTOSTART_ARG}")
}

/// Task Manager marks a turned-off start entry with an odd first byte in `StartupApproved\Run`.
pub fn startup_disabled(approved: &[u8]) -> bool {
    approved.first().is_some_and(|flag| flag & 1 == 1)
}

/// The start entry for the signed-in user only, also when PrismForge runs as Administrator.
pub struct SystemStartEntry;

impl StartEntry for SystemStartEntry {
    fn is_registered(&self) -> Result<bool, String> {
        platform::is_registered()
    }

    fn register(&self) -> Result<(), String> {
        platform::register()
    }

    fn unregister(&self) -> Result<(), String> {
        platform::unregister()
    }

    fn machine_wide(&self) -> bool {
        platform::machine_wide()
    }
}

fn program() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|error| error.to_string())?;
    Ok(exe.display().to_string())
}

#[cfg(windows)]
mod platform {
    use windows_registry::{CURRENT_USER, Key, LOCAL_MACHINE, Type};

    use super::{ENTRY_NAME, program, run_command, startup_disabled};

    const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
    const APPROVED: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";
    const APPROVED_ON: [u8; 12] = [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    const NOT_FOUND: i32 = 0x8007_0002_u32 as i32;

    pub fn is_registered() -> Result<bool, String> {
        if !value_exists(CURRENT_USER, RUN)? {
            return Ok(false);
        }
        match CURRENT_USER
            .open(APPROVED)
            .and_then(|key| key.get_value(ENTRY_NAME))
        {
            Ok(approved) => Ok(!startup_disabled(&approved)),
            Err(error) if error.code().0 == NOT_FOUND => Ok(true),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn register() -> Result<(), String> {
        let command = run_command(&program()?);
        CURRENT_USER
            .create(RUN)
            .and_then(|key| key.set_string(ENTRY_NAME, &command))
            .map_err(|error| error.to_string())?;
        match CURRENT_USER.options().write().open(APPROVED) {
            Ok(key) => key
                .set_bytes(ENTRY_NAME, Type::Bytes, &APPROVED_ON)
                .map_err(|error| error.to_string()),
            Err(error) if error.code().0 == NOT_FOUND => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }

    pub fn unregister() -> Result<(), String> {
        remove(RUN)?;
        remove(APPROVED)
    }

    pub fn machine_wide() -> bool {
        value_exists(LOCAL_MACHINE, RUN).unwrap_or(false)
    }

    fn value_exists(root: &Key, path: &str) -> Result<bool, String> {
        match root.open(path).and_then(|key| key.get_value(ENTRY_NAME)) {
            Ok(_) => Ok(true),
            Err(error) if error.code().0 == NOT_FOUND => Ok(false),
            Err(error) => Err(error.to_string()),
        }
    }

    fn remove(path: &str) -> Result<(), String> {
        match CURRENT_USER
            .options()
            .write()
            .open(path)
            .and_then(|key| key.remove_value(ENTRY_NAME))
        {
            Err(error) if error.code().0 != NOT_FOUND => Err(error.to_string()),
            _ => Ok(()),
        }
    }
}

#[cfg(not(windows))]
mod platform {
    use auto_launch::{AutoLaunch, AutoLaunchBuilder};

    use super::{ENTRY_NAME, program};
    use crate::lifecycle::AUTOSTART_ARG;

    fn entry() -> Result<AutoLaunch, String> {
        let path = std::env::var("APPIMAGE").or_else(|_| program())?;
        #[cfg(target_os = "linux")]
        let path = format!("\"{path}\"");
        let mut builder = AutoLaunchBuilder::new();
        builder
            .set_app_name(ENTRY_NAME)
            .set_app_path(&path)
            .set_args(&[AUTOSTART_ARG]);
        #[cfg(target_os = "macos")]
        builder.set_macos_launch_mode(auto_launch::MacOSLaunchMode::LaunchAgent);
        builder.build().map_err(|error| error.to_string())
    }

    pub fn is_registered() -> Result<bool, String> {
        entry()?.is_enabled().map_err(|error| error.to_string())
    }

    pub fn register() -> Result<(), String> {
        entry()?.enable().map_err(|error| error.to_string())
    }

    pub fn unregister() -> Result<(), String> {
        let entry = entry()?;
        if !entry.is_enabled().map_err(|error| error.to_string())? {
            return Ok(());
        }
        entry.disable().map_err(|error| error.to_string())
    }

    pub fn machine_wide() -> bool {
        false
    }
}
