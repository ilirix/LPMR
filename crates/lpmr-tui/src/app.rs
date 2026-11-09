use crate::theme::{accent_style, border_style, theme_cycle, title_style, TuiPalette};
use crossterm::event::KeyCode;
use lpmr_core::devices::{format_size, scan_block_devices, BlockDevice};
use lpmr_core::i18n::{Language, translate};
use lpmr_core::lvm::{lvm_available, scan_lvm};
use lpmr_core::ops::{execute, OperationKind, OperationRequest};
use lpmr_core::tools::discover_installed_tools;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Tabs, Wrap};
use ratatui::Frame;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Tab {
    Devices,
    Lvm,
    Tools,
    Ops,
}

pub struct App {
    pub should_quit: bool,
    lang: Language,
    theme_index: usize,
    palette: TuiPalette,
    theme_name: String,
    tab: Tab,
    devices: Vec<(BlockDevice, usize)>,
    list_state: ListState,
    logs: Vec<String>,
    lvm_text: String,
    tools_text: String,
}

impl App {
    pub fn new() -> Self {
        let (palette, theme_name) = theme_cycle(0);
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            should_quit: false,
            lang: Language::En,
            theme_index: 0,
            palette,
            theme_name,
            tab: Tab::Devices,
            devices: Vec::new(),
            list_state,
            logs: vec!["Press r refresh, t theme, l language, d dry-run op".into()],
            lvm_text: String::new(),
            tools_text: String::new(),
        }
    }

    pub fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('r') => self.refresh(),
            KeyCode::Char('t') => {
                self.theme_index += 1;
                let (p, name) = theme_cycle(self.theme_index);
                self.palette = p;
                self.theme_name = name;
                self.log(format!("Theme: {}", self.theme_name));
            }
            KeyCode::Char('l') => {
                self.lang = next_lang(self.lang);
                self.log(format!("Language: {}", self.lang.label()));
            }
            KeyCode::Tab => self.tab = next_tab(self.tab),
            KeyCode::Down | KeyCode::Char('j') => {
                let i = self.list_state.selected().unwrap_or(0);
                let max = self.devices.len().saturating_sub(1);
                self.list_state.select(Some((i + 1).min(max)));
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let i = self.list_state.selected().unwrap_or(0);
                self.list_state.select(Some(i.saturating_sub(1)));
            }
            KeyCode::Char('d') => self.dry_run_sample(),
            _ => {}
        }
    }

    fn refresh(&mut self) {
        match scan_block_devices() {
            Ok(tree) => {
                self.devices = flatten(&tree.devices, 0);
                self.log(format!("Devices: {}", self.devices.len()));
            }
            Err(e) => self.log(format!("Device scan failed: {e}")),
        }
        if !lvm_available() {
            self.lvm_text = "LVM not installed — install lvm2 package".into();
        } else {
            match scan_lvm() {
                Ok(s) => {
                    self.lvm_text = serde_json::to_string_pretty(&s).unwrap_or_default();
                }
                Err(e) => self.lvm_text = format!("LVM scan failed: {e}"),
            }
        }
        let tools = discover_installed_tools();
        self.tools_text = tools
            .iter()
            .map(|t| format!("{} -> {}", t.name, t.path.display()))
            .collect::<Vec<_>>()
            .join("\n");
    }

    fn dry_run_sample(&mut self) {
        let req = OperationRequest {
            kind: OperationKind::CreateImage,
            dry_run: true,
            device: None,
            path: Some("/var/lib/lpmr/demo.qcow2".into()),
            target: None,
            size: Some("4G".into()),
            fstype: None,
            partition_table: None,
            start: None,
            end: None,
            mount_point: None,
            options: None,
            source: None,
            dest: None,
            image_format: Some(lpmr_core::qemu::QemuFormat::Qcow2),
            vg: None,
            lv: None,
            luks_name: None,
        };
        match execute(req) {
            Ok(r) => self.log(r.command),
            Err(e) => self.log(format!("Op error: {e}")),
        }
    }

    fn log(&mut self, msg: String) {
        self.logs.push(msg);
        if self.logs.len() > 200 {
            let drain = self.logs.len() - 200;
            self.logs.drain(0..drain);
        }
    }
}

fn next_tab(tab: Tab) -> Tab {
    match tab {
        Tab::Devices => Tab::Lvm,
        Tab::Lvm => Tab::Tools,
        Tab::Tools => Tab::Ops,
        Tab::Ops => Tab::Devices,
    }
}

fn next_lang(lang: Language) -> Language {
    let all = Language::all();
    let idx = all.iter().position(|l| *l == lang).unwrap_or(0);
    all[(idx + 1) % all.len()]
}

fn flatten(devs: &[BlockDevice], depth: usize) -> Vec<(BlockDevice, usize)> {
    let mut out = Vec::new();
    for d in devs {
        out.push((d.clone(), depth));
        out.extend(flatten(&d.children, depth + 1));
    }
    out
}

pub fn render(f: &mut Frame, app: &mut App) {
    if app.devices.is_empty() {
        app.refresh();
    }
    let p = app.palette.clone();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(7),
        ])
        .split(f.area());

    let title = translate(app.lang, "app.title");
    let header = Paragraph::new(Line::from(vec![
        Span::styled(format!(" {title} "), title_style(&p)),
        Span::raw(" | Raven-style docked panels "),
        Span::styled(format!("Theme: {} ", app.theme_name), accent_style(&p)),
        Span::raw(format!("Lang: {} ", app.lang.code())),
        Span::styled(" q quit ", Style::default().fg(p.warning)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style(&p))
            .title(" LPMR "),
    );
    f.render_widget(header, chunks[0]);

    let workspace = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30),
            Constraint::Percentage(40),
            Constraint::Percentage(30),
        ])
        .split(chunks[1]);

    render_nav(f, app, workspace[0], &p);
    render_center(f, app, workspace[1], &p);
    render_ops(f, app, workspace[2], &p);

    let log_lines: Vec<Line> = app
        .logs
        .iter()
        .rev()
        .take(6)
        .rev()
        .map(|l| Line::from(l.as_str()))
        .collect();
    let log = Paragraph::new(log_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style(&p))
                .title(format!(" {} ", translate(app.lang, "panel.log"))),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(log, chunks[2]);
}

fn render_nav(f: &mut Frame, app: &mut App, area: Rect, p: &TuiPalette) {
    let titles = vec![
        translate(app.lang, "nav.devices"),
        translate(app.lang, "nav.lvm"),
        translate(app.lang, "nav.tools"),
        translate(app.lang, "nav.operations"),
    ];
    let index = match app.tab {
        Tab::Devices => 0,
        Tab::Lvm => 1,
        Tab::Tools => 2,
        Tab::Ops => 3,
    };
    let tabs = Tabs::new(titles)
        .select(index)
        .style(Style::default().fg(p.text))
        .highlight_style(accent_style(p));
    let inner = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(area);
    f.render_widget(
        tabs.block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style(&p)),
        ),
        inner[0],
    );

    match app.tab {
        Tab::Devices => {
            let items: Vec<ListItem> = app
                .devices
                .iter()
                .map(|(d, depth)| {
                    let indent = " ".repeat(depth * 2);
                    ListItem::new(format!(
                        "{indent}{} {} ({})",
                        d.name,
                        format_size(d.size_bytes),
                        d.fstype.clone().unwrap_or_else(|| "-".into())
                    ))
                })
                .collect();
            let list = List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(border_style(&p))
                    .title(format!(" {}", translate(app.lang, "nav.devices"))),
            );
            f.render_stateful_widget(list, inner[1], &mut app.list_state);
        }
        Tab::Lvm => {
            let para = Paragraph::new(app.lvm_text.as_str())
                .wrap(Wrap { trim: false })
                .block(Block::default().borders(Borders::ALL).border_style(border_style(p)));
            f.render_widget(para, inner[1]);
        }
        Tab::Tools => {
            let para = Paragraph::new(app.tools_text.as_str())
                .wrap(Wrap { trim: false })
                .block(Block::default().borders(Borders::ALL).border_style(border_style(p)));
            f.render_widget(para, inner[1]);
        }
        Tab::Ops => {
            let text = "Keys: d dry-run sample image\nr refresh\nt theme\nl language";
            f.render_widget(
                Paragraph::new(text).block(Block::default().borders(Borders::ALL).border_style(border_style(p))),
                inner[1],
            );
        }
    }
}

fn render_center(f: &mut Frame, app: &App, area: Rect, p: &TuiPalette) {
    let mut text = String::from("Select a device\n");
    if let Some(i) = app.list_state.selected() {
        if let Some((d, _)) = app.devices.get(i) {
            text = format!(
                "Path: {}\nSize: {}\nModel: {}\nFstype: {}\nMount: {:?}\n",
                d.path,
                format_size(d.size_bytes),
                d.model.clone().unwrap_or_else(|| "-".into()),
                d.fstype.clone().unwrap_or_else(|| "-".into()),
                d.mountpoints
            );
        }
    }
    let para = Paragraph::new(text)
        .wrap(Wrap { trim: true })
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style(&p))
                .title(format!(
                    " {} ",
                    translate(app.lang, "panel.details")
                )),
        );
    f.render_widget(para, area);
}

fn render_ops(f: &mut Frame, app: &App, area: Rect, p: &TuiPalette) {
    let body = format!(
        "{}\n\nSupported:\n- GPT/MBR partitions (parted)\n- ext4/xfs/btrfs/vfat/ntfs\n- LVM PV/VG/LV\n- qemu-img raw/qcow2/vmdk/vdi/vhdx\n- Proxmox pvesm/qm (when present)\n- LUKS open/close\n",
        translate(app.lang, "warn.privilege")
    );
    let para = Paragraph::new(body)
        .wrap(Wrap { trim: true })
        .style(Style::default().fg(p.text))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style(&p))
                .title(format!(" {} ", translate(app.lang, "nav.operations"))),
        );
    f.render_widget(para, area);
}
