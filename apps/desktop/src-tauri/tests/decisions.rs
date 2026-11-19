mod support;

use prismforge_core::store::{AppSettings, CloseAction, OpenRgbAddress};
use prismforge_desktop::{
    AUTOSTART_ARG, ClosePrompt, CloseStep, SettingProblem, StartEntryState, StartupPlan,
    decide_close, launched_by_system, read_entry, run_command, startup_disabled, startup_plan,
    switch_entry,
};

use support::lifecycle::FakeEntry;

fn address() -> OpenRgbAddress {
    OpenRgbAddress {
        host: "127.0.0.1".into(),
        port: 6742,
    }
}

#[test]
fn closing_quits_without_a_tray_whatever_the_user_chose() {
    for choice in [None, Some(CloseAction::Background), Some(CloseAction::Quit)] {
        assert_eq!(decide_close(choice, false), CloseStep::Quit, "{choice:?}");
    }
}

#[test]
fn with_a_tray_closing_asks_until_the_user_chooses() {
    assert_eq!(decide_close(None, true), CloseStep::Ask);
    assert_eq!(
        decide_close(Some(CloseAction::Background), true),
        CloseStep::Hide
    );
    assert_eq!(decide_close(Some(CloseAction::Quit), true), CloseStep::Quit);
}

#[test]
fn the_first_close_prompt_performs_only_an_answer_it_asked_for() {
    let mut prompt = ClosePrompt::default();

    assert_eq!(prompt.answered(Some(CloseAction::Quit)), None);
    assert!(!prompt.asking());
    assert_eq!(prompt.requested(CloseStep::Ask), CloseStep::Ask);
    assert!(prompt.asking());
    assert_eq!(prompt.requested(CloseStep::Ask), CloseStep::Ask);
    assert_eq!(
        prompt.answered(Some(CloseAction::Background)),
        Some(CloseStep::Hide)
    );
    assert!(!prompt.asking());
    assert_eq!(prompt.answered(Some(CloseAction::Background)), None);

    prompt.requested(CloseStep::Ask);
    assert_eq!(prompt.answered(None), None);
    assert!(!prompt.asking());

    prompt.requested(CloseStep::Ask);
    assert_eq!(
        prompt.answered(Some(CloseAction::Quit)),
        Some(CloseStep::Quit)
    );
}

#[test]
fn a_decided_close_clears_an_open_prompt() {
    let mut prompt = ClosePrompt::default();
    prompt.requested(CloseStep::Ask);

    assert_eq!(prompt.requested(CloseStep::Quit), CloseStep::Quit);

    assert!(!prompt.asking());
}

#[test]
fn start_up_shows_the_window_and_connects_nothing_by_default() {
    let plan = startup_plan(Some(&AppSettings::default()), true, true);

    assert_eq!(
        plan,
        StartupPlan {
            show_window: true,
            connect: None
        }
    );
    assert_eq!(
        startup_plan(None, true, true),
        StartupPlan {
            show_window: true,
            connect: None
        }
    );
}

#[test]
fn start_up_hides_the_window_only_when_the_system_started_it_with_a_tray() {
    let hidden = AppSettings {
        start_hidden: true,
        ..AppSettings::default()
    };

    assert!(!startup_plan(Some(&hidden), true, true).show_window);
    assert!(startup_plan(Some(&hidden), false, true).show_window);
    assert!(startup_plan(Some(&hidden), true, false).show_window);
}

#[test]
fn start_up_connects_only_when_chosen_and_to_the_last_used_address() {
    let chosen = AppSettings {
        connect_at_start: true,
        last_address: Some(address()),
        ..AppSettings::default()
    };
    let no_address = AppSettings {
        connect_at_start: true,
        ..AppSettings::default()
    };
    let not_chosen = AppSettings {
        last_address: Some(address()),
        ..AppSettings::default()
    };

    assert_eq!(
        startup_plan(Some(&chosen), false, false).connect,
        Some(address())
    );
    assert_eq!(startup_plan(Some(&no_address), false, false).connect, None);
    assert_eq!(startup_plan(Some(&not_chosen), false, false).connect, None);
}

#[test]
fn only_the_start_entry_argument_marks_a_start_by_the_system() {
    let by_system = ["prismforge".to_owned(), AUTOSTART_ARG.to_owned()];
    let by_user = ["prismforge".to_owned(), "--autostarted".to_owned()];

    assert!(launched_by_system(by_system));
    assert!(!launched_by_system(by_user));
}

#[test]
fn the_start_entry_switch_reports_what_the_system_reads_back() {
    let entry = FakeEntry::default();

    let on = switch_entry(&entry, true);
    let off = switch_entry(&entry, false);

    assert_eq!(on, Ok(StartEntryState::Registered));
    assert_eq!(off, Ok(StartEntryState::NotRegistered));
    assert!(!entry.is_on());
}

#[test]
fn a_refused_registration_says_so_and_stays_off() {
    let entry = FakeEntry::default();
    *entry.refuse.lock().expect("entry") = Some("access denied".into());

    let on = switch_entry(&entry, true);

    assert_eq!(
        on,
        Err(SettingProblem::StartEntry {
            wanted: true,
            detail: Some("access denied".into())
        })
    );
    assert_eq!(read_entry(&entry), StartEntryState::NotRegistered);
}

#[test]
fn a_change_the_system_did_not_keep_is_reported() {
    let entry = FakeEntry::default();
    entry
        .ignore
        .store(true, std::sync::atomic::Ordering::SeqCst);

    let on = switch_entry(&entry, true);

    assert_eq!(
        on,
        Err(SettingProblem::StartEntry {
            wanted: true,
            detail: None
        })
    );
}

#[test]
fn an_unreadable_entry_is_unknown_not_off() {
    let entry = FakeEntry::default();
    *entry.unreadable.lock().expect("entry") = Some("no home folder".into());

    assert_eq!(
        read_entry(&entry),
        StartEntryState::Unknown {
            detail: "no home folder".into()
        }
    );
    assert_eq!(
        switch_entry(&entry, true),
        Err(SettingProblem::StartEntry {
            wanted: true,
            detail: Some("no home folder".into())
        })
    );
}

/// Splits a command line the way Windows does for these inputs: quotes group, spaces separate.
fn arguments(command: &str) -> Vec<String> {
    let mut arguments = Vec::new();
    let mut current = String::new();
    let mut quoted = false;
    for character in command.chars() {
        match character {
            '"' => quoted = !quoted,
            ' ' if !quoted => {
                if !current.is_empty() {
                    arguments.push(std::mem::take(&mut current));
                }
            }
            _ => current.push(character),
        }
    }
    if !current.is_empty() {
        arguments.push(current);
    }
    arguments
}

#[test]
fn the_windows_start_command_quotes_paths_with_spaces_and_keeps_the_start_argument() {
    for exe in [
        r"C:\Program Files\PrismForge\PrismForge.exe",
        r"C:\Users\Ann Lee\AppData\Local\PrismForge\PrismForge.exe",
        r"D:\Tools\PrismForge.exe",
    ] {
        let command = run_command(exe);

        assert_eq!(command, format!("\"{exe}\" {AUTOSTART_ARG}"));
        assert_eq!(
            arguments(&command),
            [exe.to_owned(), AUTOSTART_ARG.to_owned()]
        );
    }
}

#[test]
fn an_entry_turned_off_in_task_manager_does_not_count_as_registered() {
    let enabled = [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
    let disabled = [3, 0, 0, 0, 0x10, 0x2f, 0x6a, 0x7d, 0x1c, 0x5b, 0xdb, 0x01];

    assert!(!startup_disabled(&enabled));
    assert!(startup_disabled(&disabled));
    assert!(startup_disabled(&[7, 0, 0, 0]));
    assert!(!startup_disabled(&[6, 0, 0, 0]));
    assert!(!startup_disabled(&[]));
}
