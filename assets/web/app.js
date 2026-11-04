const LANG_OPTIONS = [
  { code: "en", label: "English" },
  { code: "es", label: "Español" },
  { code: "zh", label: "中文" },
  { code: "hi", label: "हिन्दी" },
  { code: "ar", label: "العربية" },
  { code: "fr", label: "Français" },
];

const state = {
  lang: localStorage.getItem("lpmr.lang") || document.getElementById("app")?.dataset.lang || "en",
  theme: localStorage.getItem("lpmr.theme") || document.getElementById("app")?.dataset.theme || "midnight-enterprise",
  tab: "devices",
  selected: null,
  strings: {},
  themes: [],
  devices: [],
  lvm: null,
  tools: [],
  proxmox: null,
  imagePath: "",
  imageInfo: null,
  dryRun: true,
  log: [],
};

function t(key) {
  return state.strings[key] || key;
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
  root.style.setProperty("--lpmr-window", theme.window);
  root.style.setProperty("--lpmr-panel", theme.panel);
  root.style.setProperty("--lpmr-accent", theme.accent);
  root.style.setProperty("--lpmr-text", theme.text);
  root.style.setProperty("--lpmr-border", theme.border);
  root.style.setProperty("--lpmr-success", theme.success);
  root.style.setProperty("--lpmr-warning", theme.warning);
  root.style.setProperty("--lpmr-danger", theme.danger);
  root.style.setProperty("--lpmr-header", theme.panel);
  state.theme = theme.id;
  localStorage.setItem("lpmr.theme", theme.id);
  updateThemeControls();
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

function currentTheme() {
  return state.themes.find((x) => x.id === state.theme) || state.themes[0];
}

function themeSelectHtml() {
  return state.themes
    .map(
      (th) =>
        `<option value="${th.id}" ${th.id === state.theme ? "selected" : ""}>${escapeHtml(th.name)}</option>`
    )
    .join("");
}

function themeSwatchesHtml() {
  return state.themes
    .map(
      (th) =>
        `<button type="button" class="swatch ${th.id === state.theme ? "active" : ""}" data-theme-id="${th.id}" title="${escapeHtml(th.name)}" style="background:linear-gradient(135deg,${th.window} 0%,${th.accent} 100%)"></button>`
    )
    .join("");
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/"/g, "&quot;");
}

function updateThemeControls() {
  const sel = document.getElementById("themeSel");
  if (sel) sel.value = state.theme;
  const label = document.getElementById("themeName");
  const th = currentTheme();
  if (label && th) label.textContent = th.name;
  document.querySelectorAll(".swatch").forEach((el) => {
    el.classList.toggle("active", el.dataset.themeId === state.theme);
  });
}

function render() {
  const app = document.getElementById("app");
  if (!app) return;
  const tabs = [
    ["devices", t("nav.devices")],
    ["lvm", t("nav.lvm")],
    ["images", t("nav.images")],
    ["proxmox", t("nav.proxmox")],
    ["tools", t("nav.tools")],
  ];
  const th = currentTheme();
  app.innerHTML = `
    <div class="shell">
      <header class="header">
        <h1>${escapeHtml(t("app.title"))}</h1>
        <div class="spacer"></div>
        <div class="control-group">
          <span>${escapeHtml(t("lang.label"))}</span>
          <select id="langSel" aria-label="Language">
            ${LANG_OPTIONS.map(
              (l) =>
                `<option value="${l.code}" ${l.code === state.lang ? "selected" : ""}>${l.label}</option>`
            ).join("")}
          </select>
        </div>
        <div class="control-group">
          <span>${escapeHtml(t("theme.label"))}</span>
          <select id="themeSel" aria-label="Enterprise theme">${themeSelectHtml()}</select>
          <div class="theme-preview" id="themePreview">
            ${themeSwatchesHtml()}
            <span class="swatch-label" id="themeName">${th ? escapeHtml(th.name) : ""}</span>
          </div>
        </div>
        <button type="button" class="primary" id="refreshBtn">${escapeHtml(t("action.refresh"))}</button>
      </header>
      <div class="warn">${escapeHtml(t("warn.privilege"))}</div>
      <div class="workspace">
        <section class="panel" aria-label="Navigator">
          <div class="tabs" role="tablist">${tabs
            .map(
              ([id, label]) =>
                `<div class="tab ${state.tab === id ? "active" : ""}" role="tab" data-tab="${id}" tabindex="0">${escapeHtml(label)}</div>`
            )
            .join("")}</div>
          <div class="panel-body" id="leftPanel"></div>
        </section>
        <section class="panel">
          <div class="panel-title">${escapeHtml(t("panel.details"))}</div>
          <div class="panel-body" id="detailPanel"></div>
        </section>
        <section class="panel">
          <div class="panel-title">${escapeHtml(t("nav.operations"))}</div>
          <div class="panel-body" id="opsPanel"></div>
        </section>
      </div>
      <div class="log" id="logPanel" aria-live="polite"></div>
    </div>`;

  document.querySelectorAll(".tab").forEach((el) => {
    el.onclick = () => selectTab(el.dataset.tab);
    el.onkeydown = (e) => {
      if (e.key === "Enter" || e.key === " ") selectTab(el.dataset.tab);
    };
  });
  document.getElementById("langSel").onchange = async (e) => {
    state.lang = e.target.value;
    localStorage.setItem("lpmr.lang", state.lang);
    await loadI18n();
    render();
  };
  document.getElementById("themeSel").onchange = (e) => {
    applyTheme(e.target.value);
  };
  document.querySelectorAll(".swatch").forEach((el) => {
    el.onclick = () => applyTheme(el.dataset.themeId);
  });
  document.getElementById("refreshBtn").onclick = () => refreshAll();
  renderMainPanels();
  renderDetail();
  renderOpsPanel();
  renderLog();
}

function selectTab(id) {
  state.tab = id;
  document.querySelectorAll(".tab").forEach((el) => el.classList.toggle("active", el.dataset.tab === id));
  renderMainPanels();
}

function renderMainPanels() {
  const left = document.getElementById("leftPanel");
  if (!left) return;
  if (state.tab === "devices") {
    const flat = flattenDevices(state.devices);
    if (!flat.length) {
      left.innerHTML = `<p class="empty">No block devices found. Click Refresh or check permissions.</p>`;
      return;
    }
    left.innerHTML = flat
      .map(
        (d) => `
      <div class="tree-item ${state.selected === d.path ? "selected" : ""}" data-path="${escapeHtml(d.path)}" style="padding-left:${8 + d.depth * 14}px">
        <div class="device-name mono">${escapeHtml(d.name)}</div>
        <div class="device-meta">${formatBytes(d.size_bytes)} · ${escapeHtml(d.fstype || "no filesystem")}</div>
      </div>`
      )
      .join("");
    left.querySelectorAll(".tree-item").forEach((el) => {
      el.onclick = () => {
        state.selected = el.dataset.path;
        renderDetail();
        renderMainPanels();
        renderOpsPanel();
      };
    });
  } else if (state.tab === "lvm") {
    const l = state.lvm;
    if (!l) {
      left.innerHTML = `<p class="empty">Loading LVM…</p>`;
      return;
    }
    left.innerHTML = `
      <p><span class="badge">${(l.vgs || []).length} VG</span> <span class="badge">${(l.lvs || []).length} LV</span></p>
      <h4>Volume groups</h4>
      <ul>${(l.vgs || [])
        .map((v) => `<li class="mono">${escapeHtml(v.name)} — ${escapeHtml(v.size)} (free ${escapeHtml(v.free)})</li>`)
        .join("") || "<li class='empty'>None</li>"}</ul>
      <h4>Logical volumes</h4>
      <ul>${(l.lvs || [])
        .map((v) => `<li class="mono">${escapeHtml(v.vg)}/${escapeHtml(v.name)} — ${escapeHtml(v.size)}</li>`)
        .join("") || "<li class='empty'>None</li>"}</ul>`;
  } else if (state.tab === "tools") {
    left.innerHTML =
      state.tools.length > 0
        ? `<ul>${state.tools
            .map((tool) => `<li><strong>${escapeHtml(tool.name)}</strong><br/><span class="device-meta">${escapeHtml(tool.purpose)}</span></li>`)
            .join("")}</ul>`
        : `<p class="empty">No tools detected. Run <code>lpmr doctor</code>.</p>`;
  } else if (state.tab === "images") {
    left.innerHTML = `
      <div class="ops-form">
        <label>Image path<input id="imgPath" type="text" placeholder="/var/lib/vz/images/100/vm-100-disk-0.qcow2" value="${escapeHtml(state.imagePath)}"/></label>
        <button type="button" class="primary" id="imgInfoBtn">Inspect image</button>
      </div>
      <div id="imgSummary"></div>`;
    document.getElementById("imgInfoBtn").onclick = () => loadImageInfo();
    document.getElementById("imgPath").onchange = (e) => {
      state.imagePath = e.target.value;
    };
    renderImageSummary(document.getElementById("imgSummary"));
  } else if (state.tab === "proxmox") {
    if (!state.proxmox?.available) {
      left.innerHTML = `<p class="empty"><span class="badge-warn badge">Not a Proxmox node</span> Install <code>pvesm</code> or open this panel on a PVE host.</p>`;
      return;
    }
    left.innerHTML = `<ul>${(state.proxmox.storages || [])
      .map(
        (s) =>
          `<li><button type="button" class="tree-item" data-storage="${escapeHtml(s.name)}" style="width:100%;text-align:left;border:none;background:transparent;color:inherit">
            <strong>${escapeHtml(s.name)}</strong> <span class="device-meta">${escapeHtml(s.storage_type)} · ${escapeHtml(s.available)} free</span>
          </button></li>`
      )
      .join("")}</ul>`;
    left.querySelectorAll("[data-storage]").forEach((el) => {
      el.onclick = () => loadProxmoxVolumes(el.dataset.storage);
    });
  }
}

function renderImageSummary(container) {
  if (!container || !state.imageInfo) return;
  const i = state.imageInfo;
  container.innerHTML = `
    <div class="kv" style="margin-top:12px">
      <div class="k">Format</div><div class="v">${escapeHtml(i.format)}</div>
      <div class="k">Virtual size</div><div class="v">${escapeHtml(String(i.virtual_size))}</div>
      <div class="k">Disk size</div><div class="v">${escapeHtml(String(i.disk_size))}</div>
    </div>`;
}

async function loadImageInfo() {
  const input = document.getElementById("imgPath");
  state.imagePath = input?.value?.trim() || "";
  if (!state.imagePath) {
    pushLog("Enter an image path first");
    return;
  }
  try {
    state.imageInfo = await api(`/api/qemu/info?path=${encodeURIComponent(state.imagePath)}`);
    pushLog(`Image info: ${state.imageInfo.format}`);
    renderMainPanels();
    renderDetail();
  } catch (e) {
    pushLog(`Image error: ${e.message}`);
  }
}

async function loadProxmoxVolumes(storage) {
  try {
    const data = await api(`/api/proxmox/storage/${encodeURIComponent(storage)}/volumes`);
    pushLog(`Proxmox ${storage}: ${(data.volumes || []).length} volumes`);
    state.proxmoxSelected = data;
    renderDetail();
  } catch (e) {
    pushLog(`Proxmox: ${e.message}`);
  }
}

function renderDetail() {
  const panel = document.getElementById("detailPanel");
  if (!panel) return;
  if (state.tab === "proxmox" && state.proxmoxSelected) {
    panel.innerHTML = `<pre class="mono">${escapeHtml(JSON.stringify(state.proxmoxSelected, null, 2))}</pre>`;
    return;
  }
  if (state.tab === "images" && state.imageInfo) {
    panel.innerHTML = `<pre class="mono">${escapeHtml(JSON.stringify(state.imageInfo, null, 2))}</pre>`;
    return;
  }
  const flat = flattenDevices(state.devices);
  const dev = flat.find((d) => d.path === state.selected);
  if (!dev) {
    panel.innerHTML = `<p class="empty">Select a device from the list to see details.</p>`;
    return;
  }
  panel.innerHTML = `
    <div class="kv">
      <div class="k">Path</div><div class="v mono">${escapeHtml(dev.path)}</div>
      <div class="k">Size</div><div class="v">${formatBytes(dev.size_bytes)}</div>
      <div class="k">Model</div><div class="v">${escapeHtml(dev.model || "—")}</div>
      <div class="k">Serial</div><div class="v">${escapeHtml(dev.serial || "—")}</div>
      <div class="k">Filesystem</div><div class="v">${escapeHtml(dev.fstype || "—")}</div>
      <div class="k">Partition type</div><div class="v">${escapeHtml(dev.parttype || "—")}</div>
      <div class="k">Transport</div><div class="v">${escapeHtml(dev.transport || "—")}</div>
      <div class="k">Rotational</div><div class="v">${dev.rota ? "Yes (HDD)" : "No (SSD/NVMe)"}</div>
      <div class="k">Mountpoints</div><div class="v">${escapeHtml((dev.mountpoints || []).filter(Boolean).join(", ") || "—")}</div>
    </div>`;
}

function renderOpsPanel() {
  const panel = document.getElementById("opsPanel");
  if (!panel) return;
  const dev = state.selected || "";
  panel.innerHTML = `
    <div class="ops-form">
      <label class="ops-row">
        <input type="checkbox" id="dryRunChk" ${state.dryRun ? "checked" : ""}/>
        <span>${escapeHtml(t("action.dry_run"))} (recommended)</span>
      </label>
      <label>Device path<input id="opDevice" value="${escapeHtml(dev)}" placeholder="/dev/sda1"/></label>
      <label>Mount point<input id="opMount" placeholder="/mnt/data"/></label>
      <div class="ops-row">
        <button type="button" id="opMountBtn">${escapeHtml(t("action.execute"))} mount</button>
        <button type="button" id="opUmountBtn">Unmount</button>
      </div>
      <label>Image convert — destination<input id="opDest" placeholder="/path/out.qcow2"/></label>
      <div class="ops-row">
        <button type="button" id="opConvertBtn">Convert → qcow2</button>
      </div>
    </div>`;
  document.getElementById("dryRunChk").onchange = (e) => {
    state.dryRun = e.target.checked;
  };
  document.getElementById("opMountBtn").onclick = () => runOp("Mount");
  document.getElementById("opUmountBtn").onclick = () => runOp("Unmount");
  document.getElementById("opConvertBtn").onclick = () => runOp("ConvertImage");
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
    pushLog(result.message === "dry-run" ? `[dry-run] ${result.command}` : `[ok] ${result.command}`);
  } catch (e) {
    pushLog(`Operation failed: ${e.message}`);
  }
}

async function loadI18n() {
  const bundle = await api(`/api/i18n/${state.lang}`);
  state.strings = bundle.strings;
  document.documentElement.lang = state.lang;
}

async function loadThemes() {
  const data = await api("/api/themes");
  state.themes = data.themes || [];
  if (!state.themes.some((t) => t.id === state.theme)) {
    state.theme = state.themes[0]?.id || "midnight-enterprise";
  }
}

async function refreshAll() {
  pushLog("Refreshing…");
  try {
    state.devices = (await api("/api/devices")).devices || [];
  } catch (e) {
    pushLog(`Devices: ${e.message}`);
  }
  try {
    state.lvm = await api("/api/lvm");
  } catch (e) {
    pushLog(`LVM: ${e.message}`);
    state.lvm = { vgs: [], lvs: [], pvs: [] };
  }
  try {
    state.tools = (await api("/api/tools")).tools || [];
  } catch (e) {
    pushLog(`Tools: ${e.message}`);
  }
  try {
    state.proxmox = await api("/api/proxmox/storages");
  } catch (e) {
    state.proxmox = { available: false, storages: [] };
  }
  if (state.devices.length && !state.selected) {
    const flat = flattenDevices(state.devices);
    state.selected = flat[0]?.path || null;
  }
  renderMainPanels();
  renderDetail();
  renderOpsPanel();
  pushLog("Refresh complete.");
}

function renderLog() {
  const log = document.getElementById("logPanel");
  if (!log) return;
  log.innerHTML = `<strong>${escapeHtml(t("panel.log"))}</strong>${state.log
    .map((l) => `<div class="log-line">${escapeHtml(l)}</div>`)
    .join("")}`;
}

function pushLog(msg) {
  const ts = new Date().toLocaleTimeString();
  state.log.unshift(`[${ts}] ${msg}`);
  state.log = state.log.slice(0, 200);
  renderLog();
}

(async function boot() {
  try {
    await loadThemes();
    applyTheme(state.theme);
    await loadI18n();
    render();
    await refreshAll();
  } catch (e) {
    document.getElementById("app").innerHTML = `<p style="padding:24px;color:#fda4af">Failed to start panel: ${escapeHtml(e.message)}</p>`;
  }
})();
