use lpmr_core::i18n::Language;
use lpmr_core::themes::default_theme;

#[derive(Clone)]
pub struct AppState {
    pub default_lang: Language,
    pub default_theme_id: String,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            default_lang: Language::En,
            default_theme_id: default_theme().id,
        }
    }
}
