use iced::window::{self, Settings as WindowSettings};
use iced::{Element, Size, Subscription, Task, Theme};

use crate::config::OperatorConfig;
use crate::ui;

pub fn run() -> iced::Result {
    iced::daemon(App::title, App::update, App::view)
        .theme(|_, _| Theme::Dark)
        .subscription(App::subscription)
        .run_with(App::new)
}

#[derive(Debug, Clone)]
pub enum Message {
    // Main window — entry row
    EntryCallsignChanged(String),
    EntryBandChanged(Band),
    EntryModeChanged(Mode),
    EntryRstSentChanged(String),
    EntryRstRcvdChanged(String),
    EntryLocatorChanged(String),
    EntrySaveClicked,

    // Window opening
    OpenLog,
    OpenSettings,

    // Settings draft mutations
    SettingsCallsignChanged(String),
    SettingsNameChanged(String),
    SettingsQthChanged(String),
    SettingsLocatorChanged(String),
    SettingsLicenseClassChanged(String),
    SettingsCancelClicked,
    SettingsSaveClicked,

    // Window lifecycle
    WindowOpened(window::Id),
    WindowClosed(window::Id),
}

#[derive(Default)]
pub struct EntryForm {
    pub callsign: String,
    pub band: Band,
    pub mode: Mode,
    pub rst_sent: String,
    pub rst_rcvd: String,
    pub locator: String,
}

pub struct App {
    pub main_window: window::Id,
    pub log_window: Option<window::Id>,
    pub settings_window: Option<window::Id>,
    pub entry: EntryForm,
    /// Persisted operator config (last loaded or saved value).
    pub config: OperatorConfig,
    /// Working copy edited inside the Settings window. Refreshed from `config` every time the window opens.
    pub settings_draft: OperatorConfig,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let (id, open_task) = window::open(WindowSettings {
            size: Size::new(1000.0, 170.0),
            position: window::Position::Centered,
            min_size: Some(Size::new(850.0, 140.0)),
            ..WindowSettings::default()
        });

        let config = OperatorConfig::load();

        let app = Self {
            main_window: id,
            log_window: None,
            settings_window: None,
            entry: EntryForm::default(),
            settings_draft: config.clone(),
            config,
        };

        (app, open_task.map(Message::WindowOpened))
    }

    fn title(&self, window_id: window::Id) -> String {
        if Some(window_id) == self.settings_window {
            "BRlog — Nastavení".into()
        } else if Some(window_id) == self.log_window {
            "BRlog — Deník".into()
        } else {
            "BRlog".into()
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        window::close_events().map(Message::WindowClosed)
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if Some(window_id) == self.settings_window {
            ui::settings_window::view(self)
        } else if Some(window_id) == self.log_window {
            ui::log_window::view(self)
        } else {
            ui::main_window::view(self)
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // --- Entry row ---
            Message::EntryCallsignChanged(s) => self.entry.callsign = s.to_uppercase(),
            Message::EntryBandChanged(b) => self.entry.band = b,
            Message::EntryModeChanged(m) => self.entry.mode = m,
            Message::EntryRstSentChanged(s) => self.entry.rst_sent = s,
            Message::EntryRstRcvdChanged(s) => self.entry.rst_rcvd = s,
            Message::EntryLocatorChanged(s) => self.entry.locator = s.to_uppercase(),
            Message::EntrySaveClicked => {
                // TODO: persist QSO once DB exists
            }

            // --- Window opening ---
            Message::OpenLog => {
                if self.log_window.is_some() {
                    return Task::none();
                }
                let (id, task) = window::open(WindowSettings {
                    size: Size::new(1100.0, 600.0),
                    position: window::Position::Centered,
                    min_size: Some(Size::new(700.0, 400.0)),
                    ..WindowSettings::default()
                });
                self.log_window = Some(id);
                return task.map(Message::WindowOpened);
            }
            Message::OpenSettings => {
                if self.settings_window.is_some() {
                    return Task::none();
                }
                // Refresh draft from saved config so previous unsaved edits are dropped.
                self.settings_draft = self.config.clone();
                let (id, task) = window::open(WindowSettings {
                    size: Size::new(440.0, 380.0),
                    position: window::Position::Centered,
                    resizable: false,
                    ..WindowSettings::default()
                });
                self.settings_window = Some(id);
                return task.map(Message::WindowOpened);
            }

            // --- Settings draft mutations ---
            Message::SettingsCallsignChanged(s) => {
                self.settings_draft.callsign = s.to_uppercase();
            }
            Message::SettingsNameChanged(s) => self.settings_draft.name = s,
            Message::SettingsQthChanged(s) => self.settings_draft.qth = s,
            Message::SettingsLocatorChanged(s) => {
                self.settings_draft.locator = s.to_uppercase();
            }
            Message::SettingsLicenseClassChanged(s) => self.settings_draft.license_class = s,
            Message::SettingsCancelClicked => {
                // Draft is recreated next OpenSettings; just close.
                if let Some(id) = self.settings_window {
                    return window::close(id);
                }
            }
            Message::SettingsSaveClicked => {
                self.config = self.settings_draft.clone();
                if let Err(e) = self.config.save() {
                    eprintln!("[config] save failed: {e:#}");
                    // Keep window open so user can retry; no UI error display yet.
                    return Task::none();
                }
                if let Some(id) = self.settings_window {
                    return window::close(id);
                }
            }

            // --- Window lifecycle ---
            Message::WindowOpened(_id) => {}
            Message::WindowClosed(id) => {
                if id == self.main_window {
                    return iced::exit();
                }
                if Some(id) == self.settings_window {
                    self.settings_window = None;
                }
                if Some(id) == self.log_window {
                    self.log_window = None;
                }
            }
        }
        Task::none()
    }
}

// --- Domain enums ---

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Band {
    M160,
    #[default]
    M80,
    M40,
    M30,
    M20,
    M17,
    M15,
    M12,
    M10,
    M6,
    M4,
    M2,
    Cm70,
}

impl Band {
    pub const ALL: &'static [Band] = &[
        Band::M160,
        Band::M80,
        Band::M40,
        Band::M30,
        Band::M20,
        Band::M17,
        Band::M15,
        Band::M12,
        Band::M10,
        Band::M6,
        Band::M4,
        Band::M2,
        Band::Cm70,
    ];
}

impl std::fmt::Display for Band {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Band::M160 => "160m",
            Band::M80 => "80m",
            Band::M40 => "40m",
            Band::M30 => "30m",
            Band::M20 => "20m",
            Band::M17 => "17m",
            Band::M15 => "15m",
            Band::M12 => "12m",
            Band::M10 => "10m",
            Band::M6 => "6m",
            Band::M4 => "4m",
            Band::M2 => "2m",
            Band::Cm70 => "70cm",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Ssb,
    Cw,
    Ft8,
    Ft4,
    Rtty,
    Psk,
    Am,
    Fm,
}

impl Mode {
    pub const ALL: &'static [Mode] = &[
        Mode::Ssb,
        Mode::Cw,
        Mode::Ft8,
        Mode::Ft4,
        Mode::Rtty,
        Mode::Psk,
        Mode::Am,
        Mode::Fm,
    ];
}

impl std::fmt::Display for Mode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Mode::Ssb => "SSB",
            Mode::Cw => "CW",
            Mode::Ft8 => "FT8",
            Mode::Ft4 => "FT4",
            Mode::Rtty => "RTTY",
            Mode::Psk => "PSK",
            Mode::Am => "AM",
            Mode::Fm => "FM",
        };
        f.write_str(s)
    }
}
