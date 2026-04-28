use iced::widget::{column, container, horizontal_rule, row, text, Space};
use iced::window;
use iced::{Alignment, Element, Length};

use crate::app::{
    App, Band, Message, Mode, FONT_ICON, FONT_MONO, ICON_LIST, ICON_SETTINGS,
};
use crate::ui::buttons::{outlined, solid};
use crate::ui::inputs::{dropdown, input};
use crate::ui::title_bar;

pub fn view<'a>(state: &'a App, window_id: window::Id) -> Element<'a, Message> {
    container(
        column![
            title_bar::view(window_id, "BRlog", state.is_maximized(window_id)),
            horizontal_rule(1).style(title_bar::rule_style),
            header(),
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

fn header() -> Element<'static, Message> {
    container(
        row![
            text("BRlog").size(22),
            Space::with_width(Length::Fill),
            outlined(
                row![
                    text(ICON_LIST).font(FONT_ICON).size(14),
                    text("Deník").size(14),
                ]
                .spacing(6)
                .align_y(Alignment::Center)
            )
            .on_press(Message::OpenLog),
            outlined(
                row![
                    text(ICON_SETTINGS).font(FONT_ICON).size(14),
                    text("Nastavení").size(14),
                ]
                .spacing(6)
                .align_y(Alignment::Center)
            )
            .on_press(Message::OpenSettings),
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
            input("Volačka", &state.entry.callsign)
                .on_input(Message::EntryCallsignChanged)
                .on_submit(Message::EntrySaveClicked)
                .font(FONT_MONO)
                .width(Length::Fixed(130.0)),
            dropdown(
                Band::ALL,
                Some(state.entry.band),
                Message::EntryBandChanged,
                Length::Fixed(85.0),
            ),
            dropdown(
                Mode::ALL,
                Some(state.entry.mode),
                Message::EntryModeChanged,
                Length::Fixed(85.0),
            ),
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
            Space::with_width(Length::Fill),
            solid(text("Uložit").size(14)).on_press(Message::EntrySaveClicked),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding(12)
    .width(Length::Fill)
    .into()
}
