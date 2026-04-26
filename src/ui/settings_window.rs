use iced::widget::{button, column, container, row, text, text_input, Space};
use iced::{Alignment, Element, Length};

use crate::app::{App, Message};

pub fn view(state: &App) -> Element<'_, Message> {
    container(
        column![
            text("Nastavení operátora").size(20),
            Space::with_height(Length::Fixed(8.0)),
            field(
                "Volačka",
                &state.settings_draft.callsign,
                Message::SettingsCallsignChanged
            ),
            field("Jméno", &state.settings_draft.name, Message::SettingsNameChanged),
            field("QTH", &state.settings_draft.qth, Message::SettingsQthChanged),
            field(
                "Lokátor",
                &state.settings_draft.locator,
                Message::SettingsLocatorChanged
            ),
            field(
                "Licenční třída",
                &state.settings_draft.license_class,
                Message::SettingsLicenseClassChanged
            ),
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
        .spacing(10),
    )
    .padding(20)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn field<'a>(
    label: &'a str,
    value: &'a str,
    on_change: fn(String) -> Message,
) -> Element<'a, Message> {
    row![
        text(label).width(Length::Fixed(120.0)),
        text_input("", value).on_input(on_change),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}
