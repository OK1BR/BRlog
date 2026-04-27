use std::sync::{Arc, LazyLock};

use iced::theme::{Custom, Palette};
use iced::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AppTheme {
    // Dark
    #[default]
    TokyoNight,
    TokyoNightStorm,
    Dracula,
    Nord,
    GruvboxDark,
    SolarizedDark,
    CatppuccinFrappe,
    CatppuccinMacchiato,
    CatppuccinMocha,
    KanagawaWave,
    KanagawaDragon,
    Moonfly,
    Nightfly,
    Oxocarbon,
    Ferra,
    Dark,
    FullBlack,
    // Light
    Light,
    SolarizedLight,
    GruvboxLight,
    CatppuccinLatte,
    TokyoNightLight,
    KanagawaLotus,
}

impl AppTheme {
    pub const ALL: &'static [AppTheme] = &[
        // Dark
        AppTheme::TokyoNight,
        AppTheme::TokyoNightStorm,
        AppTheme::Dracula,
        AppTheme::Nord,
        AppTheme::GruvboxDark,
        AppTheme::SolarizedDark,
        AppTheme::CatppuccinFrappe,
        AppTheme::CatppuccinMacchiato,
        AppTheme::CatppuccinMocha,
        AppTheme::KanagawaWave,
        AppTheme::KanagawaDragon,
        AppTheme::Moonfly,
        AppTheme::Nightfly,
        AppTheme::Oxocarbon,
        AppTheme::Ferra,
        AppTheme::Dark,
        AppTheme::FullBlack,
        // Light
        AppTheme::Light,
        AppTheme::SolarizedLight,
        AppTheme::GruvboxLight,
        AppTheme::CatppuccinLatte,
        AppTheme::TokyoNightLight,
        AppTheme::KanagawaLotus,
    ];

    pub fn to_iced(self) -> iced::Theme {
        match self {
            AppTheme::TokyoNight => iced::Theme::TokyoNight,
            AppTheme::TokyoNightStorm => iced::Theme::TokyoNightStorm,
            AppTheme::TokyoNightLight => iced::Theme::TokyoNightLight,
            AppTheme::Dracula => iced::Theme::Dracula,
            AppTheme::Nord => iced::Theme::Nord,
            AppTheme::GruvboxDark => iced::Theme::GruvboxDark,
            AppTheme::GruvboxLight => iced::Theme::GruvboxLight,
            AppTheme::SolarizedDark => iced::Theme::SolarizedDark,
            AppTheme::SolarizedLight => iced::Theme::SolarizedLight,
            AppTheme::CatppuccinLatte => iced::Theme::CatppuccinLatte,
            AppTheme::CatppuccinFrappe => iced::Theme::CatppuccinFrappe,
            AppTheme::CatppuccinMacchiato => iced::Theme::CatppuccinMacchiato,
            AppTheme::CatppuccinMocha => iced::Theme::CatppuccinMocha,
            AppTheme::KanagawaWave => iced::Theme::KanagawaWave,
            AppTheme::KanagawaDragon => iced::Theme::KanagawaDragon,
            AppTheme::KanagawaLotus => iced::Theme::KanagawaLotus,
            AppTheme::Moonfly => iced::Theme::Moonfly,
            AppTheme::Nightfly => iced::Theme::Nightfly,
            AppTheme::Oxocarbon => iced::Theme::Oxocarbon,
            AppTheme::Ferra => iced::Theme::Ferra,
            AppTheme::Dark => iced::Theme::Dark,
            AppTheme::Light => iced::Theme::Light,
            AppTheme::FullBlack => iced::Theme::Custom(FULL_BLACK.clone()),
        }
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AppTheme::TokyoNight => "Tokyo Night",
            AppTheme::TokyoNightStorm => "Tokyo Night Storm",
            AppTheme::TokyoNightLight => "Tokyo Night Light",
            AppTheme::Dracula => "Dracula",
            AppTheme::Nord => "Nord",
            AppTheme::GruvboxDark => "Gruvbox Dark",
            AppTheme::GruvboxLight => "Gruvbox Light",
            AppTheme::SolarizedDark => "Solarized Dark",
            AppTheme::SolarizedLight => "Solarized Light",
            AppTheme::CatppuccinLatte => "Catppuccin Latte",
            AppTheme::CatppuccinFrappe => "Catppuccin Frappé",
            AppTheme::CatppuccinMacchiato => "Catppuccin Macchiato",
            AppTheme::CatppuccinMocha => "Catppuccin Mocha",
            AppTheme::KanagawaWave => "Kanagawa Wave",
            AppTheme::KanagawaDragon => "Kanagawa Dragon",
            AppTheme::KanagawaLotus => "Kanagawa Lotus",
            AppTheme::Moonfly => "Moonfly",
            AppTheme::Nightfly => "Nightfly",
            AppTheme::Oxocarbon => "Oxocarbon",
            AppTheme::Ferra => "Ferra",
            AppTheme::Dark => "Dark",
            AppTheme::Light => "Light",
            AppTheme::FullBlack => "Full Black",
        };
        f.write_str(s)
    }
}

// Pure-black surface with Ayu Dark accent colors for text and primary highlights.
static FULL_BLACK: LazyLock<Arc<Custom>> = LazyLock::new(|| {
    Arc::new(Custom::new(
        "Full Black".to_string(),
        Palette {
            background: Color::from_rgb8(0, 0, 0),
            text: Color::from_rgb8(213, 213, 213),
            primary: Color::from_rgb8(255, 143, 64),
            success: Color::from_rgb8(170, 217, 76),
            danger: Color::from_rgb8(240, 113, 120),
        },
    ))
});
