use iced::widget::{Space, button, container, mouse_area, row, rule, text};
use iced::window;
use iced::{Alignment, Background, Border, Color, Element, Length, Shadow, Theme};

use crate::app::{FONT_ICON, FONT_UI, ICON_MAXIMIZE, ICON_MINUS, ICON_RESTORE, ICON_X, Message};

const HEIGHT: f32 = 32.0;
const CTRL_WIDTH: f32 = 46.0;
const LIGHT_SIZE: f32 = 12.0;

// Windows-style close-on-hover red, slightly warmer than pure red.
const CLOSE_HOVER: Color = Color {
    r: 0.91,
    g: 0.28,
    b: 0.30,
    a: 1.0,
};
const CLOSE_PRESSED: Color = Color {
    r: 0.78,
    g: 0.23,
    b: 0.25,
    a: 1.0,
};

pub fn view<'a>(window_id: window::Id, title: &'a str, is_maximized: bool) -> Element<'a, Message> {
    if cfg!(target_os = "macos") {
        macos_layout(window_id, title)
    } else {
        windows_layout(window_id, title, is_maximized)
    }
}

fn windows_layout<'a>(
    window_id: window::Id,
    title: &'a str,
    is_maximized: bool,
) -> Element<'a, Message> {
    let max_icon = if is_maximized {
        ICON_RESTORE
    } else {
        ICON_MAXIMIZE
    };

    container(
        row![
            mouse_area(
                container(text(title).size(13).font(FONT_UI))
                    .padding([0, 12])
                    .center_y(Length::Fixed(HEIGHT))
                    .width(Length::Fill)
                    .height(Length::Fixed(HEIGHT))
            )
            .on_press(Message::WindowDrag(window_id)),
            ctrl_button(ICON_MINUS, Message::WindowMinimize(window_id), false),
            ctrl_button(max_icon, Message::WindowMaximizeToggle(window_id), false),
            ctrl_button(ICON_X, Message::WindowCloseRequested(window_id), true),
        ]
        .height(Length::Fixed(HEIGHT))
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(HEIGHT))
    .into()
}

fn macos_layout<'a>(window_id: window::Id, title: &'a str) -> Element<'a, Message> {
    container(
        row![
            light_button(
                Color::from_rgb8(0xFF, 0x5F, 0x57),
                Message::WindowCloseRequested(window_id)
            ),
            light_button(
                Color::from_rgb8(0xFE, 0xBC, 0x2E),
                Message::WindowMinimize(window_id)
            ),
            light_button(
                Color::from_rgb8(0x28, 0xC8, 0x40),
                Message::WindowMaximizeToggle(window_id)
            ),
            mouse_area(
                container(text(title).size(13).font(FONT_UI))
                    .center_x(Length::Fill)
                    .center_y(Length::Fixed(HEIGHT))
                    .width(Length::Fill)
                    .height(Length::Fixed(HEIGHT))
            )
            .on_press(Message::WindowDrag(window_id)),
        ]
        .spacing(8)
        .padding([0, 12])
        .height(Length::Fixed(HEIGHT))
        .align_y(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fixed(HEIGHT))
    .into()
}

fn ctrl_button(icon: &'static str, msg: Message, is_close: bool) -> Element<'static, Message> {
    button(
        container(text(icon).size(14).font(FONT_ICON))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .on_press(msg)
    .style(move |theme: &Theme, status: button::Status| {
        let palette = theme.extended_palette();
        let (bg, fg) = match status {
            button::Status::Hovered if is_close => (Some(CLOSE_HOVER), Color::WHITE),
            button::Status::Pressed if is_close => (Some(CLOSE_PRESSED), Color::WHITE),
            button::Status::Hovered => (
                Some(palette.background.strong.color),
                palette.background.strong.text,
            ),
            button::Status::Pressed => {
                (Some(palette.primary.weak.color), palette.primary.weak.text)
            }
            _ => (None, palette.background.base.text),
        };
        button::Style {
            background: bg.map(Background::Color),
            text_color: fg,
            border: Border::default(),
            shadow: Shadow::default(),
            ..Default::default()
        }
    })
    .width(Length::Fixed(CTRL_WIDTH))
    .height(Length::Fixed(HEIGHT))
    .padding(0)
    .into()
}

/// Subtle separator color — slightly transparent strong-bg, looks like a thinner line.
fn subtle_line(theme: &Theme) -> Color {
    let mut c = theme.extended_palette().background.strong.color;
    c.a = 0.4;
    c
}

pub fn window_border(enabled: bool) -> impl Fn(&Theme) -> container::Style {
    move |theme: &Theme| {
        if enabled {
            container::Style {
                border: Border {
                    color: subtle_line(theme),
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..container::Style::default()
            }
        } else {
            container::Style::default()
        }
    }
}

pub fn rule_style(theme: &Theme) -> rule::Style {
    rule::Style {
        color: subtle_line(theme),
        width: 1,
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
    }
}

fn light_button(color: Color, msg: Message) -> Element<'static, Message> {
    button(Space::new(
        Length::Fixed(LIGHT_SIZE),
        Length::Fixed(LIGHT_SIZE),
    ))
    .on_press(msg)
    .width(Length::Fixed(LIGHT_SIZE))
    .height(Length::Fixed(LIGHT_SIZE))
    .padding(0)
    .style(move |_theme: &Theme, _status| button::Style {
        background: Some(Background::Color(color)),
        border: Border {
            radius: (LIGHT_SIZE / 2.0).into(),
            ..Border::default()
        },
        ..Default::default()
    })
    .into()
}
