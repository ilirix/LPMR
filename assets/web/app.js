/** LPMR Web UI — layout aligned with OpenZFS Enterprise Director */

const LANG_OPTIONS = [
  { code: "en", name: "English", native: "English", flag: "🇬🇧" },
  { code: "es", name: "Spanish", native: "Español", flag: "🇪🇸" },
  { code: "zh", name: "Chinese", native: "简体中文", flag: "🇨🇳" },
  { code: "hi", name: "Hindi", native: "हिन्दी", flag: "🇮🇳" },
  { code: "fr", name: "French", native: "Français", flag: "🇫🇷" },
  { code: "ar", name: "Arabic", native: "العربية", flag: "🇸🇦", dir: "rtl" },
];

const NAV = [
  { id: "dashboard", labelKey: "nav.dashboard", sub: "Overview & health", icon: "grid" },
  { id: "devices", labelKey: "nav.devices", subKey: "nav.devices.sub", icon: "disk" },
  { id: "lvm", labelKey: "nav.lvm", sub: "Volume groups & logical volumes", icon: "layers" },
  { id: "images", labelKey: "nav.images", sub: "RAW · QCOW2 · VMDK", icon: "box" },
  { id: "proxmox", labelKey: "nav.proxmox", sub: "pvesm storages & volumes", icon: "server" },
  { id: "tools", labelKey: "nav.tools", sub: "Installed system utilities", icon: "wrench" },
  { id: "logs", labelKey: "panel.log", sub: "Operation history", icon: "scroll" },
];

const ICONS = {
  grid: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"/><rect x="14" y="3" width="7" height="7"/><rect x="3" y="14" width="7" height="7"/><rect x="14" y="14" width="7" height="7"/></svg>',
  disk: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"/><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"/></svg>',
  layers: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5"/></svg>',
  box: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/></svg>',
  server: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/></svg>',
  wrench: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/></svg>',
  scroll: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M8 21h12a2 2 0 0 0 2-2v-2H10v2a2 2 0 1 1-4 0V5a2 2 0 0 0-2-2H4"/><path d="M8 21V5a2 2 0 0 1 2-2h9"/></svg>',
  settings: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>',
  refresh: '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 12a9 9 0 1 1-3-6.7"/><path d="M21 3v6h-6"/></svg>',
};

const state = {
  lang: localStorage.getItem("lpmr.lang") || "en",
  theme: localStorage.getItem("lpmr.theme") || "slate-pro",
  tab: "dashboard",
  selected: null,
  strings: {},
  themes: [],
  devices: [],
  lvm: null,
  tools: [],
  proxmox: null,
  proxmoxSelected: null,
  imagePath: "",
  imageInfo: null,
  dryRun: true,
  log: [],
  refreshing: false,
  settingsOpen: false,
  langMenuOpen: false,
  mobileNavOpen: false,
  toast: null,
};

function t(key) {
  return state.strings[key] || key;
}

function navLabel(item) {
  if (item.labelKey.startsWith("nav.")) return t(item.labelKey);
  return t(item.labelKey);
}

function navSub(item) {
  if (item.subKey) return t(item.subKey);
  return item.sub || "";
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/"/g, "&quot;");
}

function formatBytes(bytes) {
  if (bytes == null || Number.isNaN(bytes)) return "—";
  const n = Number(bytes);
  if (n === 0) return "0 B";
  const units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
  let v = n;
  let u = 0;
  while (v >= 1024 && u < units.length - 1) {
    v /= 1024;
    u += 1;
  }
  return u === 0 ? `${n} ${units[u]}` : `${v.toFixed(2)} ${units[u]}`;
}

function applyTheme(themeOrId) {
  const theme =
    typeof themeOrId === "object"
      ? themeOrId
      : state.themes.find((x) => x.id === themeOrId) || state.themes[0];
  if (!theme) return;
  const root = document.documentElement;
  root.style.setProperty("--oed-bg", theme.window);
  root.style.setProperty("--oed-sidebar", theme.window);
  root.style.setProperty("--oed-header", theme.panel);
  root.style.setProperty("--oed-card", theme.panel);
  root.style.setProperty("--oed-accent", theme.accent);
  root.style.setProperty("--oed-accent-soft", `${theme.accent}26`);
  root.style.setProperty("--oed-accent-border", `${theme.accent}55`);
  root.style.setProperty("--oed-text", theme.text);
  state.theme = theme.id;
  localStorage.setItem("lpmr.theme", theme.id);
}

async function api(path, opts) {
  const res = await fetch(path, opts);
  let data;
  try {
    data = await res.json();
  } catch {
    throw new Error(res.statusText || "Invalid response");
  }
  if (!res.ok) throw new Error(data.error || res.statusText);
  return data;
}

function flattenDevices(nodes, depth = 0) {
  const out = [];
  for (const n of nodes || []) {
    out.push({ ...n, depth });
    out.push(...flattenDevices(n.children, depth + 1));
  }
  return out;
}

function showToast(msg) {
  state.toast = msg;
  renderToast();
  setTimeout(() => {
    state.toast = null;
    renderToast();
  }, 3500);
}

function renderToast() {
  let el = document.getElementById("lpmrToast");
  if (!state.toast) {
    if (el) el.remove();
    return;
  }
  if (!el) {
    el = document.createElement("div");
    el.id = "lpmrToast";
    el.className = "toast";
    document.body.appendChild(el);
  }
  el.textContent = state.toast;
}

function tabTitle() {
  const item = NAV.find((n) => n.id === state.tab);
  if (!item) return t("app.title");
  return `${navLabel(item)} — ${navSub(item)}`;
}

function deviceCount() {
  return flattenDevices(state.devices).length;
}

function renderShell() {
  const app = document.getElementById("app");
  if (!app) return;
  const langMeta = LANG_OPTIONS.find((l) => l.code === state.lang) || LANG_OPTIONS[0];
  const flatCount = deviceCount();

  app.innerHTML = `
    <div class="mobile-backdrop ${state.mobileNavOpen ? "open" : ""}" id="mobileBackdrop"></div>
    <div class="app-shell">
      <aside class="sidebar ${state.mobileNavOpen ? "open" : ""}" id="sidebar">
        <div class="sidebar-brand">
          <div class="brand-row">
            <div style="display:flex;align-items:center;gap:0.65rem">
              <div class="brand-logo">${ICONS.layers}</div>
              <div>
                <div class="brand-title">LPMR <span class="brand-badge">Enterprise</span></div>
                <p class="brand-sub">Linux Storage Director</p>
              </div>
            </div>
          </div>
        </div>
        <nav class="sidebar-nav" aria-label="Main">
          ${NAV.map(
            (item) => `
            <button type="button" class="nav-item ${state.tab === item.id ? "active" : ""}" data-tab="${item.id}">
              ${ICONS[item.icon] || ""}
              <div>
                <div class="nav-label">${escapeHtml(navLabel(item))}</div>
                <div class="nav-sub">${escapeHtml(navSub(item))}</div>
              </div>
            </button>`
          ).join("")}
        </nav>
        <div class="sidebar-footer">
          <div class="status-card">
            <div class="status-row">
              <span style="color:var(--oed-muted)">Engine</span>
              <span class="status-pill">Rust Axum</span>
            </div>
            <div class="status-row" style="color:var(--oed-muted-dim);font-family:var(--font-mono)">
              <span>Tools</span><span>${state.tools.length} installed</span>
            </div>
          </div>
        </div>
      </aside>
      <div class="main-col">
        <header class="top-header">
          <div class="header-left">
            <button type="button" class="btn btn-icon btn-ghost menu-toggle" id="menuBtn" aria-label="Menu">${ICONS.grid}</button>
            <span class="header-crumb">Storage Node</span>
            <span class="header-crumb">/</span>
            <h1 class="header-title">${escapeHtml(tabTitle())}</h1>
            <span class="header-badge">${flatCount} devices</span>
          </div>
          <div class="header-actions">
            <div class="lang-menu" id="langMenu">
              <button type="button" class="btn" id="langBtn">
                <span>${langMeta.flag}</span>
                <span class="mono" style="font-size:11px">${langMeta.code.toUpperCase()}</span>
              </button>
              <div class="lang-dropdown" id="langDropdown" style="display:${state.langMenuOpen ? "block" : "none"}">
                <div class="lang-dropdown-head">${escapeHtml(t("lang.label"))}</div>
                ${LANG_OPTIONS.map(
                  (l) => `
                  <button type="button" class="lang-option ${l.code === state.lang ? "active" : ""}" data-lang="${l.code}">
                    <span>${l.flag} ${escapeHtml(l.native)}</span>
                    ${l.code === state.lang ? "✓" : ""}
                  </button>`
                ).join("")}
              </div>
            </div>
            <button type="button" class="btn btn-sky" id="settingsBtn">${ICONS.settings}<span class="hide-sm">Settings</span></button>
            <button type="button" class="btn btn-icon btn-ghost" id="refreshBtn" title="${escapeHtml(t("action.refresh"))}" ${state.refreshing ? "disabled" : ""}>
              ${ICONS.refresh}
            </button>
          </div>
        </header>
        <div class="banner-warn">${escapeHtml(t("warn.privilege"))}</div>
        <main class="viewport" id="viewport"></main>
      </div>
    </div>
    <div id="modalRoot"></div>`;

  document.querySelectorAll(".nav-item").forEach((el) => {
    el.onclick = () => {
      state.tab = el.dataset.tab;
      state.mobileNavOpen = false;
      document.querySelectorAll(".nav-item").forEach((n) => n.classList.toggle("active", n.dataset.tab === state.tab));
      document.getElementById("sidebar")?.classList.remove("open");
      document.getElementById("mobileBackdrop")?.classList.remove("open");
      document.querySelector(".header-title").textContent = tabTitle();
      renderViewport();
    };
  });
  document.getElementById("menuBtn")?.addEventListener("click", () => {
    state.mobileNavOpen = !state.mobileNavOpen;
    document.getElementById("sidebar")?.classList.toggle("open", state.mobileNavOpen);
    document.getElementById("mobileBackdrop")?.classList.toggle("open", state.mobileNavOpen);
  });
  document.getElementById("mobileBackdrop")?.addEventListener("click", () => {
    state.mobileNavOpen = false;
    document.getElementById("sidebar")?.classList.remove("open");
    document.getElementById("mobileBackdrop")?.classList.remove("open");
  });
  document.getElementById("refreshBtn")?.addEventListener("click", () => refreshAll());
  document.getElementById("settingsBtn")?.addEventListener("click", () => {
    state.settingsOpen = true;
    renderModal();
  });
  document.getElementById("langBtn")?.addEventListener("click", (e) => {
    e.stopPropagation();
    state.langMenuOpen = !state.langMenuOpen;
    const dd = document.getElementById("langDropdown");
    if (dd) dd.style.display = state.langMenuOpen ? "block" : "none";
  });
  document.querySelectorAll(".lang-option").forEach((el) => {
    el.onclick = async () => {
      state.lang = el.dataset.lang;
      state.langMenuOpen = false;
      localStorage.setItem("lpmr.lang", state.lang);
      const meta = LANG_OPTIONS.find((l) => l.code === state.lang);
      document.documentElement.dir = meta?.dir || "ltr";
      await loadI18n();
      renderShell();
      renderViewport();
      bindGlobalClick();
    };
  });
  renderViewport();
}

function bindGlobalClick() {
  document.addEventListener("click", () => {
    if (state.langMenuOpen) {
      state.langMenuOpen = false;
      const dd = document.getElementById("langDropdown");
      if (dd) dd.style.display = "none";
    }
  });
}

function renderModal() {
  const root = document.getElementById("modalRoot");
  if (!root) return;
  if (!state.settingsOpen) {
    root.innerHTML = "";
    return;
  }
  root.innerHTML = `
    <div class="modal-backdrop" id="modalBackdrop">
      <div class="modal" role="dialog" aria-labelledby="settingsTitle">
        <div class="modal-head">
          <div>
            <h2 id="settingsTitle" style="margin:0;font-size:1rem">${escapeHtml(t("theme.label"))} & ${escapeHtml(t("lang.label"))}</h2>
            <p style="margin:0.35rem 0 0;font-size:0.75rem;color:var(--oed-muted)">OpenZFS Enterprise Director style appearance</p>
          </div>
          <button type="button" class="btn btn-icon btn-ghost" id="closeModal">✕</button>
        </div>
        <div class="modal-body">
          <p style="font-size:0.6875rem;text-transform:uppercase;color:var(--oed-muted);letter-spacing:0.06em">${escapeHtml(t("theme.label"))}</p>
          <div class="theme-grid" style="margin:0.5rem 0 1rem">
            ${state.themes
              .map(
                (th) => `
              <button type="button" class="theme-card ${th.id === state.theme ? "active" : ""}" data-theme="${th.id}">
                <div class="theme-card-preview" style="background:linear-gradient(135deg,${th.window},${th.accent})"></div>
                <div class="theme-card-name">${escapeHtml(th.name)}</div>
              </button>`
              )
              .join("")}
          </div>
          <button type="button" class="btn btn-primary" id="closeModal2">Done</button>
        </div>
      </div>
    </div>`;
  const close = () => {
    state.settingsOpen = false;
    renderModal();
  };
  document.getElementById("modalBackdrop")?.addEventListener("click", (e) => {
    if (e.target.id === "modalBackdrop") close();
  });
  const closeBtn = document.getElementById("closeModal");
  const closeBtn2 = document.getElementById("closeModal2");
  if (closeBtn) closeBtn.onclick = close;
  if (closeBtn2) closeBtn2.onclick = close;
  document.querySelectorAll(".theme-card").forEach((el) => {
    el.onclick = () => {
      applyTheme(el.dataset.theme);
      document.querySelectorAll(".theme-card").forEach((c) => c.classList.toggle("active", c.dataset.theme === state.theme));
      showToast(`Theme: ${state.themes.find((x) => x.id === state.theme)?.name || state.theme}`);
    };
  });
}

function renderViewport() {
  const vp = document.getElementById("viewport");
  if (!vp) return;
  if (state.tab === "dashboard") vp.innerHTML = renderDashboard();
  else if (state.tab === "devices") vp.innerHTML = renderDevicesView();
  else if (state.tab === "lvm") vp.innerHTML = renderLvmView();
  else if (state.tab === "images") vp.innerHTML = renderImagesView();
  else if (state.tab === "proxmox") vp.innerHTML = renderProxmoxView();
  else if (state.tab === "tools") vp.innerHTML = renderToolsView();
  else if (state.tab === "logs") vp.innerHTML = renderLogsView();
  bindViewportEvents();
}

function renderDashboard() {
  const flat = flattenDevices(state.devices);
  const vgCount = state.lvm?.vgs?.length || 0;
  const lvCount = state.lvm?.lvs?.length || 0;
  return `
    <div class="grid-stats">
      <div class="stat-card"><h3>Block devices</h3><div class="stat-value">${flat.length}</div><div class="stat-meta">Disks, partitions, loops</div></div>
      <div class="stat-card"><h3>LVM</h3><div class="stat-value">${vgCount}<span style="font-size:0.875rem;color:var(--oed-muted)"> VG</span></div><div class="stat-meta">${lvCount} logical volumes</div></div>
      <div class="stat-card"><h3>System tools</h3><div class="stat-value">${state.tools.length}</div><div class="stat-meta">parted · qemu-img · lvm …</div></div>
      <div class="stat-card"><h3>Proxmox</h3><div class="stat-value">${state.proxmox?.available ? "Online" : "N/A"}</div><div class="stat-meta">${state.proxmox?.available ? (state.proxmox.storages?.length || 0) + " storages" : "pvesm not detected"}</div></div>
    </div>
    <div class="card">
      <div class="card-head">Quick actions</div>
      <div class="card-body form-row">
        <button type="button" class="btn btn-primary" data-goto="devices">Manage devices</button>
        <button type="button" class="btn btn-sky" data-goto="images">Inspect VM image</button>
        <button type="button" class="btn" data-action="refresh">Refresh telemetry</button>
      </div>
    </div>`;
}

function renderDevicesView() {
  const flat = flattenDevices(state.devices);
  const dev = flat.find((d) => d.path === state.selected);
  return `
    <div class="split-view">
      <div class="card">
        <div class="card-head">${escapeHtml(t("nav.devices"))}</div>
        <div class="card-body">
          ${flat.length ? flat.map((d) => `
            <button type="button" class="device-row ${state.selected === d.path ? "selected" : ""}" data-path="${escapeHtml(d.path)}" style="padding-left:${8 + d.depth * 12}px">
              <div class="device-name">${escapeHtml(d.name)}</div>
              <div class="device-meta">${formatBytes(d.size_bytes)} · ${escapeHtml(d.fstype || "unformatted")}</div>
            </button>`).join("") : `<p class="empty">No devices — click Refresh</p>`}
        </div>
      </div>
      <div style="display:flex;flex-direction:column;gap:0.75rem">
        <div class="card">
          <div class="card-head">${escapeHtml(t("panel.details"))}</div>
          <div class="card-body">${dev ? deviceDetailHtml(dev) : `<p class="empty">Select a device</p>`}</div>
        </div>
        <div class="card">
          <div class="card-head">${escapeHtml(t("nav.operations"))}</div>
          <div class="card-body">${opsFormHtml()}</div>
        </div>
      </div>
    </div>`;
}

function deviceDetailHtml(dev) {
  return `
    <div class="kv">
      <div class="k">Path</div><div class="v mono">${escapeHtml(dev.path)}</div>
      <div class="k">Size</div><div class="v">${formatBytes(dev.size_bytes)}</div>
      <div class="k">Model</div><div class="v">${escapeHtml(dev.model || "—")}</div>
      <div class="k">Filesystem</div><div class="v">${escapeHtml(dev.fstype || "—")}</div>
      <div class="k">Transport</div><div class="v">${escapeHtml(dev.transport || "—")}</div>
      <div class="k">Mount</div><div class="v">${escapeHtml((dev.mountpoints || []).filter(Boolean).join(", ") || "—")}</div>
    </div>`;
}

function opsFormHtml() {
  const dev = state.selected || "";
  return `
    <div class="form-stack">
      <label class="checkbox-row"><input type="checkbox" id="dryRunChk" ${state.dryRun ? "checked" : ""}/> ${escapeHtml(t("action.dry_run"))}</label>
      <label>Device<input id="opDevice" value="${escapeHtml(dev)}"/></label>
      <label>Mount point<input id="opMount" placeholder="/mnt/data"/></label>
      <div class="form-row">
        <button type="button" class="btn btn-primary" id="opMountBtn">Mount</button>
        <button type="button" class="btn" id="opUmountBtn">Unmount</button>
      </div>
      <label>Convert destination<input id="opDest" placeholder="/path/out.qcow2"/></label>
      <button type="button" class="btn btn-sky" id="opConvertBtn">Convert → qcow2</button>
    </div>`;
}

function renderLvmView() {
  const l = state.lvm || { vgs: [], lvs: [], pvs: [] };
  return `
    <div class="split-view">
      <div class="card"><div class="card-head">Volume groups</div><div class="card-body"><ul>${(l.vgs || []).map((v) => `<li class="mono">${escapeHtml(v.name)} — ${escapeHtml(v.size)} (free ${escapeHtml(v.free)})</li>`).join("") || "<li class='empty'>None</li>"}</ul></div></div>
      <div class="card"><div class="card-head">Logical volumes</div><div class="card-body"><ul>${(l.lvs || []).map((v) => `<li class="mono">${escapeHtml(v.vg)}/${escapeHtml(v.name)} — ${escapeHtml(v.size)}</li>`).join("") || "<li class='empty'>None</li>"}</ul></div></div>
    </div>`;
}

function renderImagesView() {
  return `
    <div class="split-view">
      <div class="card">
        <div class="card-head">${escapeHtml(t("nav.images"))}</div>
        <div class="card-body form-stack">
          <label>Image path<input id="imgPath" value="${escapeHtml(state.imagePath)}" placeholder="/var/lib/vz/images/100/vm-100-disk-0.qcow2"/></label>
          <button type="button" class="btn btn-primary" id="imgInfoBtn">Inspect (qemu-img)</button>
        </div>
      </div>
      <div class="card"><div class="card-head">Metadata</div><div class="card-body">${state.imageInfo ? `<pre class="mono">${escapeHtml(JSON.stringify(state.imageInfo, null, 2))}</pre>` : `<p class="empty">No image loaded</p>`}</div></div>
    </div>`;
}

function renderProxmoxView() {
  if (!state.proxmox?.available) {
    return `<div class="card"><div class="card-body"><p class="empty">Not a Proxmox node — install <code>pvesm</code> or run on PVE.</p></div></div>`;
  }
  return `
    <div class="split-view">
      <div class="card"><div class="card-head">Storages</div><div class="card-body">
        ${(state.proxmox.storages || []).map((s) => `
          <button type="button" class="device-row" data-storage="${escapeHtml(s.name)}">
            <div class="device-name">${escapeHtml(s.name)}</div>
            <div class="device-meta">${escapeHtml(s.storage_type)} · ${escapeHtml(s.available)} free</div>
          </button>`).join("")}
      </div></div>
      <div class="card"><div class="card-head">Volumes</div><div class="card-body">${state.proxmoxSelected ? `<pre class="mono">${escapeHtml(JSON.stringify(state.proxmoxSelected, null, 2))}</pre>` : `<p class="empty">Select a storage</p>`}</div></div>
    </div>`;
}

function renderToolsView() {
  return `
    <div class="card">
      <div class="card-head">${escapeHtml(t("nav.tools"))}</div>
      <div class="card-body">
        ${state.tools.length ? `<ul>${state.tools.map((tool) => `<li style="margin-bottom:0.65rem"><strong class="mono">${escapeHtml(tool.name)}</strong><br/><span class="device-meta">${escapeHtml(tool.purpose)}</span></li>`).join("")}</ul>` : `<p class="empty">Run <code>lpmr doctor</code></p>`}
      </div>
    </div>`;
}

function renderLogsView() {
  return `
    <div class="card">
      <div class="card-head">${escapeHtml(t("panel.log"))}</div>
      <div class="card-body log-view">
        ${state.log.length ? state.log.map((l) => `<div class="log-line">${escapeHtml(l)}</div>`).join("") : `<p class="empty">No events yet</p>`}
      </div>
    </div>`;
}

function bindViewportEvents() {
  document.querySelectorAll(".device-row[data-path]").forEach((el) => {
    el.onclick = () => {
      state.selected = el.dataset.path;
      if (state.tab === "devices") renderViewport();
    };
  });
  document.querySelectorAll("[data-goto]").forEach((el) => {
    el.onclick = () => {
      state.tab = el.dataset.goto;
      renderShell();
    };
  });
  document.querySelector("[data-action=refresh]")?.addEventListener("click", () => refreshAll());
  document.getElementById("dryRunChk")?.addEventListener("change", (e) => {
    state.dryRun = e.target.checked;
  });
  document.getElementById("opMountBtn")?.addEventListener("click", () => runOp("Mount"));
  document.getElementById("opUmountBtn")?.addEventListener("click", () => runOp("Unmount"));
  document.getElementById("opConvertBtn")?.addEventListener("click", () => runOp("ConvertImage"));
  document.getElementById("imgInfoBtn")?.addEventListener("click", () => loadImageInfo());
  document.getElementById("imgPath")?.addEventListener("change", (e) => {
    state.imagePath = e.target.value;
  });
  document.querySelectorAll("[data-storage]").forEach((el) => {
    el.onclick = () => loadProxmoxVolumes(el.dataset.storage);
  });
}

async function loadImageInfo() {
  state.imagePath = document.getElementById("imgPath")?.value?.trim() || "";
  if (!state.imagePath) {
    showToast("Enter an image path");
    return;
  }
  try {
    state.imageInfo = await api(`/api/qemu/info?path=${encodeURIComponent(state.imagePath)}`);
    pushLog(`Image: ${state.imageInfo.format}`);
    showToast(`Format: ${state.imageInfo.format}`);
    renderViewport();
  } catch (e) {
    showToast(e.message);
    pushLog(`Image error: ${e.message}`);
    if (state.tab === "logs") renderViewport();
  }
}

async function loadProxmoxVolumes(storage) {
  try {
    state.proxmoxSelected = await api(`/api/proxmox/storage/${encodeURIComponent(storage)}/volumes`);
    pushLog(`Proxmox ${storage}: ${(state.proxmoxSelected.volumes || []).length} volumes`);
    renderViewport();
  } catch (e) {
    showToast(e.message);
  }
}

async function runOp(kind) {
  const device = document.getElementById("opDevice")?.value?.trim();
  const mount_point = document.getElementById("opMount")?.value?.trim();
  const dest = document.getElementById("opDest")?.value?.trim();
  const body = { kind, dry_run: state.dryRun, device, mount_point };
  if (kind === "ConvertImage") {
    body.source = state.imagePath || device;
    body.dest = dest;
  }
  try {
    const result = await api("/api/operations", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body),
    });
    const msg = result.message === "dry-run" ? `[dry-run] ${result.command}` : result.command;
    pushLog(msg);
    showToast(msg);
  } catch (e) {
    showToast(e.message);
    pushLog(`Operation failed: ${e.message}`);
  }
}

async function loadI18n() {
  const bundle = await api(`/api/i18n/${state.lang}`);
  state.strings = bundle.strings;
  state.strings["nav.dashboard"] = state.strings["nav.dashboard"] || "Dashboard";
  state.strings["nav.devices.sub"] = state.strings["nav.devices.sub"] || "Local disks & partitions";
  document.documentElement.lang = state.lang;
}

async function loadThemes() {
  const data = await api("/api/themes");
  state.themes = data.themes || [];
  if (!state.themes.some((x) => x.id === state.theme)) {
    state.theme = state.themes.find((x) => x.id === "slate-pro")?.id || state.themes[0]?.id;
  }
}

function pushLog(msg) {
  const ts = new Date().toLocaleTimeString();
  state.log.unshift(`[${ts}] ${msg}`);
  state.log = state.log.slice(0, 300);
}

async function refreshAll() {
  state.refreshing = true;
  document.getElementById("refreshBtn")?.setAttribute("disabled", "true");
  pushLog("Refreshing…");
  try {
    state.devices = (await api("/api/devices")).devices || [];
  } catch (e) {
    pushLog(`Devices: ${e.message}`);
  }
  try {
    state.lvm = await api("/api/lvm");
  } catch {
    state.lvm = { vgs: [], lvs: [], pvs: [] };
  }
  try {
    state.tools = (await api("/api/tools")).tools || [];
  } catch (e) {
    pushLog(`Tools: ${e.message}`);
  }
  try {
    state.proxmox = await api("/api/proxmox/storages");
  } catch {
    state.proxmox = { available: false, storages: [] };
  }
  if (state.devices.length && !state.selected) {
    state.selected = flattenDevices(state.devices)[0]?.path || null;
  }
  state.refreshing = false;
  document.getElementById("refreshBtn")?.removeAttribute("disabled");
  pushLog("Refresh complete.");
  showToast("Telemetry updated");
  const badge = document.querySelector(".header-badge");
  if (badge) badge.textContent = `${deviceCount()} devices`;
  renderViewport();
  const footerTools = document.querySelector(".sidebar-footer .status-row span:last-child");
  if (footerTools) footerTools.textContent = `${state.tools.length} installed`;
}

(async function boot() {
  try {
    await loadThemes();
    applyTheme(state.theme);
    await loadI18n();
    const meta = LANG_OPTIONS.find((l) => l.code === state.lang);
    document.documentElement.dir = meta?.dir || "ltr";
    renderShell();
    bindGlobalClick();
    await refreshAll();
  } catch (e) {
    document.getElementById("app").innerHTML = `<div class="empty" style="padding:2rem;color:#fb7185">Failed to start: ${escapeHtml(e.message)}</div>`;
  }
})();
