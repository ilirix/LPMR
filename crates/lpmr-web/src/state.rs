use crate::jobs::JobRecord;
use lpmr_core::i18n::Language;
use lpmr_core::themes::default_theme;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type JobStore = Arc<RwLock<HashMap<String, JobRecord>>>;

#[derive(Clone)]
pub struct AppState {
    pub default_lang: Language,
    pub default_theme_id: String,
    pub assets_dir: std::path::PathBuf,
    pub jobs: JobStore,
}

impl AppState {
    pub fn new(assets_dir: std::path::PathBuf) -> Self {
        Self {
            default_lang: Language::En,
            default_theme_id: default_theme().id,
            assets_dir,
            jobs: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
