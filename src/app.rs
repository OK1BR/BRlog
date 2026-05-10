use std::time::Duration;

use chrono::{DateTime, Utc};
use iced::window::{self, Settings as WindowSettings};
use iced::{Element, Font, Point, Size, Subscription, Task, Theme};

use crate::config::{AppConfig, Language};
use crate::db::Db;
use crate::i18n;
use crate::models::qso::Qso;
use crate::t;
use crate::theme::AppTheme;
use crate::ui;
use crate::ui::bar::{self, BackgroundStatus};
use crate::ui::settings::SettingsPage;

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

// Lucide icon codepoints — see https://lucide.dev for the full inventory.
// The window-control glyphs (minus/maximize/restore/x) are only used as a
// fallback on non-Windows platforms; on Windows the title bar uses native
// `Segoe Fluent Icons`, so these read as dead code on a Windows build.
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub const ICON_MINUS: &str = "\u{E11C}"; // minus
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub const ICON_MAXIMIZE: &str = "\u{E167}"; // square — classic Windows-style maximize outline
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub const ICON_RESTORE: &str = "\u{E09E}"; // copy — two overlapping squares (restore-down)
#[cfg_attr(target_os = "windows", allow(dead_code))]
pub const ICON_X: &str = "\u{E1B2}"; // x
pub const ICON_LIST: &str = "\u{E106}"; // list
pub const ICON_SETTINGS: &str = "\u{E154}"; // settings (gear)

pub fn run() -> iced::Result {
    iced::daemon(App::new, App::update, App::view)
        .title(App::title)
        .theme(App::theme)
        .subscription(App::subscription)
        .font(INTER_BYTES)
        .font(MONO_BYTES)
        .font(LUCIDE_BYTES)
        .default_font(FONT_UI)
        .run()
}

#[derive(Debug, Clone)]
pub enum Message {
    // Main window — entry row
    EntryCallsignChanged(String),
    EntryRstSentChanged(String),
    EntryRstRcvdChanged(String),
    EntryLocatorChanged(String),
    EntrySaveClicked,

    MacroPressed(u8),

    // Log window — row selection + context menu
    QsoSelected(i64),
    QsoContextMenu(i64),
    LogCursorMoved(Point),
    ContextMenuDismiss,
    DeleteQsoConfirmed(i64),

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
    SettingsLanguageChanged(Language),
    SettingsCategoryChanged(SettingsPage),
    SettingsSearchChanged(String),
    SettingsCancelClicked,
    SettingsSaveClicked,

    // Custom title bar actions
    WindowMinimize(window::Id),
    WindowMaximizeToggle(window::Id),
    WindowDrag(window::Id),
    WindowDragResize(window::Id, window::Direction),
    WindowCloseRequested(window::Id),

    // Window lifecycle
    WindowClosed(window::Id),

    // Keyboard navigation
    TabPressed { shift: bool },

    // Status bar — periodic UTC tick (1 Hz) used to repaint the clock.
    Tick,
}

pub struct ContextMenuState {
    pub qso_id: i64,
    /// Anchor point inside the Log window body, in body-local coordinates.
    pub position: Point,
}

pub struct EntryForm {
    pub callsign: String,
    /// Read-only in the UI; will be driven by TCI in a later phase. Stored in Hz;
    /// rendered as `MHz.kHz.HH` (10 Hz resolution) via `format_frequency_hz`.
    pub frequency: u64,
    /// Read-only in the UI; will be driven by TCI in a later phase.
    pub mode: String,
    pub rst_sent: String,
    pub rst_rcvd: String,
    pub locator: String,
}

impl Default for EntryForm {
    fn default() -> Self {
        Self {
            callsign: String::new(),
            frequency: 14_200_000,
            mode: String::from("SSB"),
            rst_sent: String::new(),
            rst_rcvd: String::new(),
            locator: String::new(),
        }
    }
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
    /// Currently selected category in the Settings sidebar.
    pub settings_active_page: SettingsPage,
    /// Live search query in the Settings sidebar. Empty = no filtering.
    pub settings_search: String,
    pub db: Db,
    /// In-memory cache of all QSOs (sorted desc by datetime). Refreshed after every insert.
    pub qsos: Vec<Qso>,
    /// Currently selected QSO id in the Log window, or `None` if no row is selected.
    pub selected_qso_id: Option<i64>,
    /// Last cursor position observed inside the Log window's body, relative to
    /// the body's top-left. Used to anchor the right-click context menu near
    /// the click point, mirroring Zed's behavior.
    pub log_cursor: Point,
    /// Open context menu in the Log window, if any.
    pub context_menu: Option<ContextMenuState>,
    /// Snapshot of background-task connection states rendered in the status bar.
    /// Updated by future TCI / cluster / sync workers.
    pub bg_status: BackgroundStatus,
    /// Current UTC time, refreshed once per second via the [`Message::Tick`] subscription.
    pub current_utc: DateTime<Utc>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let (id, open_task) = window::open(WindowSettings {
            size: Size::new(1000.0, 220.0),
            position: window::Position::Centered,
            min_size: Some(Size::new(850.0, 200.0)),
            decorations: false,
            icon: app_icon(),
            ..WindowSettings::default()
        });

        let config = AppConfig::load();

        // Apply the persisted language as soon as we have the config so that
        // every subsequent string lookup goes through the right Fluent bundle.
        i18n::set_language(config.appearance.language);

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
            settings_active_page: SettingsPage::default(),
            settings_search: String::new(),
            config,
            db,
            qsos,
            selected_qso_id: None,
            log_cursor: Point::ORIGIN,
            context_menu: None,
            bg_status: BackgroundStatus::default(),
            current_utc: Utc::now(),
        };

        (app, open_task.discard())
    }

    fn title(&self, window_id: window::Id) -> String {
        if Some(window_id) == self.settings_window {
            t!("window-title-settings")
        } else if Some(window_id) == self.log_window {
            t!("window-title-log")
        } else {
            t!("window-title-app")
        }
    }

    fn theme(&self, _window_id: window::Id) -> Theme {
        self.config.appearance.theme.to_iced()
    }

    // `&self` is required by `Daemon::subscription` (`Fn(&State) -> Subscription`).
    #[allow(clippy::unused_self)]
    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            window::close_events().map(Message::WindowClosed),
            iced::keyboard::listen().filter_map(|event| match event {
                iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Tab),
                    modifiers,
                    ..
                } => Some(Message::TabPressed {
                    shift: modifiers.shift(),
                }),
                iced::keyboard::Event::KeyPressed {
                    key: iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape),
                    ..
                } => Some(Message::ContextMenuDismiss),
                _ => None,
            }),
            iced::time::every(Duration::from_secs(1)).map(|_| Message::Tick),
        ])
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        if Some(window_id) == self.settings_window {
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
                    self.entry.frequency,
                    self.entry.mode.clone(),
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
                            self.entry.rst_sent.clear();
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
                self.settings_active_page = SettingsPage::default();
                self.settings_search.clear();
                let (id, task) = window::open(WindowSettings {
                    size: Size::new(820.0, 580.0),
                    position: window::Position::Centered,
                    min_size: Some(Size::new(640.0, 420.0)),
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
                self.settings_draft.operator.license_class = s;
            }
            Message::SettingsThemeChanged(t) => self.settings_draft.appearance.theme = t,
            Message::SettingsWindowBorderChanged(b) => {
                self.settings_draft.appearance.window_border = b;
            }
            Message::SettingsLanguageChanged(lang) => {
                self.settings_draft.appearance.language = lang;
                // Apply immediately so the Settings window itself re-renders
                // in the new language while the user is still tweaking it.
                i18n::set_language(lang);
            }
            Message::SettingsCategoryChanged(page) => self.settings_active_page = page,
            Message::SettingsSearchChanged(query) => {
                self.settings_search = query;
                // Auto-jump to the first matching page when the current one
                // would be hidden by the filter, so the content pane never
                // displays a page the sidebar doesn't list.
                let q = self.settings_search.trim();
                if !q.is_empty()
                    && !self.settings_active_page.matches(q)
                    && let Some(&first) = SettingsPage::ALL.iter().find(|p| p.matches(q))
                {
                    self.settings_active_page = first;
                }
            }
            Message::SettingsCancelClicked => {
                // Draft is recreated next OpenSettings; just close.
                // Revert any live-applied language change back to the saved value.
                i18n::set_language(self.config.appearance.language);
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
            Message::WindowDragResize(id, direction) => {
                return window::drag_resize(id, direction);
            }
            Message::WindowCloseRequested(id) => return window::close(id),

            // --- Keyboard navigation ---
            Message::TabPressed { shift } => {
                return if shift {
                    iced::widget::operation::focus_previous()
                } else {
                    iced::widget::operation::focus_next()
                };
            }

            Message::MacroPressed(_idx) => {}

            // --- Log window — row selection + context menu ---
            Message::QsoSelected(id) => {
                self.selected_qso_id = if self.selected_qso_id == Some(id) {
                    None
                } else {
                    Some(id)
                };
                // Any plain left click also closes an open context menu — mirrors
                // Zed where clicking elsewhere in the panel dismisses the popover.
                self.context_menu = None;
            }
            Message::QsoContextMenu(id) => {
                self.selected_qso_id = Some(id);
                self.context_menu = Some(ContextMenuState {
                    qso_id: id,
                    position: self.log_cursor,
                });
            }
            Message::LogCursorMoved(p) => self.log_cursor = p,
            Message::ContextMenuDismiss => self.context_menu = None,
            Message::DeleteQsoConfirmed(id) => {
                if let Err(e) = self.db.delete_qso(id) {
                    eprintln!("[db] delete_qso failed: {e:#}");
                } else {
                    self.qsos.retain(|q| q.id != Some(id));
                    if self.selected_qso_id == Some(id) {
                        self.selected_qso_id = None;
                    }
                }
                self.context_menu = None;
            }

            // --- Status bar tick ---
            Message::Tick => self.current_utc = bar::now(),

            // --- Window lifecycle ---
            Message::WindowClosed(id) => {
                if id == self.main_window {
                    return iced::exit();
                }
                if Some(id) == self.settings_window {
                    self.settings_window = None;
                    self.settings_maximized = false;
                    // If user closed via X button without Save/Cancel, ensure
                    // the live-applied language reverts to the saved value.
                    i18n::set_language(self.config.appearance.language);
                }
                if Some(id) == self.log_window {
                    self.log_window = None;
                    self.log_maximized = false;
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

