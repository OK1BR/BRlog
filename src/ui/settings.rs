use iced::widget::{checkbox, column, container, row, rule, text, Space};
use iced::window;
use iced::{Alignment, Element, Length};

use crate::app::{App, Message, FONT_MONO};
use crate::config::Language;
use crate::t;
use crate::theme::AppTheme;
use crate::ui::buttons::{outlined, solid};
use crate::ui::inputs::{dropdown, input};
use crate::ui::title;

pub fn view<'a>(state: &'a App, window_id: window::Id) -> Element<'a, Message> {
    container(
        column![
            title::view(
                window_id,
                t!("window-title-settings"),
                state.is_maximized(window_id),
                false,
            ),
            rule::horizontal(1).style(title::rule_style),
            settings_body(state),
        ]
        .spacing(0),
    )
    .style(title::window_border(
        state.config.appearance.window_border && !state.is_maximized(window_id),
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn settings_body(state: &App) -> Element<'_, Message> {
    container(
        column![
            section_label(t!("section-operator")),
            field(
                t!("field-callsign"),
                &state.settings_draft.operator.callsign,
                Message::SettingsCallsignChanged
            ),
            field(
                t!("field-name"),
                &state.settings_draft.operator.name,
                Message::SettingsNameChanged
            ),
            field(
                t!("field-qth"),
                &state.settings_draft.operator.qth,
                Message::SettingsQthChanged
            ),
            field(
                t!("field-locator"),
                &state.settings_draft.operator.locator,
                Message::SettingsLocatorChanged
            ),
            field(
                t!("field-license-class"),
                &state.settings_draft.operator.license_class,
                Message::SettingsLicenseClassChanged
            ),
            Space::new().height(Length::Fixed(12.0)),
            section_label(t!("section-appearance")),
            theme_row(state),
            border_row(state),
            language_row(state),
            Space::new().height(Length::Fill),
            row![
                Space::new().width(Length::Fill),
                outlined(text(t!("button-cancel")).size(14))
                    .on_press(Message::SettingsCancelClicked),
                solid(text(t!("button-save")).size(14)).on_press(Message::SettingsSaveClicked),
            ]
            .spacing(8),
        ]
        .spacing(8),
    )
    .padding(20)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn section_label(label: String) -> Element<'static, Message> {
    text(label).size(16).into()
}

fn field<'a>(
    label: String,
    value: &'a str,
    on_change: fn(String) -> Message,
) -> Element<'a, Message> {
    row![
        text(label).size(14).width(Length::Fixed(120.0)),
        input("", value).on_input(on_change).font(FONT_MONO),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn theme_row(state: &App) -> Element<'_, Message> {
    row![
        text(t!("setting-theme"))
            .size(14)
            .width(Length::Fixed(120.0)),
        dropdown(
            AppTheme::ALL,
            Some(state.settings_draft.appearance.theme),
            Message::SettingsThemeChanged,
            Length::Fill,
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn border_row(state: &App) -> Element<'_, Message> {
    row![
        text(t!("setting-window-border"))
            .size(14)
            .width(Length::Fixed(120.0)),
        checkbox(state.settings_draft.appearance.window_border)
            .on_toggle(Message::SettingsWindowBorderChanged),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn language_row(state: &App) -> Element<'_, Message> {
    row![
        text(t!("setting-language"))
            .size(14)
            .width(Length::Fixed(120.0)),
        dropdown(
            Language::ALL,
            Some(state.settings_draft.appearance.language),
            Message::SettingsLanguageChanged,
            Length::Fill,
        ),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}
