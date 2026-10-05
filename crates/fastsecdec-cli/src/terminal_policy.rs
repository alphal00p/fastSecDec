//! One color decision shared by terminal reports, errors and the dashboard.
use ratatui::style::{Color, Style};

#[derive(Clone, Copy)]
pub struct ColorPolicy {
    enabled: bool,
}

impl ColorPolicy {
    pub fn for_stream(plain: bool, is_terminal: bool) -> Self {
        Self {
            enabled: !plain && is_terminal && std::env::var_os("NO_COLOR").is_none(),
        }
    }

    pub fn enabled(self) -> bool {
        self.enabled
    }

    pub fn foreground(self, color: Color) -> Style {
        if self.enabled {
            Style::default().fg(color)
        } else {
            Style::default()
        }
    }
}
