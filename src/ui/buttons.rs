//! Zed-style buttons matching the outlined controls in `inputs.rs` —
//! same radius, border treatment and focus colors so the entire UI reads
//! as one consistent set of widgets.

use iced::widget::button::{self, Button};
use iced::widget::button as button_widget;
use iced::{Background, Border, Element, Theme};

const RADIUS: f32 = 4.0;
const PADDING_Y: u16 = 4;
const PADDING_X: u16 = 12;

fn subtle_border(theme: &Theme) -> iced::Color {
    let mut c = theme.extended_palette().background.strong.color;
    c.a = 0.4;
    c
}

/// Outlined button — 1px subtle border, transparent fill at rest.
/// Hover: full strong border + subtle background. Press: primary.weak fill.
pub fn outlined_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let (bg, border_color, text_color) = match status {
        button::Status::Active => (
            None,
            subtle_border(theme),
            palette.background.base.text,
        ),
        button::Status::Hovered => (
            Some(Background::Color(palette.background.weak.color)),
            palette.background.strong.color,
            palette.background.base.text,
        ),
        button::Status::Pressed => (
            Some(Background::Color(palette.primary.weak.color)),
            palette.primary.strong.color,
            palette.primary.weak.text,
        ),
        button::Status::Disabled => {
            let mut c = palette.background.base.text;
            c.a = 0.4;
            (None, subtle_border(theme), c)
        }
    };

    button::Style {
        background: bg,
        text_color,
        border: Border {
            color: border_color,
            width: 1.0,
            radius: RADIUS.into(),
        },
        shadow: Default::default(),
    }
}

/// Solid button — primary.strong fill, used for the main action of a screen
/// (Save / Confirm). Mirrors Zed's `Solid` filled style.
pub fn solid_style(theme: &Theme, status: button::Status) -> button::Style {
    let palette = theme.extended_palette();

    let (bg_color, text_color) = match status {
        button::Status::Active => (palette.primary.strong.color, palette.primary.strong.text),
        button::Status::Hovered => (palette.primary.base.color, palette.primary.base.text),
        button::Status::Pressed => (palette.primary.weak.color, palette.primary.weak.text),
        button::Status::Disabled => {
            let mut c = palette.primary.strong.color;
            c.a = 0.4;
            (c, palette.primary.strong.text)
        }
    };

    button::Style {
        background: Some(Background::Color(bg_color)),
        text_color,
        border: Border {
            color: bg_color,
            width: 1.0,
            radius: RADIUS.into(),
        },
        shadow: Default::default(),
    }
}

pub fn outlined<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Button<'a, Message> {
    button_widget(content)
        .style(outlined_style)
        .padding([PADDING_Y, PADDING_X])
}

pub fn solid<'a, Message: Clone + 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Button<'a, Message> {
    button_widget(content)
        .style(solid_style)
        .padding([PADDING_Y, PADDING_X])
}
