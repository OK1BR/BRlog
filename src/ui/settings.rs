use iced::widget::{checkbox, column, container, horizontal_rule, row, text, Space};
use iced::window;
use iced::{Alignment, Element, Length};

use crate::app::{App, Message, FONT_MONO};
use crate::theme::AppTheme;
use crate::ui::buttons::{outlined, solid};
use crate::ui::inputs::{dropdown, input};
use crate::ui::title_bar;

pub fn view<'a>(state: &'a App, window_id: window::Id) -> Element<'a, Message> {
    container(
        column![
            title_bar::view(
                window_id,
                "BRlog — Nastavení",
                state.is_maximized(window_id),
                false,
            ),
            horizontal_rule(1).style(title_bar::rule_style),
            settings_body(state),
        ]
        .spacing(0),
    )
    .style(title_bar::window_border(state.config.appearance.window_border))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn settings_body(state: &App) -> Element<'_, Message> {
    container(
        column![
            section_label("Operátor"),
            field(
                "Volačka",
                &state.settings_draft.operator.callsign,
                Message::SettingsCallsignChanged
            ),
            field(
                "Jméno",
                &state.settings_draft.operator.name,
                Message::SettingsNameChanged
            ),
            field(
                "QTH",
                &state.settings_draft.operator.qth,
                Message::SettingsQthChanged
            ),
            field(
                "Lokátor",
                &state.settings_draft.operator.locator,
                Message::SettingsLocatorChanged
            ),
            field(
                "Licenční třída",
                &state.settings_draft.operator.license_class,
                Message::SettingsLicenseClassChanged
            ),
            Space::with_height(Length::Fixed(12.0)),
            section_label("Vzhled"),
            theme_row(state),
            border_row(state),
            Space::with_height(Length::Fill),
            row![
                Space::with_width(Length::Fill),
                outlined(text("Zrušit").size(14)).on_press(Message::SettingsCancelClicked),
                solid(text("Uložit").size(14)).on_press(Message::SettingsSaveClicked),
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

fn section_label(label: &str) -> Element<'_, Message> {
    text(label).size(16).into()
}

fn field<'a>(
    label: &'a str,
    value: &'a str,
    on_change: fn(String) -> Message,
) -> Element<'a, Message> {
    row![
        text(label).width(Length::Fixed(120.0)),
        input("", value).on_input(on_change).font(FONT_MONO),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn theme_row(state: &App) -> Element<'_, Message> {
    row![
        text("Téma").width(Length::Fixed(120.0)),
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
        text("Ohraničení okna").width(Length::Fixed(120.0)),
        checkbox("", state.settings_draft.appearance.window_border)
            .on_toggle(Message::SettingsWindowBorderChanged),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}
