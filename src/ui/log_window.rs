use iced::widget::{column, container, horizontal_rule, row, scrollable, text};
use iced::{Element, Length};

use crate::app::{App, Message};

pub fn view(state: &App) -> Element<'_, Message> {
    column![
        table_header(),
        horizontal_rule(1),
        qso_list(state),
    ]
    .spacing(0)
    .into()
}

fn table_header() -> Element<'static, Message> {
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
    .padding([8, 12])
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
