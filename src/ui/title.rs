//! Custom title bar — visual proportions follow Zed's `platform_title_bar` crate.
//!
//! References from Zed source (crates/platform_title_bar/src/platforms/platform_windows.rs
//! and crates/ui/src/utils/constants.rs):
//!   - bar height: 32 px (Windows), 34 px (other)
//!   - caption button width: 36 px
//!   - close hover: rgb(232, 17, 32)  (official Microsoft red)
//!   - close pressed: same color at 0.8 alpha + white text at 0.8
//!   - other hover: theme.colors.ghost_element_hover  (subtle)

use iced::widget::{Space, button, container, mouse_area, row, rule, text};
use iced::window;
use iced::{Alignment, Background, Border, Color, Element, Font, Length, Shadow, Theme};

use crate::app::{FONT_ICON, FONT_UI, ICON_LIST, ICON_SETTINGS, Message};

const HEIGHT: f32 = 32.0;
const CTRL_WIDTH: f32 = 36.0;
const ACTION_WIDTH: f32 = 36.0;
const LIGHT_SIZE: f32 = 12.0;
const LIGHT_SPACING: f32 = 8.0;
const SIDE_PADDING: f32 = 8.0;

// ── Window-control glyphs ──────────────────────────────────────────────
// Zed renders caption buttons with `Segoe Fluent Icons` (Windows 11) or
// `Segoe MDL2 Assets` (Windows 10) at 10 px — see Zed
// `crates/platform_title_bar/src/platforms/platform_windows.rs`.
// On macOS the buttons are traffic lights, so this only matters on Windows
// / Linux. On Linux we fall back to the bundled lucide font.
#[cfg(target_os = "windows")]
const CHROME_FONT: Font = Font::with_name("Segoe Fluent Icons");
#[cfg(not(target_os = "windows"))]
const CHROME_FONT: Font = FONT_ICON;

#[cfg(target_os = "windows")]
const CHROME_ICON_SIZE: f32 = 10.0;
#[cfg(not(target_os = "windows"))]
const CHROME_ICON_SIZE: f32 = 12.0;

#[cfg(target_os = "windows")]
const CHROME_MINIMIZE: &str = "\u{E921}";
#[cfg(target_os = "windows")]
const CHROME_MAXIMIZE: &str = "\u{E922}";
#[cfg(target_os = "windows")]
const CHROME_RESTORE: &str = "\u{E923}";
#[cfg(target_os = "windows")]
const CHROME_CLOSE: &str = "\u{E8BB}";

#[cfg(not(target_os = "windows"))]
const CHROME_MINIMIZE: &str = crate::app::ICON_MINUS;
#[cfg(not(target_os = "windows"))]
const CHROME_MAXIMIZE: &str = crate::app::ICON_MAXIMIZE;
#[cfg(not(target_os = "windows"))]
const CHROME_RESTORE: &str = crate::app::ICON_RESTORE;
#[cfg(not(target_os = "windows"))]
const CHROME_CLOSE: &str = crate::app::ICON_X;

const CLOSE_HOVER: Color = Color {
    r: 232.0 / 255.0,
    g: 17.0 / 255.0,
    b: 32.0 / 255.0,
    a: 1.0,
};
const CLOSE_PRESSED: Color = Color {
    r: 186.0 / 255.0,
    g: 14.0 / 255.0,
    b: 26.0 / 255.0,
    a: 1.0,
};

pub fn view(
    window_id: window::Id,
    title: String,
    is_maximized: bool,
    show_actions: bool,
) -> Element<'static, Message> {
    if cfg!(target_os = "macos") {
        macos_layout(window_id, title, show_actions)
    } else {
        windows_layout(window_id, title, is_maximized, show_actions)
    }
}

fn windows_layout(
    window_id: window::Id,
    title: String,
    is_maximized: bool,
    show_actions: bool,
) -> Element<'static, Message> {
    let max_icon = if is_maximized {
        CHROME_RESTORE
    } else {
        CHROME_MAXIMIZE
    };

    let mut bar = row![]
        .height(Length::Fixed(HEIGHT))
        .align_y(Alignment::Center);

    // Hamburger / settings on the left, Zed-style.
    if show_actions {
        bar = bar
            .push(action_button(ICON_LIST, Message::OpenLog))
            .push(action_button(ICON_SETTINGS, Message::OpenSettings));
    }

    // Drag area with title — fills the remaining space between actions and controls.
    bar = bar.push(
        mouse_area(
            container(text(title).size(12).font(FONT_UI).style(muted_text))
                .padding([0, if show_actions { 8 } else { SIDE_PADDING as u16 }])
                .center_y(Length::Fixed(HEIGHT))
                .width(Length::Fill)
                .height(Length::Fixed(HEIGHT)),
        )
        .on_press(Message::WindowDrag(window_id)),
    );

    bar = bar
        .push(ctrl_button(
            CHROME_MINIMIZE,
            Message::WindowMinimize(window_id),
            false,
        ))
        .push(ctrl_button(
            max_icon,
            Message::WindowMaximizeToggle(window_id),
            false,
        ))
        .push(ctrl_button(
            CHROME_CLOSE,
            Message::WindowCloseRequested(window_id),
            true,
        ));

    container(bar)
        .width(Length::Fill)
        .height(Length::Fixed(HEIGHT))
        .into()
}

fn macos_layout(
    window_id: window::Id,
    title: String,
    show_actions: bool,
) -> Element<'static, Message> {
    let mut bar = row![
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
    ]
    .spacing(LIGHT_SPACING)
    .padding([0, 12])
    .height(Length::Fixed(HEIGHT))
    .align_y(Alignment::Center);

    bar = bar.push(
        mouse_area(
            container(text(title).size(12).font(FONT_UI).style(muted_text))
                .center_x(Length::Fill)
                .center_y(Length::Fixed(HEIGHT))
                .width(Length::Fill)
                .height(Length::Fixed(HEIGHT)),
        )
        .on_press(Message::WindowDrag(window_id)),
    );

    if show_actions {
        bar = bar
            .push(action_button(ICON_LIST, Message::OpenLog))
            .push(action_button(ICON_SETTINGS, Message::OpenSettings));
    }

    container(bar)
        .width(Length::Fill)
        .height(Length::Fixed(HEIGHT))
        .into()
}

fn action_button(icon: &'static str, msg: Message) -> Element<'static, Message> {
    button(
        container(text(icon).size(14).font(FONT_ICON))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .on_press(msg)
    .style(|theme: &Theme, status: button::Status| {
        let palette = theme.extended_palette();
        let (bg, fg) = match status {
            button::Status::Hovered => (Some(ghost_hover(theme)), palette.background.base.text),
            button::Status::Pressed => (Some(ghost_active(theme)), palette.background.base.text),
            _ => (None, mute(palette.background.base.text)),
        };
        button::Style {
            background: bg.map(Background::Color),
            text_color: fg,
            border: Border::default(),
            shadow: Shadow::default(),
            ..button::Style::default()
        }
    })
    .width(Length::Fixed(ACTION_WIDTH))
    .height(Length::Fixed(HEIGHT))
    .padding(0)
    .into()
}

fn ctrl_button(icon: &'static str, msg: Message, is_close: bool) -> Element<'static, Message> {
    button(
        container(text(icon).size(CHROME_ICON_SIZE).font(CHROME_FONT))
            .center_x(Length::Fill)
            .center_y(Length::Fill),
    )
    .on_press(msg)
    .style(move |theme: &Theme, status: button::Status| {
        let palette = theme.extended_palette();
        let (bg, fg) = match status {
            button::Status::Hovered if is_close => (Some(CLOSE_HOVER), Color::WHITE),
            button::Status::Pressed if is_close => (
                Some(CLOSE_PRESSED),
                Color {
                    a: 0.85,
                    ..Color::WHITE
                },
            ),
            button::Status::Hovered => (Some(ghost_hover(theme)), palette.background.base.text),
            button::Status::Pressed => (Some(ghost_active(theme)), palette.background.base.text),
            _ => (None, mute(palette.background.base.text)),
        };
        button::Style {
            background: bg.map(Background::Color),
            text_color: fg,
            border: Border::default(),
            shadow: Shadow::default(),
            ..button::Style::default()
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

/// Zed `ghost_element_hover` equivalent — subtle overlay (~7% of foreground over background).
fn ghost_hover(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    let base = p.background.base.text;
    Color {
        a: if p.is_dark { 0.07 } else { 0.05 },
        ..base
    }
}

/// Zed `ghost_element_active` equivalent — slightly stronger than hover.
fn ghost_active(theme: &Theme) -> Color {
    let p = theme.extended_palette();
    let base = p.background.base.text;
    Color {
        a: if p.is_dark { 0.12 } else { 0.09 },
        ..base
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
        radius: 0.0.into(),
        fill_mode: rule::FillMode::Full,
        snap: true,
    }
}

fn light_button(color: Color, msg: Message) -> Element<'static, Message> {
    button(
        Space::new()
            .width(Length::Fixed(LIGHT_SIZE))
            .height(Length::Fixed(LIGHT_SIZE)),
    )
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
        ..button::Style::default()
    })
    .into()
}
