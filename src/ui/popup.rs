//! Dropdown popup window — opens as a real, decoration-free OS window
//! anchored just below the trigger button. This sidesteps iced 0.13's
//! in-window overlay so band/mode menus can extend past the (very short)
//! main window.

use iced::widget::{button, column, container, scrollable, text};
use iced::{Background, Border, Color, Element, Length, Theme};

use crate::app::{App, Band, DropdownKind, Message, Mode, PopupState};

pub const POPUP_WIDTH: f32 = 140.0;
pub const POPUP_ROW_HEIGHT: f32 = 28.0;
pub const POPUP_PADDING: f32 = 4.0;

const ROW_TEXT_SIZE: f32 = 14.0;

fn subtle_border(theme: &Theme) -> Color {
    let mut c = theme.extended_palette().background.strong.color;
    c.a = 0.4;
    c
}

pub fn view<'a>(state: &'a App, popup: PopupState) -> Element<'a, Message> {
    let (current, labels): (String, Vec<String>) = match popup.kind {
        DropdownKind::Band => (
            state.entry.band.to_string(),
            Band::ALL.iter().map(|b| b.to_string()).collect(),
        ),
        DropdownKind::Mode => (
            state.entry.mode.to_string(),
            Mode::ALL.iter().map(|m| m.to_string()).collect(),
        ),
    };

    let rows: Vec<Element<'a, Message>> = labels
        .into_iter()
        .enumerate()
        .map(|(idx, label)| {
            let is_selected = label == current;
            row_button(popup.kind, idx, label, is_selected)
        })
        .collect();

    container(scrollable(column(rows).spacing(0)).height(Length::Fill))
        .padding(POPUP_PADDING)
        .style(|theme: &Theme| container::Style {
            background: Some(Background::Color(
                theme.extended_palette().background.base.color,
            )),
            border: Border {
                color: subtle_border(theme),
                width: 1.0,
                radius: 6.0.into(),
            },
            ..container::Style::default()
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn row_button<'a>(
    kind: DropdownKind,
    idx: usize,
    label: String,
    is_selected: bool,
) -> Element<'a, Message> {
    button(text(label).size(ROW_TEXT_SIZE))
        .on_press(Message::DropdownItemSelected(kind, idx))
        .padding([4, 10])
        .width(Length::Fill)
        .style(move |theme: &Theme, status: button::Status| {
            let palette = theme.extended_palette();
            let (bg, fg) = match status {
                button::Status::Hovered | button::Status::Pressed => (
                    Some(Background::Color(palette.primary.weak.color)),
                    palette.primary.weak.text,
                ),
                _ if is_selected => (
                    Some(Background::Color(palette.primary.weak.color)),
                    palette.primary.weak.text,
                ),
                _ => (None, palette.background.base.text),
            };
            button::Style {
                background: bg,
                text_color: fg,
                border: Border {
                    radius: 4.0.into(),
                    ..Border::default()
                },
                ..Default::default()
            }
        })
        .into()
}
