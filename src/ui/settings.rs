//! Settings window — Zed-style two-pane layout.
//!
//! Top:    custom title bar (shared widget)
//! Body:   sidebar (search + page list) | vertical rule | content (sections + items)
//! Footer: Cancel / Save spanning the full width
//!
//! See `crates/settings_ui/src/settings_ui.rs` in the Zed repo for the
//! reference structure (`NavBarEntry`, `SettingsPage`, `SettingsPageItem`).

use iced::widget::text_input as text_input_widget;
use iced::widget::{
    Column, Space, button, checkbox, column, container, row, rule, scrollable, text, text_input,
};
use iced::window;
use iced::{Alignment, Background, Border, Element, Length, Shadow, Theme};

use crate::app::{App, FONT_MONO, Message};
use crate::config::Language;
use crate::t;
use crate::theme::AppTheme;
use crate::ui::buttons::{outlined, solid};
use crate::ui::inputs::{dropdown, input};
use crate::ui::title;

const SIDEBAR_WIDTH: f32 = 200.0;
const FIELD_LABEL_WIDTH: f32 = 140.0;
const FIELD_INPUT_WIDTH: f32 = 280.0;
const FIELD_SPACING: f32 = 10.0;
const SECTION_SPACING: f32 = 18.0;
const SIDEBAR_ENTRY_RADIUS: f32 = 4.0;

// ── Page enum ──────────────────────────────────────────────────────────────

/// Top-level category in the settings sidebar. Mirrors Zed's `SettingsPage`
/// (one entry per `NavBarEntry` root).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SettingsPage {
    #[default]
    Operator,
    Appearance,
    Radio,
    Cluster,
    Maps,
    Backup,
    About,
}

impl SettingsPage {
    pub const ALL: &'static [SettingsPage] = &[
        SettingsPage::Operator,
        SettingsPage::Appearance,
        SettingsPage::Radio,
        SettingsPage::Cluster,
        SettingsPage::Maps,
        SettingsPage::Backup,
        SettingsPage::About,
    ];

    pub fn title(self) -> String {
        match self {
            SettingsPage::Operator => t!("settings-page-operator"),
            SettingsPage::Appearance => t!("settings-page-appearance"),
            SettingsPage::Radio => t!("settings-page-radio"),
            SettingsPage::Cluster => t!("settings-page-cluster"),
            SettingsPage::Maps => t!("settings-page-maps"),
            SettingsPage::Backup => t!("settings-page-backup"),
            SettingsPage::About => t!("settings-page-about"),
        }
    }

    /// Every translated string in the page (title, section headers, field
    /// labels) — used for the search index. When `query` matches any of these
    /// the page is shown in the sidebar.
    fn search_terms(self) -> Vec<String> {
        let mut terms = vec![self.title()];
        match self {
            SettingsPage::Operator => terms.extend([
                t!("settings-section-identity"),
                t!("field-callsign"),
                t!("field-name"),
                t!("settings-section-location"),
                t!("field-qth"),
                t!("field-locator"),
                t!("settings-section-license"),
                t!("field-license-class"),
            ]),
            SettingsPage::Appearance => terms.extend([
                t!("settings-section-theme"),
                t!("setting-theme"),
                t!("setting-window-border"),
                t!("settings-section-language"),
                t!("setting-language"),
            ]),
            SettingsPage::About => terms.extend([
                t!("settings-section-about"),
                t!("about-version"),
                t!("about-license"),
                t!("about-repository"),
            ]),
            // Placeholder pages: only the title is searchable.
            SettingsPage::Radio
            | SettingsPage::Cluster
            | SettingsPage::Maps
            | SettingsPage::Backup => {}
        }
        terms
    }

    pub fn matches(self, query: &str) -> bool {
        if query.is_empty() {
            return true;
        }
        let needle = query.to_lowercase();
        self.search_terms()
            .iter()
            .any(|t| t.to_lowercase().contains(&needle))
    }
}

// ── Top-level view ─────────────────────────────────────────────────────────

pub fn view<'a>(state: &'a App, window_id: window::Id) -> Element<'a, Message> {
    container(
        column![
            title::view(state, window_id, t!("window-title-settings"), false),
            rule::horizontal(1).style(title::rule_style),
            body(state),
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

// ── Body: sidebar | content + footer ───────────────────────────────────────

fn body(state: &App) -> Element<'_, Message> {
    column![
        row![
            sidebar(state),
            rule::vertical(1).style(title::rule_style),
            content(state),
        ]
        .height(Length::Fill),
        rule::horizontal(1).style(title::rule_style),
        footer(),
    ]
    .into()
}

// ── Sidebar (search + page list) ───────────────────────────────────────────

fn sidebar(state: &App) -> Element<'_, Message> {
    let query = state.settings_search.trim();

    let search = container(
        text_input_widget(&t!("settings-search-placeholder"), &state.settings_search)
            .on_input(Message::SettingsSearchChanged)
            .style(search_input_style)
            .padding([4, 8])
            .size(13),
    )
    .padding([10, 10]);

    let mut entries: Column<Message> = Column::new().spacing(2).padding([0, 6]);
    let mut any_visible = false;
    for &page in SettingsPage::ALL {
        if !page.matches(query) {
            continue;
        }
        any_visible = true;
        entries = entries.push(sidebar_entry(page, state.settings_active_page == page));
    }

    let list: Element<Message> = if any_visible {
        scrollable(entries).height(Length::Fill).into()
    } else {
        container(text(t!("settings-empty-results")).size(13).style(muted_text))
            .padding([12, 10])
            .into()
    };

    container(column![search, list].spacing(0))
        .width(Length::Fixed(SIDEBAR_WIDTH))
        .height(Length::Fill)
        .into()
}

fn sidebar_entry(page: SettingsPage, active: bool) -> Element<'static, Message> {
    button(
        container(text(page.title()).size(13))
            .padding([6, 10])
            .width(Length::Fill),
    )
    .on_press(Message::SettingsCategoryChanged(page))
    .style(move |theme: &Theme, status: button::Status| {
        sidebar_entry_style(theme, status, active)
    })
    .width(Length::Fill)
    .padding(0)
    .into()
}

fn sidebar_entry_style(theme: &Theme, status: button::Status, active: bool) -> button::Style {
    let palette = theme.extended_palette();

    let (bg, fg) = if active {
        (
            Some(palette.primary.weak.color),
            palette.primary.weak.text,
        )
    } else {
        match status {
            button::Status::Hovered => (Some(ghost(theme, 0.07, 0.05)), palette.background.base.text),
            button::Status::Pressed => (Some(ghost(theme, 0.12, 0.09)), palette.background.base.text),
            _ => (None, palette.background.base.text),
        }
    };

    button::Style {
        background: bg.map(Background::Color),
        text_color: fg,
        border: Border {
            radius: SIDEBAR_ENTRY_RADIUS.into(),
            ..Border::default()
        },
        shadow: Shadow::default(),
        ..button::Style::default()
    }
}

// ── Content (active page + filter by search) ───────────────────────────────

fn content(state: &App) -> Element<'_, Message> {
    let query = state.settings_search.trim().to_lowercase();
    let page = state.settings_active_page;

    let inner: Element<Message> = match page {
        SettingsPage::Operator => operator_page(state, &query),
        SettingsPage::Appearance => appearance_page(state, &query),
        SettingsPage::About => about_page(),
        SettingsPage::Radio
        | SettingsPage::Cluster
        | SettingsPage::Maps
        | SettingsPage::Backup => placeholder_page(),
    };

    scrollable(container(inner).padding([20, 24]).width(Length::Fill))
        .height(Length::Fill)
        .width(Length::Fill)
        .into()
}

// ── Footer ─────────────────────────────────────────────────────────────────

fn footer() -> Element<'static, Message> {
    container(
        row![
            Space::new().width(Length::Fill),
            outlined(text(t!("button-cancel")).size(14)).on_press(Message::SettingsCancelClicked),
            solid(text(t!("button-save")).size(14)).on_press(Message::SettingsSaveClicked),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding([10, 16])
    .width(Length::Fill)
    .into()
}

// ── Pages ──────────────────────────────────────────────────────────────────

fn operator_page<'a>(state: &'a App, query: &str) -> Element<'a, Message> {
    let identity = section(
        t!("settings-section-identity"),
        vec![
            field_text(
                t!("field-callsign"),
                &state.settings_draft.operator.callsign,
                Message::SettingsCallsignChanged,
                query,
            ),
            field_text(
                t!("field-name"),
                &state.settings_draft.operator.name,
                Message::SettingsNameChanged,
                query,
            ),
        ],
        query,
    );

    let location = section(
        t!("settings-section-location"),
        vec![
            field_text(
                t!("field-qth"),
                &state.settings_draft.operator.qth,
                Message::SettingsQthChanged,
                query,
            ),
            field_text(
                t!("field-locator"),
                &state.settings_draft.operator.locator,
                Message::SettingsLocatorChanged,
                query,
            ),
        ],
        query,
    );

    let license = section(
        t!("settings-section-license"),
        vec![field_text(
            t!("field-license-class"),
            &state.settings_draft.operator.license_class,
            Message::SettingsLicenseClassChanged,
            query,
        )],
        query,
    );

    join_sections(vec![identity, location, license], query)
}

fn appearance_page<'a>(state: &'a App, query: &str) -> Element<'a, Message> {
    let theme_section = section(
        t!("settings-section-theme"),
        vec![
            field_dropdown(
                t!("setting-theme"),
                AppTheme::ALL,
                Some(state.settings_draft.appearance.theme),
                Message::SettingsThemeChanged,
                query,
            ),
            field_toggle(
                t!("setting-window-border"),
                state.settings_draft.appearance.window_border,
                Message::SettingsWindowBorderChanged,
                query,
            ),
        ],
        query,
    );

    let language_section = section(
        t!("settings-section-language"),
        vec![field_dropdown(
            t!("setting-language"),
            Language::ALL,
            Some(state.settings_draft.appearance.language),
            Message::SettingsLanguageChanged,
            query,
        )],
        query,
    );

    join_sections(vec![theme_section, language_section], query)
}

fn about_page() -> Element<'static, Message> {
    let info = column![
        about_row(t!("about-version"), env!("CARGO_PKG_VERSION").to_string()),
        about_row(t!("about-license"), env!("CARGO_PKG_LICENSE").to_string()),
        about_row(
            t!("about-repository"),
            env!("CARGO_PKG_REPOSITORY").to_string()
        ),
    ]
    .spacing(FIELD_SPACING);

    section_static(t!("settings-section-about"), info.into())
}

fn placeholder_page() -> Element<'static, Message> {
    section_static(
        t!("settings-section-coming-soon"),
        text(t!("settings-coming-soon"))
            .size(13)
            .style(muted_text)
            .into(),
    )
}

// ── Section / field helpers ────────────────────────────────────────────────

/// Build a section: header + items, filtered by query. Returns `None` if
/// nothing inside survives the filter (so the caller can skip rendering it
/// entirely).
fn section<'a>(
    title_str: String,
    items: Vec<Option<Element<'a, Message>>>,
    query: &str,
) -> Option<Element<'a, Message>> {
    let header_matches = query.is_empty() || title_str.to_lowercase().contains(query);

    let visible: Vec<Element<Message>> = items
        .into_iter()
        .flatten()
        .collect();

    if visible.is_empty() && !header_matches {
        return None;
    }

    let mut col = Column::new().spacing(FIELD_SPACING);
    col = col.push(section_header(title_str));
    for it in visible {
        col = col.push(it);
    }
    Some(col.into())
}

/// Build a section with content that is not query-filtered (About, placeholder).
fn section_static(title_str: String, body: Element<'_, Message>) -> Element<'_, Message> {
    column![section_header(title_str), body]
        .spacing(FIELD_SPACING)
        .into()
}

fn section_header(label: String) -> Element<'static, Message> {
    column![
        text(label).size(13).style(section_label_style),
        rule::horizontal(1).style(title::rule_style),
    ]
    .spacing(6)
    .into()
}

fn join_sections<'a>(
    sections: Vec<Option<Element<'a, Message>>>,
    query: &str,
) -> Element<'a, Message> {
    let visible: Vec<Element<Message>> = sections.into_iter().flatten().collect();

    if visible.is_empty() {
        return container(text(t!("settings-empty-results")).size(13).style(muted_text))
            .padding(8)
            .width(Length::Fill)
            .into();
    }

    let _ = query; // matched at the section level already
    Column::with_children(visible)
        .spacing(SECTION_SPACING)
        .into()
}

fn field_text<'a>(
    label: String,
    value: &'a str,
    on_change: fn(String) -> Message,
    query: &str,
) -> Option<Element<'a, Message>> {
    if !label_matches(&label, query) {
        return None;
    }
    Some(
        row![
            field_label(label),
            input("", value)
                .on_input(on_change)
                .font(FONT_MONO)
                .width(Length::Fixed(FIELD_INPUT_WIDTH)),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into(),
    )
}

fn field_dropdown<'a, T>(
    label: String,
    options: &'a [T],
    selected: Option<T>,
    on_select: impl Fn(T) -> Message + 'a,
    query: &str,
) -> Option<Element<'a, Message>>
where
    T: Clone + ToString + PartialEq + 'a,
{
    if !label_matches(&label, query) {
        return None;
    }
    Some(
        row![
            field_label(label),
            dropdown(options, selected, on_select, Length::Fixed(FIELD_INPUT_WIDTH)),
        ]
        .spacing(8)
        .align_y(Alignment::Center)
        .into(),
    )
}

fn field_toggle<'a>(
    label: String,
    value: bool,
    on_toggle: fn(bool) -> Message,
    query: &str,
) -> Option<Element<'a, Message>> {
    if !label_matches(&label, query) {
        return None;
    }
    Some(
        row![field_label(label), checkbox(value).on_toggle(on_toggle),]
            .spacing(8)
            .align_y(Alignment::Center)
            .into(),
    )
}

fn field_label(label: String) -> Element<'static, Message> {
    text(label)
        .size(13)
        .width(Length::Fixed(FIELD_LABEL_WIDTH))
        .into()
}

fn about_row(label: String, value: String) -> Element<'static, Message> {
    row![
        text(label)
            .size(13)
            .width(Length::Fixed(FIELD_LABEL_WIDTH))
            .style(muted_text),
        text(value).size(13).font(FONT_MONO),
    ]
    .spacing(8)
    .align_y(Alignment::Center)
    .into()
}

fn label_matches(label: &str, query: &str) -> bool {
    query.is_empty() || label.to_lowercase().contains(query)
}

// ── Styles ─────────────────────────────────────────────────────────────────

fn search_input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let palette = theme.extended_palette();

    let border_color = match status {
        text_input::Status::Focused { .. } => palette.primary.strong.color,
        text_input::Status::Hovered => palette.background.strong.color,
        _ => {
            let mut c = palette.background.strong.color;
            c.a = 0.4;
            c
        }
    };

    text_input::Style {
        background: Background::Color(palette.background.base.color),
        border: Border {
            color: border_color,
            width: 1.0,
            radius: 4.0.into(),
        },
        icon: palette.background.weak.text,
        placeholder: palette.background.strong.color,
        value: palette.background.base.text,
        selection: palette.primary.weak.color,
    }
}

fn section_label_style(theme: &Theme) -> text::Style {
    let mut c = theme.extended_palette().background.base.text;
    c.a = 0.72;
    text::Style { color: Some(c) }
}

fn muted_text(theme: &Theme) -> text::Style {
    let mut c = theme.extended_palette().background.base.text;
    c.a = 0.6;
    text::Style { color: Some(c) }
}

fn ghost(theme: &Theme, dark_alpha: f32, light_alpha: f32) -> iced::Color {
    let palette = theme.extended_palette();
    iced::Color {
        a: if palette.is_dark { dark_alpha } else { light_alpha },
        ..palette.background.base.text
    }
}
