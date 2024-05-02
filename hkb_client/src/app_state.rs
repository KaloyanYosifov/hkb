use std::fmt::Display;

use parking_lot::{MappedMutexGuard, Mutex, MutexGuard};

use crate::terminal;

static GLOBAL_APP_STATE: Mutex<Option<AppState>> = parking_lot::const_mutex(None);

#[derive(Debug, Clone, Copy)]
pub enum AppView {
    Main,
    Reminders,
}

impl Display for AppView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::Main => "Main",
            Self::Reminders => "Reminders",
        };

        write!(f, "{}", text)
    }
}

pub struct AppState {
    view: AppView,
    editing: bool,
    ignore_navigation_events: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            editing: false,
            view: AppView::Main,
            ignore_navigation_events: false,
        }
    }
}

impl AppState {
    fn get_global() -> MappedMutexGuard<'static, Self> {
        MutexGuard::map(GLOBAL_APP_STATE.lock(), |reader| {
            reader.get_or_insert_with(Self::default)
        })
    }
}

pub fn set_view(view: AppView) {
    AppState::get_global().view = view;
}

pub fn get_view() -> AppView {
    AppState::get_global().view
}

pub fn set_editing(editing: bool) {
    if editing {
        terminal::set_cursor_steady_bar();
    } else {
        terminal::set_cursor_to_default();
    }

    AppState::get_global().editing = editing;
}

pub fn is_editing() -> bool {
    AppState::get_global().editing
}

pub fn disable_navigation_events() {
    AppState::get_global().ignore_navigation_events = true;
}

pub fn enable_navigation_events() {
    AppState::get_global().ignore_navigation_events = false;
}

pub fn should_ignore_navigation_events() -> bool {
    AppState::get_global().ignore_navigation_events
}
