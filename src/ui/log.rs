use iced::widget::{column, container, horizontal_rule, row, scrollable, text, Column};
use iced::window;
use iced::{Alignment, Element, Length};

use crate::app::{App, Message, FONT_MONO};
use crate::models::qso::Qso;
use crate::ui::title_bar;

const COL_DATE: f32 = 100.0;
const COL_UTC: f32 = 70.0;
const COL_CALL: f32 = 120.0;
const COL_BAND: f32 = 70.0;
const COL_MODE: f32 = 70.0;
const COL_RST_S: f32 = 60.0;
const COL_RST_R: f32 = 60.0;
const COL_LOC: f32 = 80.0;

pub fn view<'a>(state: &'a App, window_id: window::Id) -> Element<'a, Message> {
    container(
        column![
            title_bar::view(window_id, "BRlog — Deník", state.is_maximized(window_id)),
            horizontal_rule(1).style(title_bar::rule_style),
            table_header(),
            horizontal_rule(1).style(title_bar::rule_style),
            qso_list(state),
        ]
        .spacing(0),
    )
    .style(title_bar::window_border(state.config.appearance.window_border))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn table_header() -> Element<'static, Message> {
    container(
        row![
            head_cell("Datum", COL_DATE),
            head_cell("UTC", COL_UTC),
            head_cell("Volačka", COL_CALL),
            head_cell("Band", COL_BAND),
            head_cell("Mód", COL_MODE),
            head_cell("RST↑", COL_RST_S),
            head_cell("RST↓", COL_RST_R),
            head_cell("Lokátor", COL_LOC),
        ]
        .spacing(8),
    )
    .padding([8, 12])
    .width(Length::Fill)
    .into()
}

fn head_cell(label: &str, width: f32) -> Element<'_, Message> {
    text(label).size(13).width(Length::Fixed(width)).into()
}

fn qso_list(state: &App) -> Element<'_, Message> {
    if state.qsos.is_empty() {
        return container(text("Žádná QSO. Zaloguj první přes hlavní okno.").size(13))
            .padding(20)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    let rows: Vec<Element<Message>> = state.qsos.iter().map(qso_row).collect();

    scrollable(Column::with_children(rows).spacing(0))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn qso_row(qso: &Qso) -> Element<'_, Message> {
    let date = qso.qso_datetime.format("%Y-%m-%d").to_string();
    let utc = qso.qso_datetime.format("%H:%M").to_string();
    container(
        row![
            cell(date, COL_DATE),
            cell(utc, COL_UTC),
            cell(qso.callsign.clone(), COL_CALL),
            cell(qso.band.to_string(), COL_BAND),
            cell(qso.mode.to_string(), COL_MODE),
            cell(qso.rst_sent.clone(), COL_RST_S),
            cell(qso.rst_rcvd.clone(), COL_RST_R),
            cell(qso.locator.clone(), COL_LOC),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([4, 12])
    .width(Length::Fill)
    .into()
}

fn cell(value: String, width: f32) -> Element<'static, Message> {
    text(value)
        .size(13)
        .font(FONT_MONO)
        .width(Length::Fixed(width))
        .into()
}
