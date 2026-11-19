use prismforge_core::store::CloseAction;

/// What a request to close the window does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CloseStep {
    Hide,
    Quit,
    /// The user has not chosen yet; the window asks before anything happens.
    Ask,
}

/// Without a tray a hidden window could not come back, so closing always quits.
pub fn decide_close(on_close: Option<CloseAction>, tray: bool) -> CloseStep {
    if !tray {
        return CloseStep::Quit;
    }
    match on_close {
        Some(CloseAction::Background) => CloseStep::Hide,
        Some(CloseAction::Quit) => CloseStep::Quit,
        None => CloseStep::Ask,
    }
}

/// The first-close question: an answer counts only while the question is open.
#[derive(Debug, Default)]
pub struct ClosePrompt {
    asking: bool,
}

impl ClosePrompt {
    pub fn asking(&self) -> bool {
        self.asking
    }

    pub fn requested(&mut self, step: CloseStep) -> CloseStep {
        self.asking = step == CloseStep::Ask;
        step
    }

    /// `None` cancels; the window stays open and nothing is chosen.
    pub fn answered(&mut self, choice: Option<CloseAction>) -> Option<CloseStep> {
        if !std::mem::take(&mut self.asking) {
            return None;
        }
        match choice? {
            CloseAction::Background => Some(CloseStep::Hide),
            CloseAction::Quit => Some(CloseStep::Quit),
        }
    }
}
