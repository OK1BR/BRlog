use iced::widget::{button, column, container, text, vertical_space, Space};
use iced::{Alignment, Element, Length};

use crate::app::{App, Message, View};

const SIDEBAR_WIDTH: f32 = 220.0;

pub fn view(state: &App) -> Element<'_, Message> {
    let title = text("BRlog").size(28);

    container(
        column![
            title,
            Space::with_height(Length::Fixed(20.0)),
            nav_button("Deník", View::Logbook, state.current_view),
            nav_button("Nové QSO", View::NewQso, state.current_view),
            nav_button("Nastavení", View::Settings, state.current_view),
            vertical_space(),
            text(format!("v{}", env!("CARGO_PKG_VERSION"))).size(11),
        ]
        .spacing(6)
        .padding(12)
        .align_x(Alignment::Center),
    )
    .width(Length::Fixed(SIDEBAR_WIDTH))
    .height(Length::Fill)
    .into()
}

fn nav_button(label: &'static str, target: View, current: View) -> Element<'static, Message> {
    let btn = button(text(label).size(15))
        .width(Length::Fill)
        .on_press(Message::Navigate(target));

    if current == target {
        btn.style(button::primary).into()
    } else {
        btn.style(button::text).into()
    }
}
