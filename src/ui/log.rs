use iced::widget::{
    button, column, container, mouse_area, row, rule, scrollable, stack, text, Column, Space,
};
use iced::window;
use iced::{
    Alignment, Background, Border, Color, Element, Length, Padding, Point, Shadow, Theme, Vector,
};

use crate::app::{App, ContextMenuState, Message, FONT_MONO};
use crate::models::qso::{format_frequency_hz, Qso};
use crate::t;
use crate::ui::{resize, title};

const COL_DATE: f32 = 100.0;
const COL_UTC: f32 = 70.0;
const COL_CALL: f32 = 120.0;
const COL_FREQ: f32 = 100.0;
const COL_MODE: f32 = 70.0;
const COL_RST_S: f32 = 60.0;
const COL_RST_R: f32 = 60.0;
const COL_LOC: f32 = 80.0;

const MENU_WIDTH: f32 = 180.0;
const POPOVER_RADIUS: f32 = 6.0;
const MENU_ITEM_RADIUS: f32 = 4.0;

pub fn view<'a>(state: &'a App, window_id: window::Id) -> Element<'a, Message> {
    let is_maximized = state.is_maximized(window_id);
    let body: Element<'a, Message> = column![
        title::view(state, window_id, t!("window-title-log"), false),
        rule::horizontal(1).style(title::rule_style),
        table_header(),
        rule::horizontal(1).style(title::rule_style),
        qso_list(state),
    ]
    .spacing(0)
    .into();

    // Wrap the body in a mouse_area to (a) track cursor position so right-click
    // can anchor the menu near it, and (b) catch outside-of-menu clicks for
    // dismissal. Children get events first, so a click on a menu item or a row
    // button is captured before this on_press fires.
    let tracked: Element<'a, Message> = mouse_area(body)
        .on_move(Message::LogCursorMoved)
        .on_press(Message::ContextMenuDismiss)
        .into();

    // Stack is always present so the Scrollable's tree position is stable;
    // toggling whether Stack wraps the body would reset scroll on every menu
    // open/close.
    let menu_layer: Element<'a, Message> = match state.context_menu.as_ref() {
        Some(menu) => context_menu_overlay(menu),
        None => Space::new().into(),
    };

    let layered: Element<'a, Message> = stack![tracked, menu_layer]
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    container(resize::wrap(layered, window_id, !is_maximized))
        .style(title::window_border(
            state.config.appearance.window_border && !is_maximized,
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn table_header() -> Element<'static, Message> {
    container(
        row![
            head_cell(t!("field-date"), COL_DATE),
            head_cell(t!("field-utc"), COL_UTC),
            head_cell(t!("field-callsign"), COL_CALL),
            head_cell(t!("field-frequency"), COL_FREQ),
            head_cell(t!("field-mode"), COL_MODE),
            head_cell(t!("field-rst-sent"), COL_RST_S),
            head_cell(t!("field-rst-rcvd"), COL_RST_R),
            head_cell(t!("field-locator"), COL_LOC),
        ]
        .spacing(8),
    )
    .padding([8, 12])
    .width(Length::Fill)
    .into()
}

fn head_cell(label: String, width: f32) -> Element<'static, Message> {
    text(label).size(13).width(Length::Fixed(width)).into()
}

fn qso_list(state: &App) -> Element<'_, Message> {
    if state.qsos.is_empty() {
        return container(text(t!("log-empty")).size(13))
            .padding(20)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
    }

    let selected = state.selected_qso_id;
    let rows: Vec<Element<Message>> = state
        .qsos
        .iter()
        .map(|qso| qso_row(qso, qso.id.is_some() && qso.id == selected))
        .collect();

    scrollable(Column::with_children(rows).spacing(0))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn qso_row(qso: &Qso, selected: bool) -> Element<'_, Message> {
    let date = qso.qso_datetime.format("%Y-%m-%d").to_string();
    let utc = qso.qso_datetime.format("%H:%M").to_string();
    let inner = container(
        row![
            cell(date, COL_DATE),
            cell(utc, COL_UTC),
            cell(qso.callsign.clone(), COL_CALL),
            cell(format_frequency_hz(qso.frequency), COL_FREQ),
            cell(qso.mode.clone(), COL_MODE),
            cell(qso.rst_sent.clone(), COL_RST_S),
            cell(qso.rst_rcvd.clone(), COL_RST_R),
            cell(qso.locator.clone(), COL_LOC),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([4, 12])
    .width(Length::Fill);

    let mut btn = button(inner)
        .style(move |theme: &Theme, status: button::Status| row_style(theme, status, selected))
        .padding(0)
        .width(Length::Fill);

    // QSOs read from the DB always have an id; guard anyway so an in-memory
    // unsaved row would degrade gracefully into a non-interactive strip.
    let Some(id) = qso.id else {
        return btn.into();
    };
    btn = btn.on_press(Message::QsoSelected(id));

    // Right-click goes to the context menu. mouse_area's on_right_press captures
    // the event, so the inner button's left-click handler is unaffected.
    mouse_area(btn).on_right_press(Message::QsoContextMenu(id)).into()
}

fn row_style(theme: &Theme, status: button::Status, selected: bool) -> button::Style {
    let palette = theme.extended_palette();

    let (bg, fg) = if selected {
        (
            Some(palette.primary.weak.color),
            palette.primary.weak.text,
        )
    } else {
        match status {
            button::Status::Hovered => (Some(ghost(theme, 0.07, 0.05)), palette.background.base.text),
            button::Status::Pressed => (Some(ghost(theme, 0.12, 0.09)), palette.background.base.text),
            _ => (None, palette.background.base.text),
        }
    };

    button::Style {
        background: bg.map(Background::Color),
        text_color: fg,
        border: Border::default(),
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

fn ghost(theme: &Theme, dark_alpha: f32, light_alpha: f32) -> Color {
    let palette = theme.extended_palette();
    Color {
        a: if palette.is_dark { dark_alpha } else { light_alpha },
        ..palette.background.base.text
    }
}

fn cell(value: String, width: f32) -> Element<'static, Message> {
    text(value)
        .size(13)
        .font(FONT_MONO)
        .width(Length::Fixed(width))
        .into()
}

// ── Context menu (Zed-style popover) ────────────────────────────────────────

fn context_menu_overlay(menu: &ContextMenuState) -> Element<'static, Message> {
    let id = menu.qso_id;
    let popover = container(
        column![menu_item(t!("menu-delete"), Message::DeleteQsoConfirmed(id), true)]
            .spacing(2),
    )
    .style(popover_style)
    .padding(4)
    .width(Length::Fixed(MENU_WIDTH));

    // Position the popover inside the stack by padding it from the body's
    // top-left. `position` is in body-local coords (from on_move), so this
    // anchors the popover near the cursor at right-click time.
    let anchor = clamp_anchor(menu.position);
    container(popover)
        .padding(Padding {
            top: anchor.y,
            left: anchor.x,
            right: 0.0,
            bottom: 0.0,
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Keep the popover from running off the top-left edge of the body. We don't
/// know the viewport width here, so the right/bottom clamping happens
/// implicitly: if the cursor is near the edge, Iced will still render and the
/// menu may overflow the visible area. Adjust later when we have viewport info.
fn clamp_anchor(p: Point) -> Point {
    Point::new(p.x.max(0.0), p.y.max(0.0))
}

fn menu_item(
    label: String,
    on_press: Message,
    destructive: bool,
) -> Element<'static, Message> {
    button(
        container(text(label).size(13))
            .padding([4, 10])
            .width(Length::Fill),
    )
    .on_press(on_press)
    .style(move |theme: &Theme, status: button::Status| {
        menu_item_style(theme, status, destructive)
    })
    .padding(0)
    .width(Length::Fill)
    .into()
}

fn menu_item_style(theme: &Theme, status: button::Status, destructive: bool) -> button::Style {
    let palette = theme.extended_palette();

    let (bg, fg) = match (status, destructive) {
        (button::Status::Hovered, true) => (
            Some(palette.danger.weak.color),
            palette.danger.weak.text,
        ),
        (button::Status::Hovered, false) => (
            Some(palette.background.weak.color),
            palette.background.weak.text,
        ),
        (button::Status::Pressed, true) => (
            Some(palette.danger.base.color),
            palette.danger.base.text,
        ),
        (button::Status::Pressed, false) => (
            Some(palette.background.strong.color),
            palette.background.strong.text,
        ),
        (_, true) => (None, palette.danger.strong.color),
        (_, false) => (None, palette.background.base.text),
    };

    button::Style {
        background: bg.map(Background::Color),
        text_color: fg,
        border: Border {
            radius: MENU_ITEM_RADIUS.into(),
            ..Border::default()
        },
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

fn popover_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let mut border_color = palette.background.strong.color;
    border_color.a = 0.4;

    container::Style {
        background: Some(Background::Color(palette.background.base.color)),
        text_color: Some(palette.background.base.text),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: POPOVER_RADIUS.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
            offset: Vector::new(0.0, 4.0),
            blur_radius: 12.0,
        },
        ..container::Style::default()
    }
}
