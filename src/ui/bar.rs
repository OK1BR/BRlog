//! Zed-style status bar — thin strip at the bottom of the main window with
//! QSO count on the left and a row of background-task indicator icons on the
//! right (TCI, DX cluster, QRZ lookup, sync queue, UTC clock).
//!
//! Each indicator carries a [`ConnState`] so future TCI / cluster / sync
//! subsystems can flip it from `Disconnected` (muted) → `Connecting` (amber)
//! → `Connected` (green) / `Error` (red) without touching the view code.

use chrono::{DateTime, Utc};
use iced::widget::tooltip::Position;
use iced::widget::{container, row, rule, text, tooltip, Space};
use iced::{Alignment, Background, Border, Color, Element, Length, Theme};

use crate::app::{App, FONT_ICON, FONT_MONO, Message};
use crate::t;
use crate::ui::title;

const HEIGHT: f32 = 24.0;
const ICON_SIZE: f32 = 14.0;
const ICON_SLOT: f32 = 26.0;
const TEXT_SIZE: f32 = 12.0;
const SIDE_PADDING: u16 = 10;

// Lucide codepoints for the background-task indicators.
const ICON_RADIO_TOWER: &str = "\u{E404}"; // TCI / transceiver link
const ICON_NETWORK: &str = "\u{E125}"; // DX cluster TCP link
const ICON_USER_SEARCH: &str = "\u{E579}"; // QRZ / callbook lookup
const ICON_CLOUD_UPLOAD: &str = "\u{E091}"; // eQSL / LoTW / HamQTH upload queue

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
// Connecting/Connected/Error are flipped on by future TCI / cluster / sync
// workers — keep them defined so the view code can already render every state.
#[allow(dead_code)]
pub enum ConnState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BackgroundStatus {
    pub tci: ConnState,
    pub cluster: ConnState,
    pub qrz: ConnState,
    pub sync: ConnState,
}

pub fn view(state: &App) -> Element<'_, Message> {
    let qso_count = text(format!("{} QSO", state.qsos.len()))
        .size(TEXT_SIZE)
        .style(muted_text);

    let utc = text(state.current_utc.format("%H:%M:%S UTC").to_string())
        .size(TEXT_SIZE)
        .font(FONT_MONO)
        .style(muted_text);

    let bar = row![
        qso_count,
        Space::new().width(Length::Fill),
        indicator(ICON_RADIO_TOWER, state.bg_status.tci, t!("status-tci")),
        indicator(ICON_NETWORK, state.bg_status.cluster, t!("status-cluster")),
        indicator(ICON_USER_SEARCH, state.bg_status.qrz, t!("status-qrz")),
        indicator(ICON_CLOUD_UPLOAD, state.bg_status.sync, t!("status-sync")),
        Space::new().width(Length::Fixed(6.0)),
        utc,
    ]
    .spacing(2)
    .align_y(Alignment::Center)
    .height(Length::Fixed(HEIGHT));

    container(bar)
        .padding([0, SIDE_PADDING])
        .width(Length::Fill)
        .height(Length::Fixed(HEIGHT))
        .align_y(Alignment::Center)
        .into()
}

/// One round-glyph indicator with a tooltip describing the underlying task.
/// Non-interactive for now — the visual state is driven by background workers.
fn indicator(icon: &'static str, state: ConnState, name: String) -> Element<'static, Message> {
    let color_fn = move |theme: &Theme| text::Style {
        color: Some(color(state, theme)),
    };

    let glyph = container(text(icon).size(ICON_SIZE).font(FONT_ICON).style(color_fn))
        .center_x(Length::Fixed(ICON_SLOT))
        .center_y(Length::Fixed(HEIGHT));

    let tip_text = format!("{name} — {}", label(state));

    tooltip(
        glyph,
        container(text(tip_text).size(12))
            .padding([4, 8])
            .style(tooltip_style),
        Position::Top,
    )
    .gap(4.0)
    .padding(0)
    .into()
}

fn color(state: ConnState, theme: &Theme) -> Color {
    let palette = theme.extended_palette();
    match state {
        ConnState::Disconnected => mute(palette.background.base.text),
        ConnState::Connecting => Color::from_rgb(0.95, 0.71, 0.18),
        ConnState::Connected => palette.success.strong.color,
        ConnState::Error => palette.danger.strong.color,
    }
}

fn label(state: ConnState) -> String {
    match state {
        ConnState::Disconnected => t!("status-state-disconnected"),
        ConnState::Connecting => t!("status-state-connecting"),
        ConnState::Connected => t!("status-state-connected"),
        ConnState::Error => t!("status-state-error"),
    }
}

fn mute(c: Color) -> Color {
    Color { a: 0.72, ..c }
}

fn muted_text(theme: &Theme) -> text::Style {
    text::Style {
        color: Some(mute(theme.extended_palette().background.base.text)),
    }
}

pub(crate) fn tooltip_style(theme: &Theme) -> container::Style {
    let palette = theme.extended_palette();
    let mut border = palette.background.strong.color;
    border.a = 0.6;
    container::Style {
        background: Some(Background::Color(palette.background.weak.color)),
        text_color: Some(palette.background.base.text),
        border: Border {
            color: border,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..container::Style::default()
    }
}

/// Top separator above the bar — thin subtle line, matching the rest of the
/// UI rules.
pub fn separator() -> Element<'static, Message> {
    rule::horizontal(1).style(title::rule_style).into()
}

/// Convenience used by the subscription to derive a fresh UTC timestamp from
/// the [`std::time::Instant`] tick. Kept here so the whole "what does a tick
/// mean" lives next to the view.
pub fn now() -> DateTime<Utc> {
    Utc::now()
}
