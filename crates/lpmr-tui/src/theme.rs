use lpmr_core::themes::{enterprise_themes, EnterpriseTheme};
use ratatui::style::{Color, Modifier, Style};

#[derive(Clone)]
pub struct TuiPalette {
    pub window: Color,
    pub panel: Color,
    pub accent: Color,
    pub text: Color,
    pub border: Color,
    pub success: Color,
    pub warning: Color,
    pub danger: Color,
}

impl From<&EnterpriseTheme> for TuiPalette {
    fn from(t: &EnterpriseTheme) -> Self {
        Self {
            window: hex(&t.window),
            panel: hex(&t.panel),
            accent: hex(&t.accent),
            text: hex(&t.text),
            border: hex(&t.border),
            success: hex(&t.success),
            warning: hex(&t.warning),
            danger: hex(&t.danger),
        }
    }
}

pub fn theme_cycle(index: usize) -> (TuiPalette, String) {
    let themes = enterprise_themes();
    let t = &themes[index % themes.len()];
    (TuiPalette::from(t), t.name.clone())
}

fn hex(s: &str) -> Color {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        return Color::White;
    }
    let r = u8::from_str_radix(&s[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&s[2..4], 16).unwrap_or(255);
    let b = u8::from_str_radix(&s[4..6], 16).unwrap_or(255);
    Color::Rgb(r, g, b)
}

pub fn title_style(p: &TuiPalette) -> Style {
    Style::default()
        .fg(p.text)
        .add_modifier(Modifier::BOLD)
}

pub fn border_style(p: &TuiPalette) -> Style {
    Style::default().fg(p.border)
}

pub fn accent_style(p: &TuiPalette) -> Style {
    Style::default().fg(p.accent).add_modifier(Modifier::BOLD)
}
