use iced::widget::{column, container, horizontal_rule, row, text};
use iced::window;
use iced::{Alignment, Element, Length};

use crate::app::{App, Band, FONT_MONO, Message, Mode};
use crate::ui::buttons::outlined;
use crate::ui::inputs::{dropdown, input};
use crate::ui::title_bar;

pub fn view<'a>(state: &'a App, window_id: window::Id) -> Element<'a, Message> {
    container(
        column![
            title_bar::view(window_id, "BRlog", state.is_maximized(window_id), true),
            horizontal_rule(1).style(title_bar::rule_style),
            entry_row(state),
            horizontal_rule(1).style(title_bar::rule_style),
            macros_grid(),
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
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding(12)
    .width(Length::Fill)
    .into()
}

fn macros_grid() -> Element<'static, Message> {
    fn macro_row(range: std::ops::RangeInclusive<u8>) -> Element<'static, Message> {
        row(range.map(macro_button).collect::<Vec<_>>())
            .spacing(6)
            .into()
    }
    container(column![macro_row(1..=6), macro_row(7..=12)].spacing(6))
        .padding(12)
        .width(Length::Fill)
        .into()
}

fn macro_button(idx: u8) -> Element<'static, Message> {
    outlined(text(format!("F{idx}")).size(13))
        .on_press(Message::MacroPressed(idx))
        .width(Length::FillPortion(1))
        .into()
}
