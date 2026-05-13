//! Logbook switcher strip — thin row above the entry form and the log table
//! that shows the active logbook and lets the operator switch to a different
//! one. Same strip is reused by both the main window and the log window so
//! changing the active log in either place updates the other.

use iced::widget::{container, row, text};
use iced::{Alignment, Element, Length};

use crate::app::{App, Message};
use crate::t;
use crate::ui::inputs::dropdown;

const LABEL_SIZE: f32 = 12.0;
const DROPDOWN_WIDTH: f32 = 220.0;

pub fn view(state: &App) -> Element<'_, Message> {
    let label = text(t!("log-switcher-label")).size(LABEL_SIZE);
    let selected = state.active_log().cloned();
    let picker = dropdown(
        &state.logs,
        selected,
        Message::LogSwitched,
        Length::Fixed(DROPDOWN_WIDTH),
    );

    container(
        row![label, picker]
            .spacing(8)
            .align_y(Alignment::Center),
    )
    .padding([6, 12])
    .width(Length::Fill)
    .into()
}
