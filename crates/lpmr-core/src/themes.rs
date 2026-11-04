use serde::{Deserialize, Serialize};

/// Enterprise themes inspired by Raven palette tokens (midnight, nord, ocean, ember, frost).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnterpriseTheme {
    pub id: String,
    pub name: String,
    pub window: String,
    pub panel: String,
    pub accent: String,
    pub text: String,
    pub border: String,
    pub success: String,
    pub warning: String,
    pub danger: String,
}

impl EnterpriseTheme {
    pub fn css_vars(&self) -> String {
        format!(
            "--lpmr-window:{};--lpmr-panel:{};--lpmr-accent:{};--lpmr-text:{};--lpmr-border:{};--lpmr-success:{};--lpmr-warning:{};--lpmr-danger:{};--lpmr-header:{};",
            self.window, self.panel, self.accent, self.text, self.border, self.success, self.warning, self.danger, self.panel
        )
    }
}

pub fn enterprise_themes() -> Vec<EnterpriseTheme> {
    vec![
        EnterpriseTheme {
            id: "midnight-enterprise".into(),
            name: "Midnight Enterprise".into(),
            window: "#100D1B".into(),
            panel: "#191429".into(),
            accent: "#A78BFA".into(),
            text: "#F2EDFF".into(),
            border: "#352B4B".into(),
            success: "#6EE7B7".into(),
            warning: "#FCD34D".into(),
            danger: "#FDA4AF".into(),
        },
        EnterpriseTheme {
            id: "slate-pro".into(),
            name: "Slate Pro".into(),
            window: "#0F172A".into(),
            panel: "#1E293B".into(),
            accent: "#38BDF8".into(),
            text: "#E2E8F0".into(),
            border: "#334155".into(),
            success: "#4ADE80".into(),
            warning: "#FBBF24".into(),
            danger: "#F87171".into(),
        },
        EnterpriseTheme {
            id: "azure-horizon".into(),
            name: "Azure Horizon".into(),
            window: "#061525".into(),
            panel: "#0B2545".into(),
            accent: "#0078D4".into(),
            text: "#F0F6FC".into(),
            border: "#1B3A57".into(),
            success: "#107C10".into(),
            warning: "#FFB900".into(),
            danger: "#D13438".into(),
        },
        EnterpriseTheme {
            id: "forest-admin".into(),
            name: "Forest Admin".into(),
            window: "#0B1F17".into(),
            panel: "#143528".into(),
            accent: "#34D399".into(),
            text: "#ECFDF5".into(),
            border: "#1F4535".into(),
            success: "#22C55E".into(),
            warning: "#EAB308".into(),
            danger: "#EF4444".into(),
        },
        EnterpriseTheme {
            id: "ruby-executive".into(),
            name: "Ruby Executive".into(),
            window: "#1A0B10".into(),
            panel: "#2A121C".into(),
            accent: "#FB7185".into(),
            text: "#FFF1F2".into(),
            border: "#4C1D2E".into(),
            success: "#86EFAC".into(),
            warning: "#FDE047".into(),
            danger: "#FB7185".into(),
        },
    ]
}

pub fn theme_by_id(id: &str) -> Option<EnterpriseTheme> {
    enterprise_themes().into_iter().find(|t| t.id == id)
}

pub fn default_theme() -> EnterpriseTheme {
    enterprise_themes()[0].clone()
}
