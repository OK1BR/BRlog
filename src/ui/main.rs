use iced::widget::{column, container, horizontal_rule, row};
use iced::window;
use iced::{Alignment, Element, Length};

use crate::app::{App, DropdownKind, FONT_MONO, Message};
use crate::ui::inputs::{input, popup_trigger};
use crate::ui::title_bar;

pub fn view<'a>(state: &'a App, window_id: window::Id) -> Element<'a, Message> {
    container(
        column![
            title_bar::view(window_id, "BRlog", state.is_maximized(window_id), true),
            horizontal_rule(1).style(title_bar::rule_style),
            entry_row(state),
        ]
        .spacing(0),
    )
    .style(title_bar::window_border(state.config.appearance.window_border))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn entry_row(state: &App) -> Element<'_, Message> {
    container(
        row![
            input("Volačka", &state.entry.callsign)
                .on_input(Message::EntryCallsignChanged)
                .on_submit(Message::EntrySaveClicked)
                .font(FONT_MONO)
                .width(Length::Fixed(130.0)),
            popup_trigger(state.entry.band.to_string())
                .on_press(Message::DropdownTriggerClicked(DropdownKind::Band))
                .width(Length::Fixed(85.0)),
            popup_trigger(state.entry.mode.to_string())
                .on_press(Message::DropdownTriggerClicked(DropdownKind::Mode))
                .width(Length::Fixed(85.0)),
            input("RST↑", &state.entry.rst_sent)
                .on_input(Message::EntryRstSentChanged)
                .on_submit(Message::EntrySaveClicked)
                .font(FONT_MONO)
                .width(Length::Fixed(70.0)),
            input("RST↓", &state.entry.rst_rcvd)
                .on_input(Message::EntryRstRcvdChanged)
                .on_submit(Message::EntrySaveClicked)
                .font(FONT_MONO)
                .width(Length::Fixed(70.0)),
            input("Lokátor", &state.entry.locator)
                .on_input(Message::EntryLocatorChanged)
                .on_submit(Message::EntrySaveClicked)
                .font(FONT_MONO)
                .width(Length::Fixed(110.0)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding(12)
    .width(Length::Fill)
    .into()
}
