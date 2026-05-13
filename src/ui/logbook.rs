//! Logbook manager window — dedicated CRUD surface for logbooks.
//!
//! Lives outside Settings on purpose: contests, DXpeditions and the like need
//! more room than a Settings page, and changes here (create/rename/delete) are
//! applied immediately, not through the Save/Cancel cycle.

use iced::widget::{
    button, column, container, row, rule, scrollable, text, Column, Space,
};
use iced::window;
use iced::{Alignment, Border, Element, Length, Shadow, Theme};

use crate::app::{App, FONT_MONO, Message};
use crate::models::log::{Log, LogKind};
use crate::t;
use crate::ui::buttons::{outlined, solid};
use crate::ui::inputs::{dropdown, input};
use crate::ui::title;

const NAME_COL_WIDTH: f32 = 240.0;
const KIND_COL_WIDTH: f32 = 160.0;
const FIELD_LABEL_WIDTH: f32 = 110.0;
const FIELD_INPUT_WIDTH: f32 = 300.0;
const FIELD_SPACING: f32 = 10.0;
const SECTION_SPACING: f32 = 22.0;
const MARKER_WIDTH: f32 = 30.0;

pub fn view<'a>(state: &'a App, window_id: window::Id) -> Element<'a, Message> {
    let is_maximized = state.is_maximized(window_id);
    container(
        column![
            title::view(state, window_id, t!("window-title-logbook"), false),
            rule::horizontal(1).style(title::rule_style),
            body(state),
        ]
        .spacing(0),
    )
    .style(title::window_border(
        state.config.appearance.window_border && !is_maximized,
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn body(state: &App) -> Element<'_, Message> {
    let rows: Vec<Element<Message>> = state.logs.iter().map(|log| log_row(state, log)).collect();

    let listing: Element<Message> = if rows.is_empty() {
        container(text(t!("logbook-empty")).size(13).style(muted_text))
            .padding([12, 0])
            .into()
    } else {
        Column::with_children(rows).spacing(4).into()
    };

    let existing = column![section_header(t!("settings-section-logbooks")), listing]
        .spacing(FIELD_SPACING);

    let create_button: Element<Message> = {
        let disabled = state.new_log_name.trim().is_empty();
        let btn = solid(text(t!("button-create-log")).size(14));
        if disabled {
            btn.into()
        } else {
            btn.on_press(Message::NewLogCreate).into()
        }
    };

    let new_form = column![
        row![
            field_label(t!("field-log-name")),
            input("", &state.new_log_name)
                .on_input(Message::NewLogNameChanged)
                .on_submit(Message::NewLogCreate)
                .font(FONT_MONO)
                .width(Length::Fixed(FIELD_INPUT_WIDTH)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        row![
            field_label(t!("field-log-kind")),
            dropdown(
                LogKind::ALL,
                Some(state.new_log_kind),
                Message::NewLogKindChanged,
                Length::Fixed(FIELD_INPUT_WIDTH),
            ),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
        row![
            Space::new().width(Length::Fixed(FIELD_LABEL_WIDTH)),
            create_button,
        ]
        .spacing(8),
    ]
    .spacing(FIELD_SPACING);

    let new_section = column![section_header(t!("settings-section-new-logbook")), new_form]
        .spacing(FIELD_SPACING);

    scrollable(
        container(
            column![existing, new_section]
                .spacing(SECTION_SPACING)
                .width(Length::Fill),
        )
        .padding([20, 24])
        .width(Length::Fill),
    )
    .height(Length::Fill)
    .width(Length::Fill)
    .into()
}

fn log_row<'a>(state: &'a App, log: &'a Log) -> Element<'a, Message> {
    let id = log.id;
    let is_active = log.is_active;
    let can_delete = state.logs.len() > 1 && state.db.count_qsos(id).unwrap_or(0) == 0;

    let marker = active_marker(is_active, id);

    let name_cell: Element<Message> = match state.log_rename_draft.as_ref() {
        Some(draft) if draft.id == id => input("", &draft.name)
            .on_input(Message::LogRenameChanged)
            .on_submit(Message::LogRenameCommit)
            .font(FONT_MONO)
            .width(Length::Fixed(NAME_COL_WIDTH))
            .into(),
        _ => container(text(log.name.clone()).size(13).font(FONT_MONO))
            .width(Length::Fixed(NAME_COL_WIDTH))
            .padding([4, 8])
            .into(),
    };

    let kind_cell = dropdown(
        LogKind::ALL,
        Some(log.kind),
        move |k| Message::LogKindChanged(id, k),
        Length::Fixed(KIND_COL_WIDTH),
    );

    let actions: Element<Message> = match state.log_rename_draft.as_ref() {
        Some(draft) if draft.id == id => row![
            outlined(text(t!("button-save")).size(13)).on_press(Message::LogRenameCommit),
            outlined(text(t!("button-cancel")).size(13)).on_press(Message::LogRenameCancel),
        ]
        .spacing(4)
        .into(),
        _ => {
            let delete_btn = outlined(text(t!("button-delete")).size(13));
            let delete_btn = if can_delete {
                delete_btn.on_press(Message::LogDelete(id))
            } else {
                delete_btn
            };
            row![
                outlined(text(t!("button-rename")).size(13))
                    .on_press(Message::LogRenameStart(id)),
                delete_btn,
            ]
            .spacing(4)
            .into()
        }
    };

    row![marker, name_cell, kind_cell, actions]
        .spacing(8)
        .align_y(Alignment::Center)
        .into()
}

fn active_marker(is_active: bool, id: i64) -> Element<'static, Message> {
    let glyph = if is_active { "\u{25CF}" } else { "\u{25CB}" };
    let btn = button(
        container(text(glyph).size(14))
            .padding([2, 6])
            .center_x(Length::Fixed(MARKER_WIDTH)),
    )
    .style(move |theme: &Theme, status: button::Status| marker_style(theme, status, is_active))
    .padding(0);

    if is_active {
        btn.into()
    } else {
        btn.on_press(Message::LogActivate(id)).into()
    }
}

fn marker_style(theme: &Theme, status: button::Status, active: bool) -> button::Style {
    let palette = theme.extended_palette();
    let fg = if active {
        palette.primary.strong.color
    } else {
        match status {
            button::Status::Hovered => palette.background.base.text,
            _ => {
                let mut c = palette.background.base.text;
                c.a = 0.45;
                c
            }
        }
    };
    button::Style {
        background: None,
        text_color: fg,
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

fn section_header(label: String) -> Element<'static, Message> {
    column![
        text(label).size(13).style(section_label_style),
        rule::horizontal(1).style(title::rule_style),
    ]
    .spacing(6)
    .into()
}

fn field_label(label: String) -> Element<'static, Message> {
    text(label)
        .size(13)
        .width(Length::Fixed(FIELD_LABEL_WIDTH))
        .into()
}

fn section_label_style(theme: &Theme) -> text::Style {
    let mut c = theme.extended_palette().background.base.text;
    c.a = 0.72;
    text::Style { color: Some(c) }
}

fn muted_text(theme: &Theme) -> text::Style {
    let mut c = theme.extended_palette().background.base.text;
    c.a = 0.6;
    text::Style { color: Some(c) }
}
