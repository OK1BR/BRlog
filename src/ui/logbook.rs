use iced::widget::{column, container, text};
use iced::{Alignment, Element, Length};

use crate::app::{App, Message};

pub fn view(_state: &App) -> Element<'_, Message> {
    container(
        column![
            text("Deník").size(28),
            text("(zde bude tabulka QSO s filtrem a hledáním)").size(14),
        ]
        .spacing(8)
        .align_x(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}
