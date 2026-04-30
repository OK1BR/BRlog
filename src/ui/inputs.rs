//! Zed-style outlined controls (dropdown + text input) sharing the same
//! border, radius, padding and focus colors so the entry row reads as one
//! visually consistent strip.

use iced::overlay::menu;
use iced::widget::pick_list::{self, Handle};
use iced::widget::text_input::{self, TextInput};
use iced::widget::{pick_list as pick_list_widget, text_input as text_input_widget};
use iced::{Background, Border, Element, Length, Pixels, Theme};

const RADIUS: f32 = 4.0;
const MENU_RADIUS: f32 = 6.0;
const PADDING_Y: u16 = 4;
const PADDING_X: u16 = 8;
const TEXT_SIZE: f32 = 14.0;
const HANDLE_SIZE: f32 = 12.0;

fn subtle_border(theme: &Theme) -> iced::Color {
    let mut c = theme.extended_palette().background.strong.color;
    c.a = 0.4;
    c
}

fn pick_list_style(theme: &Theme, status: pick_list::Status) -> pick_list::Style {
    let palette = theme.extended_palette();

    let border_color = match status {
        pick_list::Status::Active => subtle_border(theme),
        pick_list::Status::Hovered => palette.background.strong.color,
        pick_list::Status::Opened => palette.primary.strong.color,
    };

    pick_list::Style {
        text_color: palette.background.base.text,
        placeholder_color: palette.background.strong.color,
        handle_color: palette.background.base.text,
        background: Background::Color(palette.background.base.color),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: RADIUS.into(),
        },
    }
}

fn menu_style(theme: &Theme) -> menu::Style {
    let palette = theme.extended_palette();

    menu::Style {
        background: Background::Color(palette.background.base.color),
        border: Border {
            color: subtle_border(theme),
            width: 1.0,
            radius: MENU_RADIUS.into(),
        },
        text_color: palette.background.base.text,
        selected_background: Background::Color(palette.primary.weak.color),
        selected_text_color: palette.primary.weak.text,
    }
}

fn text_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.extended_palette();

    let border_color = match status {
        text_input::Status::Active => subtle_border(theme),
        text_input::Status::Hovered => palette.background.strong.color,
        text_input::Status::Focused => palette.primary.strong.color,
        text_input::Status::Disabled => subtle_border(theme),
    };

    let bg = match status {
        text_input::Status::Disabled => palette.background.weak.color,
        _ => palette.background.base.color,
    };

    let value = match status {
        text_input::Status::Disabled => palette.background.strong.color,
        _ => palette.background.base.text,
    };

    text_input::Style {
        background: Background::Color(bg),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: RADIUS.into(),
        },
        icon: palette.background.weak.text,
        placeholder: palette.background.strong.color,
        value,
        selection: palette.primary.weak.color,
    }
}

pub fn dropdown<'a, T, Message>(
    options: &'a [T],
    selected: Option<T>,
    on_select: impl Fn(T) -> Message + 'a,
    width: Length,
) -> Element<'a, Message>
where
    T: Clone + ToString + PartialEq + 'a,
    Message: Clone + 'a,
{
    pick_list_widget(options, selected, on_select)
        .style(pick_list_style)
        .menu_style(menu_style)
        .padding([PADDING_Y, PADDING_X])
        .text_size(TEXT_SIZE)
        .handle(Handle::Arrow {
            size: Some(Pixels(HANDLE_SIZE)),
        })
        .width(width)
        .into()
}

pub fn input<'a, Message: Clone + 'a>(
    placeholder: &str,
    value: &str,
) -> TextInput<'a, Message> {
    text_input_widget(placeholder, value)
        .style(text_input_style)
        .padding([PADDING_Y, PADDING_X])
        .size(TEXT_SIZE)
}
