use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Language {
    #[default]
    En,
    Es,
    Zh,
    Hi,
    Ar,
    Fr,
}

impl Language {
    pub fn code(self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Es => "es",
            Self::Zh => "zh",
            Self::Hi => "hi",
            Self::Ar => "ar",
            Self::Fr => "fr",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Es => "Español",
            Self::Zh => "中文",
            Self::Hi => "हिन्दी",
            Self::Ar => "العربية",
            Self::Fr => "Français",
        }
    }

    pub fn all() -> &'static [Language] {
        &[
            Language::En,
            Language::Es,
            Language::Zh,
            Language::Hi,
            Language::Ar,
            Language::Fr,
        ]
    }

    pub fn from_code(code: &str) -> Language {
        match code.to_lowercase().as_str() {
            "es" => Language::Es,
            "zh" | "zh-cn" | "zh-tw" => Language::Zh,
            "hi" => Language::Hi,
            "ar" => Language::Ar,
            "fr" => Language::Fr,
            _ => Language::En,
        }
    }
}

type Table = &'static [(&'static str, &'static str)];

const EN: Table = &[
    ("app.title", "Proxmox VE Storage Orchestrator"),
    ("nav.dashboard", "Dashboard"),
    ("nav.devices", "Devices"),
    ("nav.devices.sub", "Local disks & partitions"),
    ("nav.lvm", "LVM"),
    ("nav.images", "VM Images"),
    ("nav.proxmox", "Proxmox"),
    ("nav.operations", "Operations"),
    ("nav.tools", "System Tools"),
    ("action.refresh", "Refresh"),
    ("action.dry_run", "Dry run"),
    ("action.execute", "Execute"),
    ("panel.details", "Details"),
    ("panel.log", "Operation log"),
    ("theme.label", "Enterprise theme"),
    ("lang.label", "Language"),
    ("warn.privilege", "Most actions require root privileges."),
];

const ES: Table = &[
    ("app.title", "Panel de almacenamiento LPMR"),
    ("nav.devices", "Dispositivos"),
    ("nav.lvm", "LVM"),
    ("nav.images", "Imágenes VM"),
    ("nav.proxmox", "Proxmox"),
    ("nav.operations", "Operaciones"),
    ("nav.tools", "Herramientas"),
    ("action.refresh", "Actualizar"),
    ("action.dry_run", "Simulación"),
    ("action.execute", "Ejecutar"),
    ("panel.details", "Detalles"),
    ("panel.log", "Registro"),
    ("theme.label", "Tema empresarial"),
    ("lang.label", "Idioma"),
    ("warn.privilege", "La mayoría de acciones requieren root."),
];

const ZH: Table = &[
    ("app.title", "LPMR 存储面板"),
    ("nav.devices", "设备"),
    ("nav.lvm", "LVM"),
    ("nav.images", "虚拟机镜像"),
    ("nav.proxmox", "Proxmox"),
    ("nav.operations", "操作"),
    ("nav.tools", "系统工具"),
    ("action.refresh", "刷新"),
    ("action.dry_run", "试运行"),
    ("action.execute", "执行"),
    ("panel.details", "详情"),
    ("panel.log", "操作日志"),
    ("theme.label", "企业主题"),
    ("lang.label", "语言"),
    ("warn.privilege", "大多数操作需要 root 权限。"),
];

const HI: Table = &[
    ("app.title", "LPMR स्टोरेज पैनल"),
    ("nav.devices", "डिवाइस"),
    ("nav.lvm", "LVM"),
    ("nav.images", "VM इमेज"),
    ("nav.proxmox", "Proxmox"),
    ("nav.operations", "ऑपरेशन"),
    ("nav.tools", "सिस्टम टूल"),
    ("action.refresh", "रीफ़्रेश"),
    ("action.dry_run", "ड्राई रन"),
    ("action.execute", "चलाएँ"),
    ("panel.details", "विवरण"),
    ("panel.log", "लॉग"),
    ("theme.label", "एंटरप्राइज थीम"),
    ("lang.label", "भाषा"),
    ("warn.privilege", "अधिकांश क्रियाओं के लिए root आवश्यक है।"),
];

const AR: Table = &[
    ("app.title", "لوحة تخزين LPMR"),
    ("nav.devices", "الأجهزة"),
    ("nav.lvm", "LVM"),
    ("nav.images", "صور الأجهزة الافتراضية"),
    ("nav.proxmox", "Proxmox"),
    ("nav.operations", "العمليات"),
    ("nav.tools", "أدوات النظام"),
    ("action.refresh", "تحديث"),
    ("action.dry_run", "تجربة"),
    ("action.execute", "تنفيذ"),
    ("panel.details", "التفاصيل"),
    ("panel.log", "السجل"),
    ("theme.label", "سمة المؤسسة"),
    ("lang.label", "اللغة"),
    ("warn.privilege", "معظم الإجراءات تتطلب صلاحيات root."),
];

const FR: Table = &[
    ("app.title", "Panneau de stockage LPMR"),
    ("nav.devices", "Périphériques"),
    ("nav.lvm", "LVM"),
    ("nav.images", "Images VM"),
    ("nav.proxmox", "Proxmox"),
    ("nav.operations", "Opérations"),
    ("nav.tools", "Outils système"),
    ("action.refresh", "Actualiser"),
    ("action.dry_run", "Simulation"),
    ("action.execute", "Exécuter"),
    ("panel.details", "Détails"),
    ("panel.log", "Journal"),
    ("theme.label", "Thème entreprise"),
    ("lang.label", "Langue"),
    ("warn.privilege", "La plupart des actions nécessitent root."),
];

fn table(lang: Language) -> Table {
    match lang {
        Language::En => EN,
        Language::Es => ES,
        Language::Zh => ZH,
        Language::Hi => HI,
        Language::Ar => AR,
        Language::Fr => FR,
    }
}

pub fn translate(lang: Language, key: &str) -> String {
    let t = table(lang);
    t.iter()
        .find(|(k, _)| *k == key)
        .map(|(_, v)| (*v).to_string())
        .unwrap_or_else(|| key.to_string())
}
