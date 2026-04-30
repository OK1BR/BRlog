//! Dropdown popup window — opens as a real, decoration-free OS window
//! anchored just below the trigger button. This sidesteps iced 0.13's
//! in-window overlay so band/mode menus can extend past the (very short)
//! main window.

use iced::widget::{button, column, container, row, scrollable, text};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use crate::app::{App, AppMenuItem, Band, DropdownKind, FONT_ICON, Message, Mode, PopupState};

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
    let rows: Vec<Element<'a, Message>> = match popup.kind {
        DropdownKind::Band => {
            let current = state.entry.band.to_string();
            Band::ALL
                .iter()
                .enumerate()
                .map(|(idx, b)| {
                    let label = b.to_string();
                    let selected = label == current;
                    value_row(popup.kind, idx, label, selected)
                })
                .collect()
        }
        DropdownKind::Mode => {
            let current = state.entry.mode.to_string();
            Mode::ALL
                .iter()
                .enumerate()
                .map(|(idx, m)| {
                    let label = m.to_string();
                    let selected = label == current;
                    value_row(popup.kind, idx, label, selected)
                })
                .collect()
        }
        DropdownKind::AppMenu => AppMenuItem::ALL
            .iter()
            .enumerate()
            .map(|(idx, item)| menu_row(popup.kind, idx, *item))
            .collect(),
    };

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

fn value_row<'a>(
    kind: DropdownKind,
    idx: usize,
    label: String,
    is_selected: bool,
) -> Element<'a, Message> {
    row_button(
        kind,
        idx,
        text(label).size(ROW_TEXT_SIZE).into(),
        is_selected,
    )
}

fn menu_row<'a>(kind: DropdownKind, idx: usize, item: AppMenuItem) -> Element<'a, Message> {
    let content = row![
        text(item.icon()).font(FONT_ICON).size(14),
        text(item.label()).size(ROW_TEXT_SIZE),
    ]
    .spacing(8)
    .align_y(Alignment::Center);
    row_button(kind, idx, content.into(), false)
}

fn row_button<'a>(
    kind: DropdownKind,
    idx: usize,
    content: Element<'a, Message>,
    is_selected: bool,
) -> Element<'a, Message> {
    button(content)
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
