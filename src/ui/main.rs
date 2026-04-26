use iced::widget::{
    button, column, container, horizontal_rule, pick_list, row, text, text_input, Space,
};
use iced::{Alignment, Element, Length};

use crate::app::{App, Band, Message, Mode, FONT_MONO};

pub fn view(state: &App) -> Element<'_, Message> {
    column![header(), horizontal_rule(1), entry_row(state)]
        .spacing(0)
        .into()
}

fn header() -> Element<'static, Message> {
    container(
        row![
            text("BRlog").size(22),
            Space::with_width(Length::Fill),
            button(text("\u{1F4CB} Deník").size(14))
                .on_press(Message::OpenLog)
                .style(button::secondary),
            button(text("\u{2699} Nastavení").size(14))
                .on_press(Message::OpenSettings)
                .style(button::secondary),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding(12)
    .width(Length::Fill)
    .into()
}

fn entry_row(state: &App) -> Element<'_, Message> {
    container(
        row![
            text_input("Volačka", &state.entry.callsign)
                .on_input(Message::EntryCallsignChanged)
                .on_submit(Message::EntrySaveClicked)
                .font(FONT_MONO)
                .width(Length::Fixed(130.0)),
            pick_list(Band::ALL, Some(state.entry.band), Message::EntryBandChanged)
                .width(Length::Fixed(85.0)),
            pick_list(Mode::ALL, Some(state.entry.mode), Message::EntryModeChanged)
                .width(Length::Fixed(85.0)),
            text_input("RST↑", &state.entry.rst_sent)
                .on_input(Message::EntryRstSentChanged)
                .font(FONT_MONO)
                .width(Length::Fixed(70.0)),
            text_input("RST↓", &state.entry.rst_rcvd)
                .on_input(Message::EntryRstRcvdChanged)
                .font(FONT_MONO)
                .width(Length::Fixed(70.0)),
            text_input("Lokátor", &state.entry.locator)
                .on_input(Message::EntryLocatorChanged)
                .font(FONT_MONO)
                .width(Length::Fixed(110.0)),
            Space::with_width(Length::Fill),
            button(text("Uložit"))
                .on_press(Message::EntrySaveClicked)
                .style(button::primary),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding(12)
    .width(Length::Fill)
    .into()
}
