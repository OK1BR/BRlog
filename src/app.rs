use iced::window::{self, Settings as WindowSettings};
use iced::{Element, Font, Point, Size, Subscription, Task, Theme};

use crate::config::AppConfig;
use crate::db::Db;
use crate::models::qso::Qso;
use crate::theme::AppTheme;
use crate::ui;
use crate::ui::popup::{POPUP_PADDING, POPUP_ROW_HEIGHT, POPUP_WIDTH};

const INTER_BYTES: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
const MONO_BYTES: &[u8] = include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf");
const LUCIDE_BYTES: &[u8] = include_bytes!("../assets/fonts/lucide.ttf");
const APP_ICON_PNG: &[u8] = include_bytes!("../assets/icon.png");

fn app_icon() -> Option<window::Icon> {
    window::icon::from_file_data(APP_ICON_PNG, None).ok()
}

pub const FONT_UI: Font = Font::with_name("Inter");
pub const FONT_MONO: Font = Font::with_name("JetBrains Mono");
pub const FONT_ICON: Font = Font::with_name("lucide");

// Lucide icon codepoints — see assets/fonts/lucide.css for the full inventory.
pub const ICON_MINUS: &str = "\u{E11C}"; // minus
pub const ICON_MAXIMIZE: &str = "\u{E167}"; // square — classic Windows-style maximize outline
pub const ICON_RESTORE: &str = "\u{E09E}"; // copy — two overlapping squares (restore-down)
pub const ICON_X: &str = "\u{E1B2}"; // x
pub const ICON_LIST: &str = "\u{E106}"; // list
pub const ICON_SETTINGS: &str = "\u{E154}"; // settings (gear)

pub fn run() -> iced::Result {
    iced::daemon(App::title, App::update, App::view)
        .theme(App::theme)
        .subscription(App::subscription)
        .font(INTER_BYTES)
        .font(MONO_BYTES)
        .font(LUCIDE_BYTES)
        .default_font(FONT_UI)
        .run_with(App::new)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropdownKind {
    Band,
    Mode,
}

impl DropdownKind {
    /// Number of options in this dropdown — drives popup window height.
    pub fn item_count(self) -> usize {
        match self {
            DropdownKind::Band => Band::ALL.len(),
            DropdownKind::Mode => Mode::ALL.len(),
        }
    }

    /// Trigger button's left edge in main-window-local coordinates.
    /// Hand-derived from the entry_row layout in `ui::main` — keep in sync.
    pub fn trigger_x(self) -> f32 {
        const ROW_PADDING: f32 = 12.0;
        const CALLSIGN_W: f32 = 130.0;
        const SPACING: f32 = 8.0;
        const TRIGGER_W: f32 = 85.0;
        let band_x = ROW_PADDING + CALLSIGN_W + SPACING;
        match self {
            DropdownKind::Band => band_x,
            DropdownKind::Mode => band_x + TRIGGER_W + SPACING,
        }
    }
}

/// Bottom edge (in main-window-local coordinates) of the entry row's trigger
/// buttons. Title bar (32) + rule (1) + header (~53) + rule (1) + entry-row
/// padding (12) + trigger height (~24) = 123. Tweak if layout shifts.
pub const ENTRY_TRIGGER_BOTTOM: f32 = 123.0;
/// Vertical gap between trigger and popup window.
pub const POPUP_GAP: f32 = 4.0;

#[derive(Debug, Clone)]
pub enum Message {
    // Main window — entry row
    EntryCallsignChanged(String),
    EntryRstSentChanged(String),
    EntryRstRcvdChanged(String),
    EntryLocatorChanged(String),
    EntrySaveClicked,

    // Dropdown popup (Band / Mode in cramped main window)
    DropdownTriggerClicked(DropdownKind),
    DropdownAnchorReady(DropdownKind, Option<Point>),
    DropdownItemSelected(DropdownKind, usize),
    DropdownClose,

    // Window opening
    OpenLog,
    OpenSettings,

    // Settings draft mutations
    SettingsCallsignChanged(String),
    SettingsNameChanged(String),
    SettingsQthChanged(String),
    SettingsLocatorChanged(String),
    SettingsLicenseClassChanged(String),
    SettingsThemeChanged(AppTheme),
    SettingsWindowBorderChanged(bool),
    SettingsCancelClicked,
    SettingsSaveClicked,

    // Custom title bar actions
    WindowMinimize(window::Id),
    WindowMaximizeToggle(window::Id),
    WindowDrag(window::Id),
    WindowCloseRequested(window::Id),

    // Window lifecycle
    WindowClosed(window::Id),
    /// Any non-redraw window event — popup auto-close logic sits in `update`.
    WindowEvent(window::Id, window::Event),

    // Keyboard navigation
    TabPressed { shift: bool },
}

#[derive(Debug, Clone, Copy)]
pub struct PopupState {
    pub kind: DropdownKind,
    pub window_id: window::Id,
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
    /// Maximized state per window, tracked from custom title bar clicks.
    pub main_maximized: bool,
    pub log_maximized: bool,
    pub settings_maximized: bool,
    pub entry: EntryForm,
    /// Persisted app config (last loaded or saved value).
    pub config: AppConfig,
    /// Working copy edited inside the Settings window. Refreshed from `config` every time the window opens.
    pub settings_draft: AppConfig,
    pub db: Db,
    /// In-memory cache of all QSOs (sorted desc by datetime). Refreshed after every insert.
    pub qsos: Vec<Qso>,
    /// The currently open dropdown popup window, if any.
    pub popup: Option<PopupState>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let (id, open_task) = window::open(WindowSettings {
            size: Size::new(1000.0, 202.0),
            position: window::Position::Centered,
            min_size: Some(Size::new(850.0, 172.0)),
            decorations: false,
            icon: app_icon(),
            ..WindowSettings::default()
        });

        let config = AppConfig::load();

        let db = Db::open().expect("failed to open SQLite database");
        let qsos = db.list_qsos().unwrap_or_else(|e| {
            eprintln!("[db] list_qsos failed at startup, using empty list: {e:#}");
            Vec::new()
        });

        let app = Self {
            main_window: id,
            log_window: None,
            settings_window: None,
            main_maximized: false,
            log_maximized: false,
            settings_maximized: false,
            entry: EntryForm::default(),
            settings_draft: config.clone(),
            config,
            db,
            qsos,
            popup: None,
        };

        (app, open_task.discard())
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

    fn theme(&self, _window_id: window::Id) -> Theme {
        self.config.appearance.theme.to_iced()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::close_events().map(Message::WindowClosed),
            window::events().map(|(id, ev)| Message::WindowEvent(id, ev)),
            iced::keyboard::on_key_press(|key, modifiers| match key {
                iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab) => {
                    Some(Message::TabPressed {
                        shift: modifiers.shift(),
                    })
                }
                iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => {
                    Some(Message::DropdownClose)
                }
                _ => None,
            }),
        ])
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if let Some(popup) = self.popup
            && popup.window_id == window_id
        {
            ui::popup::view(self, popup)
        } else if Some(window_id) == self.settings_window {
            ui::settings::view(self, window_id)
        } else if Some(window_id) == self.log_window {
            ui::log::view(self, window_id)
        } else {
            ui::main::view(self, window_id)
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // --- Entry row ---
            Message::EntryCallsignChanged(s) => self.entry.callsign = s.to_uppercase(),
            Message::EntryRstSentChanged(s) => self.entry.rst_sent = s,
            Message::EntryRstRcvdChanged(s) => self.entry.rst_rcvd = s,
            Message::EntryLocatorChanged(s) => self.entry.locator = s.to_uppercase(),
            Message::EntrySaveClicked => {
                let callsign = self.entry.callsign.trim().to_string();
                if callsign.is_empty() {
                    return Task::none();
                }
                let qso = Qso::new_now(
                    callsign,
                    self.entry.band,
                    self.entry.mode,
                    self.entry.rst_sent.clone(),
                    self.entry.rst_rcvd.clone(),
                    self.entry.locator.trim().to_string(),
                );
                match self.db.insert_qso(&qso) {
                    Ok(_id) => match self.db.list_qsos() {
                        Ok(list) => {
                            self.qsos = list;
                            self.entry.callsign.clear();
                            self.entry.locator.clear();
                            self.entry.rst_rcvd.clear();
                        }
                        Err(e) => eprintln!("[db] list_qsos after insert failed: {e:#}"),
                    },
                    Err(e) => eprintln!("[db] insert_qso failed: {e:#}"),
                }
            }

            // --- Window opening ---
            Message::OpenLog => {
                if self.log_window.is_some() {
                    return Task::none();
                }
                let (id, task) = window::open(WindowSettings {
                    size: Size::new(1100.0, 632.0),
                    position: window::Position::Centered,
                    min_size: Some(Size::new(700.0, 432.0)),
                    decorations: false,
                    icon: app_icon(),
                    ..WindowSettings::default()
                });
                self.log_window = Some(id);
                return task.discard();
            }
            Message::OpenSettings => {
                if self.settings_window.is_some() {
                    return Task::none();
                }
                // Refresh draft from saved config so previous unsaved edits are dropped.
                self.settings_draft = self.config.clone();
                let (id, task) = window::open(WindowSettings {
                    size: Size::new(460.0, 512.0),
                    position: window::Position::Centered,
                    resizable: false,
                    decorations: false,
                    icon: app_icon(),
                    ..WindowSettings::default()
                });
                self.settings_window = Some(id);
                return task.discard();
            }

            // --- Settings draft mutations ---
            Message::SettingsCallsignChanged(s) => {
                self.settings_draft.operator.callsign = s.to_uppercase();
            }
            Message::SettingsNameChanged(s) => self.settings_draft.operator.name = s,
            Message::SettingsQthChanged(s) => self.settings_draft.operator.qth = s,
            Message::SettingsLocatorChanged(s) => {
                self.settings_draft.operator.locator = s.to_uppercase();
            }
            Message::SettingsLicenseClassChanged(s) => {
                self.settings_draft.operator.license_class = s
            }
            Message::SettingsThemeChanged(t) => self.settings_draft.appearance.theme = t,
            Message::SettingsWindowBorderChanged(b) => {
                self.settings_draft.appearance.window_border = b
            }
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

            // --- Custom title bar actions ---
            Message::WindowMinimize(id) => return window::minimize(id, true),
            Message::WindowMaximizeToggle(id) => {
                let new = !self.is_maximized(id);
                if id == self.main_window {
                    self.main_maximized = new;
                } else if Some(id) == self.log_window {
                    self.log_maximized = new;
                } else if Some(id) == self.settings_window {
                    self.settings_maximized = new;
                }
                return window::maximize(id, new);
            }
            Message::WindowDrag(id) => return window::drag(id),
            Message::WindowCloseRequested(id) => return window::close(id),

            // --- Keyboard navigation ---
            Message::TabPressed { shift } => {
                return if shift {
                    iced::widget::focus_previous()
                } else {
                    iced::widget::focus_next()
                };
            }

            // --- Dropdown popup (Band / Mode) ---
            Message::DropdownTriggerClicked(kind) => {
                if let Some(open) = self.popup {
                    // Toggle: clicking the same trigger closes; another kind closes
                    // the current popup, then we'll re-open after the close roundtrip.
                    let close = window::close(open.window_id);
                    self.popup = None;
                    if open.kind == kind {
                        return close;
                    }
                    return Task::batch([
                        close,
                        window::get_position(self.main_window)
                            .map(move |pos| Message::DropdownAnchorReady(kind, pos)),
                    ]);
                }
                return window::get_position(self.main_window)
                    .map(move |pos| Message::DropdownAnchorReady(kind, pos));
            }
            Message::DropdownAnchorReady(kind, anchor) => {
                let Some(origin) = anchor else {
                    return Task::none();
                };
                let position = Point::new(
                    origin.x + kind.trigger_x(),
                    origin.y + ENTRY_TRIGGER_BOTTOM + POPUP_GAP,
                );
                let height = kind.item_count() as f32 * POPUP_ROW_HEIGHT
                    + POPUP_PADDING * 2.0
                    + 2.0;
                let (id, task) = window::open(WindowSettings {
                    size: Size::new(POPUP_WIDTH, height),
                    position: window::Position::Specific(position),
                    decorations: false,
                    resizable: false,
                    level: window::Level::AlwaysOnTop,
                    exit_on_close_request: false,
                    ..WindowSettings::default()
                });
                self.popup = Some(PopupState { kind, window_id: id });
                return task.discard();
            }
            Message::DropdownItemSelected(kind, idx) => {
                match kind {
                    DropdownKind::Band => {
                        if let Some(b) = Band::ALL.get(idx).copied() {
                            self.entry.band = b;
                        }
                    }
                    DropdownKind::Mode => {
                        if let Some(m) = Mode::ALL.get(idx).copied() {
                            self.entry.mode = m;
                        }
                    }
                }
                if let Some(popup) = self.popup.take() {
                    return window::close(popup.window_id);
                }
            }
            Message::DropdownClose => {
                if let Some(popup) = self.popup.take() {
                    return window::close(popup.window_id);
                }
            }

            // --- Window lifecycle ---
            Message::WindowClosed(id) => {
                if id == self.main_window {
                    return iced::exit();
                }
                if Some(id) == self.settings_window {
                    self.settings_window = None;
                    self.settings_maximized = false;
                }
                if Some(id) == self.log_window {
                    self.log_window = None;
                    self.log_maximized = false;
                }
                if let Some(popup) = self.popup
                    && popup.window_id == id
                {
                    self.popup = None;
                }
            }
            Message::WindowEvent(id, ev) => {
                let popup_id = self.popup.map(|p| p.window_id);
                match ev {
                    window::Event::Unfocused if Some(id) == popup_id => {
                        if let Some(popup) = self.popup.take() {
                            return window::close(popup.window_id);
                        }
                    }
                    window::Event::Moved(_) | window::Event::Resized(_)
                        if id == self.main_window =>
                    {
                        if let Some(popup) = self.popup.take() {
                            return window::close(popup.window_id);
                        }
                    }
                    _ => {}
                }
            }
        }
        Task::none()
    }

    pub fn is_maximized(&self, window_id: window::Id) -> bool {
        if window_id == self.main_window {
            self.main_maximized
        } else if Some(window_id) == self.log_window {
            self.log_maximized
        } else if Some(window_id) == self.settings_window {
            self.settings_maximized
        } else {
            false
        }
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

impl std::str::FromStr for Band {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "160m" => Band::M160,
            "80m" => Band::M80,
            "40m" => Band::M40,
            "30m" => Band::M30,
            "20m" => Band::M20,
            "17m" => Band::M17,
            "15m" => Band::M15,
            "12m" => Band::M12,
            "10m" => Band::M10,
            "6m" => Band::M6,
            "4m" => Band::M4,
            "2m" => Band::M2,
            "70cm" => Band::Cm70,
            _ => return Err(()),
        })
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

impl std::str::FromStr for Mode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "SSB" => Mode::Ssb,
            "CW" => Mode::Cw,
            "FT8" => Mode::Ft8,
            "FT4" => Mode::Ft4,
            "RTTY" => Mode::Rtty,
            "PSK" => Mode::Psk,
            "AM" => Mode::Am,
            "FM" => Mode::Fm,
            _ => return Err(()),
        })
    }
}
