use iced::widget::{Space, button, column, container, pick_list, row, text, text_input};
use iced::window;
use iced::{Alignment, Element, Length};

use crate::app::{App, Message, FONT_MONO};
use crate::theme::AppTheme;
use crate::ui::title_bar;

pub fn view<'a>(state: &'a App, window_id: window::Id) -> Element<'a, Message> {
    column![
        title_bar::view(window_id, "BRlog — Nastavení", state.is_maximized(window_id)),
        settings_body(state),
    ]
    .spacing(0)
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
            Space::with_height(Length::Fill),
            row![
                Space::with_width(Length::Fill),
                button(text("Zrušit"))
                    .on_press(Message::SettingsCancelClicked)
                    .style(button::secondary),
                button(text("Uložit"))
                    .on_press(Message::SettingsSaveClicked)
                    .style(button::primary),
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
        text_input("", value).on_input(on_change).font(FONT_MONO),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn theme_row(state: &App) -> Element<'_, Message> {
    row![
        text("Téma").width(Length::Fixed(120.0)),
        pick_list(
            AppTheme::ALL,
            Some(state.settings_draft.appearance.theme),
            Message::SettingsThemeChanged,
        )
        .width(Length::Fill),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}
