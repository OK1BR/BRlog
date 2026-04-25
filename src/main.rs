use iced::widget::{column, container, svg, text};
use iced::{Alignment, Element, Length, Task};

const LOGO_BYTES: &[u8] = include_bytes!("../assets/BRlog_logo_v5.svg");

fn main() -> iced::Result {
    iced::application("BRlog", App::update, App::view)
        .window_size((900.0, 600.0))
        .run_with(App::new)
}

#[derive(Default)]
struct App {}

#[derive(Debug, Clone)]
enum Message {}

impl App {
    fn new() -> (Self, Task<Message>) {
        (Self::default(), Task::none())
    }

    fn update(&mut self, _message: Message) -> Task<Message> {
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let logo = svg(svg::Handle::from_memory(LOGO_BYTES.to_vec()))
            .width(Length::Fixed(220.0))
            .height(Length::Fixed(220.0));

        let title = text("BRlog").size(48);
        let subtitle = text("radioamatérský denník").size(18);

        container(
            column![logo, title, subtitle]
                .spacing(16)
                .align_x(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
    }
}
