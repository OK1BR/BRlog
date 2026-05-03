use iced::widget::{column, container, row, rule, text};
use iced::window;
use iced::{Alignment, Element, Length};

use crate::app::{App, Band, FONT_MONO, Message, Mode};
use crate::t;
use crate::ui::buttons::outlined;
use crate::ui::inputs::{dropdown, input};
use crate::ui::{resize, title};

pub fn view<'a>(state: &'a App, window_id: window::Id) -> Element<'a, Message> {
    let is_maximized = state.is_maximized(window_id);
    let body: Element<'a, Message> = column![
        title::view(window_id, t!("window-title-app"), is_maximized, true),
        rule::horizontal(1).style(title::rule_style),
        entry_row(state),
        rule::horizontal(1).style(title::rule_style),
        macros_grid(),
    ]
    .spacing(0)
    .into();

    container(resize::wrap(body, window_id, !is_maximized))
        .style(title::window_border(
            state.config.appearance.window_border && !is_maximized,
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn entry_row(state: &App) -> Element<'_, Message> {
    container(
        row![
            input(&t!("field-callsign"), &state.entry.callsign)
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
            input(&t!("field-rst-sent"), &state.entry.rst_sent)
                .on_input(Message::EntryRstSentChanged)
                .on_submit(Message::EntrySaveClicked)
                .font(FONT_MONO)
                .width(Length::Fixed(70.0)),
            input(&t!("field-rst-rcvd"), &state.entry.rst_rcvd)
                .on_input(Message::EntryRstRcvdChanged)
                .on_submit(Message::EntrySaveClicked)
                .font(FONT_MONO)
                .width(Length::Fixed(70.0)),
            input(&t!("field-locator"), &state.entry.locator)
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
