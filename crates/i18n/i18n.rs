use gpui::{App, BorrowAppContext as _, Global};
use settings::{RegisterSetting, Settings, SettingsContent, SettingsStore, UiLanguage};
use std::{cell::{Cell, RefCell}, collections::HashMap};
#[allow(unused_imports)]
use util::__rust_embed::RustEmbed as _;
use util::{asset_str, fs_embed};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct CurrentLocale(pub UiLanguage);

impl Global for CurrentLocale {}

#[derive(Clone, PartialEq, RegisterSetting)]
pub struct UiLanguageSettings {
    pub ui_language: UiLanguage,
}

impl Settings for UiLanguageSettings {
    fn from_settings(content: &SettingsContent) -> Self {
        Self {
            ui_language: content.ui_language.unwrap_or_default(),
        }
    }
}

fs_embed! {
    pub struct LocaleAssets,
    crate_relative = "../../assets",
    root_relative = "assets",
    include = ["locales/**/*"],
}

struct Catalog {
    locale: UiLanguage,
    entries: HashMap<String, String>,
}

thread_local! {
    static CURRENT_LOCALE: Cell<UiLanguage> = const { Cell::new(UiLanguage::En) };
    static CATALOG: RefCell<Option<Catalog>> = const { RefCell::new(None) };
}

fn locale_asset_dir(locale: UiLanguage) -> Option<&'static str> {
    match locale {
        UiLanguage::En => None,
        UiLanguage::ZhCn => Some("locales/zh-CN/"),
    }
}

fn load_catalog(locale: UiLanguage) -> HashMap<String, String> {
    let Some(dir) = locale_asset_dir(locale) else {
        return HashMap::new();
    };

    let mut catalog = HashMap::new();
    for path in LocaleAssets::iter() {
        let Some(file_name) = path.strip_prefix(dir) else {
            continue;
        };
        if !file_name.ends_with(".json") {
            continue;
        }
        match serde_json::from_str::<HashMap<String, String>>(&asset_str::<LocaleAssets>(&path)) {
            Ok(entries) => catalog.extend(entries),
            Err(error) => log::error!("failed to parse locale file {path}: {error}"),
        }
    }
    catalog
}

fn with_catalog<R>(f: impl FnOnce(&HashMap<String, String>) -> R) -> R {
    CATALOG.with(|slot| {
        let mut slot = slot.borrow_mut();
        let locale = CURRENT_LOCALE.with(Cell::get);
        if slot.as_ref().is_none_or(|catalog| catalog.locale != locale) {
            *slot = Some(Catalog {
                locale,
                entries: load_catalog(locale),
            });
        }
        f(&slot.as_ref().expect("catalog was initialized").entries)
    })
}

/// Returns the translation for `source`, falling back to the source text when
/// no translation exists.
pub fn tr(source: &str) -> gpui::SharedString {
    if source.is_empty() {
        return gpui::SharedString::new_static("");
    }

    with_catalog(|catalog| match catalog.get(source) {
        Some(translated) => gpui::SharedString::new(translated),
        None => gpui::SharedString::new(source),
    })
}

/// Returns a contextual translation. The context-specific entry is preferred,
/// and the plain source text is used as a fallback.
pub fn tr_ctx(context: &str, source: &str) -> gpui::SharedString {
    let key = format!("{context}/{source}");
    let translated = with_catalog(|catalog| catalog.get(&key).cloned());
    match translated {
        Some(translated) => gpui::SharedString::new(translated),
        None => tr(source),
    }
}

/// Translates a string containing `{name}` placeholders and substitutes the
/// provided values.
pub fn tr_format(source: &str, args: &[(&str, &str)]) -> String {
    let mut translated = tr(source).to_string();
    for (name, value) in args {
        translated = translated.replace(&format!("{{{name}}}"), value);
    }
    translated
}

pub fn locale() -> UiLanguage {
    CURRENT_LOCALE.with(Cell::get)
}

fn set_locale(locale: UiLanguage, cx: &mut App) {
    let changed = CURRENT_LOCALE.with(|current| {
        if current.get() == locale {
            false
        } else {
            current.set(locale);
            true
        }
    });
    if !changed {
        return;
    }

    if cx.has_global::<CurrentLocale>() {
        cx.update_global::<CurrentLocale, _>(|current, _| current.0 = locale);
    } else {
        cx.set_global(CurrentLocale(locale));
    }
    cx.refresh_windows();
}

pub fn init(cx: &mut App) {
    let locale = UiLanguageSettings::get_global(cx).ui_language;
    CURRENT_LOCALE.with(|current| current.set(locale));
    cx.set_global(CurrentLocale(locale));
    cx.observe_global::<SettingsStore>(move |cx| {
        let locale = UiLanguageSettings::get_global(cx).ui_language;
        set_locale(locale, cx);
    })
    .detach();
}

#[macro_export]
macro_rules! tr {
    ($text:literal) => {
        $crate::tr($text)
    };
    ($text:literal, $($key:ident = $value:expr),+ $(,)?) => {{
        let values = vec![$(($crate::__private::stringify!($key), ($value).to_string())),+];
        let refs = values
            .iter()
            .map(|(key, value)| (*key, value.as_str()))
            .collect::<Vec<_>>();
        $crate::tr_format($text, &refs)
    }};
}

#[macro_export]
macro_rules! tr_ctx {
    ($context:literal, $text:literal) => {
        $crate::tr_ctx($context, $text)
    };
}

#[macro_export]
macro_rules! tr_format {
    ($text:literal, $($key:ident = $value:expr),+ $(,)?) => {{
        let values = vec![$(($crate::__private::stringify!($key), ($value).to_string())),+];
        let refs = values
            .iter()
            .map(|(key, value)| (*key, value.as_str()))
            .collect::<Vec<_>>();
        $crate::tr_format($text, &refs)
    }};
}

#[doc(hidden)]
pub mod __private {
    pub use std::stringify;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reset(locale: UiLanguage) {
        CURRENT_LOCALE.with(|current| current.set(locale));
        CATALOG.with(|catalog| *catalog.borrow_mut() = None);
    }

    #[test]
    fn falls_back_to_source_text() {
        reset(UiLanguage::En);
        assert_eq!(tr("A string that does not exist"), "A string that does not exist");
    }

    #[test]
    fn translates_with_placeholders() {
        reset(UiLanguage::ZhCn);
        assert_eq!(
            tr_format("Configure {name}", &[("name", "Settings")]),
            "配置 Settings"
        );
        reset(UiLanguage::En);
    }
}
