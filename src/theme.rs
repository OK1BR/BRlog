use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AppTheme {
    #[default]
    TokyoNight,
    Dracula,
    Nord,
    GruvboxDark,
    CatppuccinMocha,
    Light,
}

impl AppTheme {
    pub const ALL: &'static [AppTheme] = &[
        AppTheme::TokyoNight,
        AppTheme::Dracula,
        AppTheme::Nord,
        AppTheme::GruvboxDark,
        AppTheme::CatppuccinMocha,
        AppTheme::Light,
    ];

    pub fn to_iced(self) -> iced::Theme {
        match self {
            AppTheme::TokyoNight => iced::Theme::TokyoNight,
            AppTheme::Dracula => iced::Theme::Dracula,
            AppTheme::Nord => iced::Theme::Nord,
            AppTheme::GruvboxDark => iced::Theme::GruvboxDark,
            AppTheme::CatppuccinMocha => iced::Theme::CatppuccinMocha,
            AppTheme::Light => iced::Theme::Light,
        }
    }
}

impl std::fmt::Display for AppTheme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            AppTheme::TokyoNight => "Tokyo Night",
            AppTheme::Dracula => "Dracula",
            AppTheme::Nord => "Nord",
            AppTheme::GruvboxDark => "Gruvbox Dark",
            AppTheme::CatppuccinMocha => "Catppuccin Mocha",
            AppTheme::Light => "Light",
        };
        f.write_str(s)
    }
}
