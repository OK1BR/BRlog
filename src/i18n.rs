//! Localization layer — Project Fluent (`fluent-rs`) via `i18n-embed`.
//!
//! `.ftl` files live under `i18n/<lang>/brlog.ftl` and are embedded into the
//! binary by `rust-embed`, so the produced `.exe` stays single-file. The
//! current loader is exposed through an `ArcSwap` so switching language at
//! runtime only requires re-rendering — no restart.

use std::sync::Arc;

use arc_swap::ArcSwap;
use i18n_embed::fluent::{fluent_language_loader, FluentLanguageLoader};
use once_cell::sync::Lazy;
use rust_embed::RustEmbed;
use unic_langid::{langid, LanguageIdentifier};

use crate::config::Language;

#[derive(RustEmbed)]
#[folder = "i18n/"]
struct Localizations;

const FALLBACK: LanguageIdentifier = langid!("en");

static LOADER: Lazy<ArcSwap<FluentLanguageLoader>> =
    Lazy::new(|| ArcSwap::from_pointee(build_loader(detect_locale())));

fn build_loader(requested: LanguageIdentifier) -> FluentLanguageLoader {
    let loader = fluent_language_loader!();
    if let Err(e) = i18n_embed::select(&loader, &Localizations, &[requested.clone()]) {
        eprintln!("[i18n] select({requested}) failed: {e:#}");
    }
    // Disable Unicode bidi isolate marks — they show up as garbage in the UI.
    loader.set_use_isolating(false);
    loader
}

fn detect_locale() -> LanguageIdentifier {
    sys_locale::get_locale()
        .and_then(|s| s.parse().ok())
        .unwrap_or(FALLBACK)
}

fn resolve(lang: Language) -> LanguageIdentifier {
    match lang {
        Language::Auto => detect_locale(),
        Language::Czech => langid!("cs"),
        Language::English => langid!("en"),
    }
}

/// Switch the active locale. Cheap to call — replaces an Arc atomically.
pub fn set_language(lang: Language) {
    LOADER.store(Arc::new(build_loader(resolve(lang))));
}

/// Translate a static message id. Used by the `t!` macro.
pub fn tr(id: &str) -> String {
    LOADER.load().get(id)
}

/// Translate a static message id. Falls back to the id itself if the bundle
/// does not contain it (so a missing key shows up as text rather than panics).
#[macro_export]
macro_rules! t {
    ($id:literal) => {
        $crate::i18n::tr($id)
    };
}
