use iced::widget::{
    button, column, container, horizontal_rule, pick_list, row, scrollable, text, text_input,
    Space,
};
use iced::{Alignment, Element, Length};

use crate::app::{App, Band, Message, Mode};

pub fn view(state: &App) -> Element<'_, Message> {
    column![
        header(),
        horizontal_rule(1),
        entry_row(state),
        horizontal_rule(1),
        qso_table_header(),
        horizontal_rule(1),
        qso_list(state),
    ]
    .spacing(0)
    .into()
}

fn header() -> Element<'static, Message> {
    container(
        row![
            text("BRlog").size(22),
            Space::with_width(Length::Fill),
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
                .width(Length::Fixed(130.0)),
            pick_list(Band::ALL, Some(state.entry.band), Message::EntryBandChanged)
                .width(Length::Fixed(85.0)),
            pick_list(Mode::ALL, Some(state.entry.mode), Message::EntryModeChanged)
                .width(Length::Fixed(85.0)),
            text_input("RST↑", &state.entry.rst_sent)
                .on_input(Message::EntryRstSentChanged)
                .width(Length::Fixed(70.0)),
            text_input("RST↓", &state.entry.rst_rcvd)
                .on_input(Message::EntryRstRcvdChanged)
                .width(Length::Fixed(70.0)),
            text_input("Lokátor", &state.entry.locator)
                .on_input(Message::EntryLocatorChanged)
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

fn qso_table_header() -> Element<'static, Message> {
    container(
        row![
            head_cell("Datum", 100.0),
            head_cell("UTC", 70.0),
            head_cell("Volačka", 120.0),
            head_cell("Band", 70.0),
            head_cell("Mód", 70.0),
            head_cell("RST↑", 60.0),
            head_cell("RST↓", 60.0),
            head_cell("Lokátor", 80.0),
        ]
        .spacing(8),
    )
    .padding([6, 12])
    .width(Length::Fill)
    .into()
}

fn head_cell(label: &str, width: f32) -> Element<'_, Message> {
    text(label).size(13).width(Length::Fixed(width)).into()
}

fn qso_list(_state: &App) -> Element<'_, Message> {
    container(
        scrollable(
            container(text("(zatím žádná QSO — uložení přidáme po napojení DB)").size(13))
                .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
