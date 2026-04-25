use iced::widget::row;
use iced::{Element, Length, Task, Theme};

use crate::ui;

pub fn run() -> iced::Result {
    iced::application("BRlog", App::update, App::view)
        .theme(|_| Theme::Dark)
        .window_size((1100.0, 700.0))
        .run_with(App::new)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Logbook,
    NewQso,
    Settings,
}

#[derive(Debug, Clone)]
pub enum Message {
    Navigate(View),
}

pub struct App {
    pub current_view: View,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                current_view: View::Logbook,
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Navigate(target) => self.current_view = target,
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        let content: Element<Message> = match self.current_view {
            View::Logbook => ui::logbook::view(self),
            View::NewQso => ui::qso_form::view(self),
            View::Settings => ui::settings::view(self),
        };

        row![ui::sidebar::view(self), content]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
