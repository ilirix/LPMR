/**
 * LPMR — Proxmox VE Storage Orchestrator (Raven // Baremetal UI)
 */
const BUILTIN_UI_THEMES = new Set(["orange", "obsidian", "arctic"]);
const VIEW_ORDER = ["disks", "vmimages", "lvm", "zfs", "health"];
const VIEW_LABELS = {
  disks: "Disks",
  vmimages: "VM Images",
  lvm: "LVM",
  zfs: "ZFS Pools",
  health: "Health Audit",
};
const LANGUAGES = [
  ["en", "English"],
  ["es", "Español"],
  ["zh", "中文"],
  ["hi", "हिन्दी"],
  ["ar", "العربية"],
  ["fr", "Français"],
];

const state = {
  view: "disks",
  lang: localStorage.getItem("lpmr.lang") || "en",
  uiTheme: localStorage.getItem("lpmr.ui-theme") || "orange",
  enterpriseThemes: [],
  lvm: { available: false, pvs: [], vgs: [], lvs: [] },
  selectedDisk: null,
  telemetry: null,
  devices: [],
  partitionMap: null,
  vmVolumes: [],
  zfs: { available: false, pools: [] },
  health: null,
  tools: [],
  dryRun: true,
  modalDryRun: true,
  log: [],
  modal: null,
  jobId: null,
  jobPct: 0,
  jobPolling: false,
  search: "",
  executeCsrf: null,
};

function esc(s) {
  return String(s).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/"/g, "&quot;");
}

function fmt(b) {
  if (!b) return "0 B";
  const u = ["B", "KiB", "MiB", "GiB", "TiB"];
  let v = b, i = 0;
  while (v >= 1024 && i < u.length - 1) { v /= 1024; i++; }
  return i ? `${v.toFixed(2)} ${u[i]}` : `${b} ${u[0]}`;
}

async function resolveProxmoxVolidToPath(volidOrPath) {
  const trimmed = (volidOrPath || "").trim();
  if (!trimmed) return "";
  if (trimmed.startsWith("/")) return trimmed;
  if (trimmed.includes(":")) {
    const r = await api(`/api/proxmox/resolve-path?volid=${encodeURIComponent(trimmed)}`);
    return r.path;
  }
  return trimmed;
}

async function ensureExecuteCsrf() {
  if (state.executeCsrf) return state.executeCsrf;
  const s = await fetch("/api/session").then((r) => r.json());
  state.executeCsrf = s.csrf;
  return state.executeCsrf;
}

async function api(path, opts) {
  opts = opts || {};
  const headers = { ...(opts.headers || {}) };
  if (opts.method === "POST" && opts.body) {
    let dryRun = true;
    try {
      const parsed = JSON.parse(opts.body);
      if (parsed.dry_run === false) dryRun = false;
    } catch {
      /* ignore */
    }
    if (!dryRun) {
      headers["X-LPMR-CSRF"] = await ensureExecuteCsrf();
    }
  }
  const r = await fetch(path, { ...opts, headers });
  const j = await r.json().catch(() => ({}));
  if (!r.ok) throw new Error(j.error || r.statusText);
  return j;
}

function log(line, cls) {
  const ts = new Date().toLocaleTimeString();
  state.log.unshift({ ts, line, cls });
  state.log = state.log.slice(0, 400);
  paintTerminal();
}

function paintTerminal() {
  const el = document.getElementById("termOut");
  if (!el) return;
  el.innerHTML = state.log
    .map((l) => `<div class="line-${l.cls || "info"}">[${esc(l.ts)}] ${esc(l.line)}</div>`)
    .join("");
}

function applyEnterprisePalette(theme) {
  const root = document.documentElement;
  root.style.setProperty("--bg", theme.window);
  root.style.setProperty("--panel", theme.panel);
  root.style.setProperty("--panel2", theme.panel);
  root.style.setProperty("--border", theme.border);
  root.style.setProperty("--text", theme.text);
  root.style.setProperty("--accent", theme.accent);
  root.style.setProperty("--accent2", theme.accent);
  root.style.setProperty("--green", theme.success);
  root.style.setProperty("--rose", theme.danger);
}

function setTheme(t) {
  state.uiTheme = t;
  localStorage.setItem("lpmr.ui-theme", t);
  const root = document.documentElement;
  if (BUILTIN_UI_THEMES.has(t)) {
    root.removeAttribute("style");
    root.setAttribute("data-ui-theme", t);
  } else {
    const ent = state.enterpriseThemes.find((th) => th.id === t);
    root.setAttribute("data-ui-theme", "custom");
    if (ent) applyEnterprisePalette(ent);
  }
  document.querySelectorAll(".theme-switch button").forEach((b) => {
    b.classList.toggle("on", b.dataset.theme === t);
  });
}

function navLinkClass(view) {
  return state.view === view ? "nav-link on" : "nav-link";
}

function themeButtonsHtml() {
  return [
    ["orange", "Orange"],
    ["obsidian", "Obsidian"],
    ["arctic", "Arctic"],
  ]
    .map(
      ([id, label]) =>
        `<button type="button" data-theme="${esc(id)}" class="${state.uiTheme === id ? "on" : ""}">${esc(label)}</button>`
    )
    .join("");
}

function switchView(view) {
  state.view = view;
  document.querySelectorAll("#tabBar button").forEach((b) => {
    b.classList.toggle("active", b.dataset.view === view);
  });
  document.querySelectorAll("[data-goto]").forEach((b) => {
    b.classList.toggle("on", b.dataset.goto === view);
  });
  paintMain();
}

function render() {
  const host = state.telemetry?.host;
  const app = document.getElementById("app");
  app.innerHTML = `
    <header class="top-brand">
      <span class="badge-raven">RAVEN // BAREMETAL</span>
      <h1>PROXMOX VE STORAGE ORCHESTRATOR<span class="badge-version">v0.1</span></h1>
      <div class="grow"></div>
      <input type="search" id="globalSearch" placeholder="Search nodes, pools, disks, VM images…" value="${esc(state.search)}"/>
      <button type="button" class="btn btn-ghost" id="btnTui">Open Terminal</button>
      <button type="button" class="btn btn-primary" id="btnCreateVol">+ Create Volume</button>
    </header>
    <nav class="tab-bar" id="tabBar">
      ${VIEW_ORDER.map((v) => `
        <button type="button" data-view="${v}" class="${state.view === v ? "active" : ""}">${VIEW_LABELS[v]}</button>`).join("")}
    </nav>
    <div class="metrics">
      <span class="metric-pill ok"><strong>${esc(host?.hostname || "node")}</strong> ONLINE · ${esc(host?.kernel || "linux")}</span>
      <span class="metric-pill"><strong>Rust RSS</strong> ${(host?.rss_mb ?? 0).toFixed(1)} MB</span>
      <span class="metric-pill"><strong>Devices</strong> ${state.devices.length}</span>
      <span class="metric-pill"><strong>Tools</strong> ${state.health?.tools_ok?.length ?? state.tools.length}</span>
      <span class="metric-pill metric-toggle ${state.dryRun ? "on" : ""}" id="dryToggle" title="Click to toggle dry-run">
        <strong>Dry-run</strong> ${state.dryRun ? "ON" : "OFF"}
      </span>
    </div>
    <div class="workspace">
      <aside class="col-nav">
        <h4>Storage infrastructure</h4>
        <button type="button" class="${navLinkClass("disks")}" data-goto="disks">Block Devices & NVMe</button>
        <button type="button" class="${navLinkClass("vmimages")}" data-goto="vmimages">Proxmox VM Volumes</button>
        <button type="button" class="${navLinkClass("lvm")}" data-goto="lvm">LVM Volume Groups</button>
        <button type="button" class="${navLinkClass("zfs")}" data-goto="zfs">ZFS Pools & Datasets</button>
        <button type="button" class="${navLinkClass("health")}" data-goto="health">Health & Dependencies</button>
        <h4>Operations</h4>
        <button type="button" class="nav-link" id="openVhdx">VHDX Convert & Resize</button>
        <button type="button" class="nav-link" id="openRaw">RAW / QCOW2 Pipeline</button>
        <h4>Localization & UI</h4>
        <select id="langSelect" class="lang-select" aria-label="Language">
          ${LANGUAGES.map(([c, label]) => `<option value="${c}" ${state.lang === c ? "selected" : ""}>${label}</option>`).join("")}
        </select>
        <div class="theme-switch">${themeButtonsHtml()}</div>
      </aside>
      <main class="col-main" id="mainView"></main>
      <aside class="col-rail">
        <div class="rail-section">
          <h4>System toolchain & CLI audit</h4>
          <div class="tool-grid" id="toolGrid"></div>
        </div>
        <div class="rail-section" style="flex:1;display:flex;flex-direction:column;min-height:0;border:none">
          <h4>Integrated log (raven-tui)</h4>
          <div class="terminal" id="termOut"></div>
        </div>
      </aside>
    </div>
    <div id="modalRoot"></div>`;

  document.querySelectorAll("#tabBar button").forEach((b) => {
    b.onclick = () => switchView(b.dataset.view);
  });
  document.querySelectorAll("[data-goto]").forEach((b) => {
    b.onclick = () => switchView(b.dataset.goto);
  });
  const dryToggle = document.getElementById("dryToggle");
  if (dryToggle) {
    dryToggle.onclick = () => {
      state.dryRun = !state.dryRun;
      dryToggle.classList.toggle("on", state.dryRun);
      dryToggle.innerHTML = `<strong>Dry-run</strong> ${state.dryRun ? "ON" : "OFF"}`;
      log(`Dry-run ${state.dryRun ? "enabled" : "disabled"}`, "warn");
    };
  }
  const langSel = document.getElementById("langSelect");
  if (langSel) {
    langSel.onchange = async () => {
      state.lang = langSel.value;
      localStorage.setItem("lpmr.lang", state.lang);
      document.documentElement.lang = state.lang;
      try {
        const bundle = await api(`/api/i18n/${state.lang}`);
        log(`Language: ${bundle.lang}`, "info");
      } catch (e) {
        log(e.message, "warn");
      }
    };
  }
  document.getElementById("btnTui").onclick = () => {
    log("Run on host: lpmr tui", "warn");
    alert("Native terminal UI: run `lpmr tui` on the Proxmox/Linux host.");
  };
  document.getElementById("btnCreateVol").onclick = () => openModal("create");
  document.getElementById("openVhdx").onclick = () => openModal("vhdx");
  document.getElementById("openRaw").onclick = () => openModal("raw");
  document.querySelectorAll(".theme-switch button").forEach((b) => {
    b.onclick = () => setTheme(b.dataset.theme);
  });
  document.getElementById("globalSearch").oninput = (e) => {
    state.search = e.target.value.toLowerCase();
    paintMain();
  };
  paintTools();
  paintMain();
  paintTerminal();
  renderModal();
}

function bind() {
  setTheme(state.uiTheme);
}

function paintTools() {
  const grid = document.getElementById("toolGrid");
  if (!grid) return;
  const ok = (state.health?.tools_ok || state.tools.map((t) => ({ name: t.name }))).slice(0, 12);
  const miss = (state.health?.tools_missing || []).slice(0, 6);
  let chips = [
    ...ok.map((t) => ({ name: t.name, ok: true })),
    ...miss.map((t) => ({ name: t.name, ok: false })),
  ];
  if (!chips.length) {
    chips = state.tools.slice(0, 12).map((t) => ({ name: t.name, ok: true }));
  }
  grid.innerHTML = chips
    .map(
      (t) =>
        `<div class="tool-chip ${t.ok ? "ok" : "err"}"><span>${esc(t.name)}</span><span>${t.ok ? "OK" : "MISS"}</span></div>`
    )
    .join("");
}

function paintMain() {
  const main = document.getElementById("mainView");
  if (!main) return;
  if (state.view === "disks") main.innerHTML = viewDisks();
  else if (state.view === "vmimages") main.innerHTML = viewVm();
  else if (state.view === "lvm") main.innerHTML = viewLvm();
  else if (state.view === "zfs") main.innerHTML = viewZfs();
  else main.innerHTML = viewHealth();
  bindMain();
}

function filterDisks(devs) {
  if (!state.search) return devs;
  return devs.filter((d) => JSON.stringify(d).toLowerCase().includes(state.search));
}

function viewDisks() {
  const disks = filterDisks(state.devices);
  const map = state.partitionMap;
  const segs = map?.segments || [];
  return `
    <div class="card">
      <div class="card-head">Partition allocation visualizer</div>
      <div class="card-body">
        <div style="font-family:var(--mono);font-size:12px;margin-bottom:8px">
          ${esc(map?.disk_path || state.selectedDisk || "—")} ${map?.model ? "· " + esc(map.model) : ""}
          ${map ? `· ${fmt(map.size_bytes)}` : ""}
        </div>
        <div class="part-bar">${segs.length ? segs.map((s) => `
          <div class="part-seg" style="width:${Math.max(s.pct, 3)}%" title="${esc(s.path)}">${esc(s.name)}</div>`).join("") : `<div class="part-seg" style="width:100%;background:var(--panel)">No partitions</div>`}
        </div>
        <div class="btn-row">
          <button type="button" class="btn btn-sm" data-op="Mount">Mount</button>
          <button type="button" class="btn btn-sm" data-op="FormatFilesystem">Format</button>
          <button type="button" class="btn btn-sm" data-op="Unmount">Unmount</button>
          <button type="button" class="btn btn-sm btn-primary" data-op="ConvertImage">Convert image</button>
        </div>
      </div>
    </div>
    <div class="card">
      <div class="card-head">Block devices</div>
      <div class="card-body">
        ${disks.map((d) => `
          <button type="button" class="btn btn-ghost" style="width:100%;margin-bottom:6px;text-align:left" data-disk="${esc(d.path)}">
            <strong>${esc(d.name)}</strong> <span style="color:var(--muted)">${fmt(d.size_bytes)}</span>
          </button>`).join("") || "<p style='color:var(--muted)'>No devices</p>"}
      </div>
    </div>`;
}

function viewVm() {
  const rows = (state.vmVolumes || []).filter((v) => !state.search || JSON.stringify(v).toLowerCase().includes(state.search));
  return `
    <div class="card">
      <div class="card-head">Proxmox VE & KVM volume orchestrator</div>
      <div class="card-body" style="padding:0">
        <table class="table">
          <thead><tr><th>VolID</th><th>Format</th><th>Size</th><th>Storage</th><th></th></tr></thead>
          <tbody>
            ${rows.length ? rows.map((v) => `
              <tr>
                <td>${esc(v.volid)}</td><td>${esc(v.format)}</td><td>${esc(v.size)}</td><td>${esc(v.storage)}</td>
                <td><button type="button" class="btn btn-sm" data-convert="${esc(v.volid)}" data-path="${esc(v.path || "")}">Convert</button></td>
              </tr>`).join("") : `<tr><td colspan="5" style="color:var(--muted)">No VM volumes (not on PVE or empty)</td></tr>`}
          </tbody>
        </table>
      </div>
    </div>`;
}

function viewLvm() {
  const snap = state.lvm || {};
  if (!snap.available) {
    return `
      <div class="card">
        <div class="card-head">LVM2 (pvs / vgs / lvs)</div>
        <div class="card-body">
          <p style="color:var(--muted)">LVM not detected. On Debian/Ubuntu: <code>apt install lvm2</code> or run <code>LPMR_INSTALL_DEPS=1 ./start.sh</code></p>
        </div>
      </div>`;
  }
  const vgs = snap.vgs || [];
  const lvs = snap.lvs || [];
  return `
    <div class="card">
      <div class="card-head">Volume groups</div>
      <div class="card-body">
        <table class="table">
          <thead><tr><th>VG</th><th>Size</th><th>Free</th><th>PVs</th><th>LVs</th></tr></thead>
          <tbody>
            ${vgs.length ? vgs.map((v) => `
              <tr><td>${esc(v.name)}</td><td>${esc(v.size)}</td><td>${esc(v.free)}</td><td>${esc(v.pvs)}</td><td>${esc(v.lvs)}</td></tr>`).join("")
              : `<tr><td colspan="5" style="color:var(--muted)">No volume groups</td></tr>`}
          </tbody>
        </table>
      </div>
    </div>
    <div class="card">
      <div class="card-head">Logical volumes</div>
      <div class="card-body" style="padding:0">
        <table class="table">
          <thead><tr><th>LV</th><th>VG</th><th>Size</th><th>Path</th></tr></thead>
          <tbody>
            ${lvs.length ? lvs.map((v) => `
              <tr><td>${esc(v.name)}</td><td>${esc(v.vg)}</td><td>${esc(v.size)}</td><td>${esc(v.path)}</td></tr>`).join("")
              : `<tr><td colspan="4" style="color:var(--muted)">No logical volumes</td></tr>`}
          </tbody>
        </table>
      </div>
    </div>`;
}

function viewZfs() {
  const pools = state.zfs.pools || [];
  return `
    <div class="card">
      <div class="card-head">ZFS pools</div>
      <div class="card-body">
        ${state.zfs.available ? pools.map((p) => `
          <div style="margin-bottom:10px;padding:10px;border:1px solid var(--border);border-radius:8px">
            <strong>${esc(p.name)}</strong> <span class="badge" style="color:var(--green)">${esc(p.health)}</span><br/>
            <span style="font-family:var(--mono);font-size:11px;color:var(--muted)">size ${esc(p.size)} · free ${esc(p.free)}</span>
          </div>`).join("") : `<p style="color:var(--muted)">zpool not available — install zfsutils-linux</p>`}
      </div>
    </div>`;
}

function viewHealth() {
  const h = state.health;
  if (!h) return `<p class="empty">Loading…</p>`;
  const entThemes = (state.enterpriseThemes || [])
    .map(
      (th) =>
        `<button type="button" class="btn btn-sm" data-theme="${esc(th.id)}">${esc(th.name)}</button>`
    )
    .join("");
  return `
    <div class="modal-grid">
      <div class="card"><div class="card-head">Devices</div><div class="card-body">${h.device_count} top-level block devices</div></div>
      <div class="card"><div class="card-head">Proxmox</div><div class="card-body">${h.proxmox ? "pvesm detected" : "Not a PVE node"}</div></div>
      <div class="card"><div class="card-head">Missing tools</div><div class="card-body"><ul>${(h.tools_missing || []).map((t) => `<li>${esc(t.name)} — ${esc(t.purpose || "")}</li>`).join("") || "<li>None</li>"}</ul></div></div>
      <div class="card"><div class="card-head">Enterprise palettes</div><div class="card-body"><div class="btn-row">${entThemes || "<span style='color:var(--muted)'>Defaults: Orange / Obsidian / Arctic</span>"}</div></div></div>
    </div>`;
}

function bindMain() {
  document.querySelectorAll("[data-disk]").forEach((el) => {
    el.onclick = async () => {
      state.selectedDisk = el.dataset.disk;
      await loadPartitionMap();
      paintMain();
    };
  });
  document.querySelectorAll("[data-op]").forEach((el) => {
    el.onclick = () => {
      if (el.dataset.op === "ConvertImage") {
        if (state.selectedDisk) state.convertSource = state.selectedDisk;
        openModal("raw");
        return;
      }
      runOp(el.dataset.op);
    };
  });
  document.querySelectorAll("[data-convert]").forEach((el) => {
    el.onclick = async () => {
      const volid = el.dataset.convert;
      try {
        state.convertSource = await resolveProxmoxVolidToPath(volid);
        log(`Resolved ${volid} → ${state.convertSource}`, "info");
        openModal("vhdx");
      } catch (e) {
        log(`Could not resolve ${volid}: ${e.message}`, "warn");
      }
    };
  });
  document.querySelectorAll("[data-theme]").forEach((el) => {
    if (el.closest(".btn-row")) el.onclick = () => setTheme(el.dataset.theme);
  });
}

async function loadPartitionMap() {
  if (!state.selectedDisk) return;
  try {
    state.partitionMap = await api(`/api/partitions?disk=${encodeURIComponent(state.selectedDisk)}`);
  } catch (e) {
    log(`Partition map: ${e.message}`, "warn");
  }
}

async function runOp(kind) {
  if (kind === "ConvertImage") {
    if (state.selectedDisk) state.convertSource = state.selectedDisk;
    openModal("raw");
    return;
  }
  const device = state.selectedDisk;
  if (!device) { log("Select a disk first", "warn"); return; }
  try {
    const body = { kind, dry_run: state.dryRun, device };
    if (kind === "Mount") body.mount_point = "/mnt/lpmr";
    const r = await api("/api/operations", { method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(body) });
    log(r.command, r.message === "dry-run" ? "warn" : "info");
  } catch (e) {
    log(e.message, "warn");
  }
}

function openModal(kind) {
  state.modal = kind;
  state.modalDryRun = true;
  renderModal();
}

function closeModal() {
  state.modal = null;
  renderModal();
}

function renderModal() {
  const root = document.getElementById("modalRoot");
  if (!root) return;
  if (!state.modal) { root.innerHTML = ""; return; }
  const titles = { vhdx: "HYPER-V VHDX CONVERSION & DYNAMIC RESIZING", raw: "RAW CONVERSION & NVMe-oF FABRIC PIPELINE", create: "CREATE VM VOLUME" };
  root.innerHTML = `
    <div class="modal-bg" id="modalBg">
      <div class="modal">
        <div class="modal-header">
          <div><h2>${esc(titles[state.modal])}</h2><p style="margin:4px 0 0;font-size:11px;color:var(--muted)">Rust-backed · qemu-img · dry-run by default</p></div>
          <button type="button" class="btn btn-sm" id="modalClose">✕</button>
        </div>
        <div class="modal-body form">
          ${state.modal !== "create" ? `<label>Source path<input id="mSource" value="${esc(state.convertSource || "")}" placeholder="/path/source.vhdx"/></label>` : ""}
          <label>${state.modal === "create" ? "Image path" : "Destination"}<input id="mDest" placeholder="${state.modal === "create" ? "/var/lib/vz/images/new.qcow2" : "/var/lib/vz/images/out.qcow2"}"/></label>
          ${state.modal === "create" ? `<label>Size<input id="mSize" value="32G" placeholder="32G"/></label>` : ""}
          <label>Output format<select id="mFmt"><option>qcow2</option><option>raw</option><option>vmdk</option><option>vhdx</option></select></label>
          <label><input type="checkbox" id="mDry" ${state.modalDryRun ? "checked" : ""}/> Dry-run (recommended)</label>
          ${state.jobId ? `<div class="progress-bar"><div class="progress-fill" style="width:${state.jobPct}%"></div></div>` : ""}
          <div class="btn-row" style="margin-top:16px">
            <button type="button" class="btn" id="modalCancel">Cancel</button>
            <button type="button" class="btn btn-primary" id="modalRun" ${state.jobPolling ? "disabled" : ""}>${state.modal === "create" ? "Create image" : "Start conversion"}</button>
          </div>
        </div>
      </div>
    </div>`;
  const bg = document.getElementById("modalBg");
  const closeBtn = document.getElementById("modalClose");
  const cancelBtn = document.getElementById("modalCancel");
  const runBtn = document.getElementById("modalRun");
  if (bg) bg.addEventListener("click", (e) => { if (e.target.id === "modalBg") closeModal(); });
  if (closeBtn) closeBtn.onclick = closeModal;
  if (cancelBtn) cancelBtn.onclick = closeModal;
  if (runBtn) {
    runBtn.onclick = () => {
      if (state.modal === "create") submitCreateVolume();
      else submitConvertJob();
    };
  }
  const dry = document.getElementById("mDry");
  if (dry) dry.onchange = () => { state.modalDryRun = dry.checked; };
}

async function submitCreateVolume() {
  const path = document.getElementById("mDest")?.value?.trim();
  const format = document.getElementById("mFmt")?.value || "qcow2";
  const dry = document.getElementById("mDry")?.checked ?? true;
  if (!path) {
    log("Image path required", "warn");
    return;
  }
  const size = document.getElementById("mSize")?.value?.trim() || "32G";
  try {
    const r = await api("/api/operations", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        kind: "CreateImage",
        dry_run: dry,
        path,
        size,
        image_format: format,
      }),
    });
    log(r.command, r.message === "dry-run" ? "warn" : "info");
    closeModal();
  } catch (e) {
    log(e.message, "warn");
  }
}

async function submitConvertJob() {
  const dest = document.getElementById("mDest")?.value?.trim();
  let source = document.getElementById("mSource")?.value?.trim();
  const format = document.getElementById("mFmt")?.value || "qcow2";
  const dry = document.getElementById("mDry")?.checked ?? true;
  if (!source || !dest) {
    log("Source and destination required", "warn");
    return;
  }
  try {
    source = await resolveProxmoxVolidToPath(source);
    const job = await api("/api/jobs/convert", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ source, dest, format, dry_run: dry }),
    });
    state.jobId = job.id;
    log(job.logs?.[0] || job.message, "info");
    if (!dry && job.status === "running") pollJob(job.id);
    else closeModal();
  } catch (e) {
    log(e.message, "warn");
  }
}

async function pollJob(id) {
  state.jobPolling = true;
  renderModal();
  while (state.jobId === id) {
    await new Promise((r) => setTimeout(r, 1000));
    let job;
    try {
      job = await api(`/api/jobs/${id}`);
    } catch (e) {
      log(e.message, "warn");
      continue;
    }
    state.jobPct = job.progress_pct || 0;
    const tail = job.logs?.slice(-1) || [];
    tail.forEach((l) => log(l, "info"));
    renderModal();
    if (job.status === "done" || job.status === "failed") {
      log(job.message, job.status === "done" ? "info" : "warn");
      state.jobId = null;
      state.jobPolling = false;
      closeModal();
      return;
    }
  }
  state.jobPolling = false;
}

async function refreshPart(label, fn, onError) {
  try {
    await fn();
  } catch (e) {
    log(`${label}: ${e.message}`, "warn");
    if (typeof onError === "function") onError();
  }
}

async function refresh() {
  log("Refreshing telemetry…", "info");
  await refreshPart("telemetry", async () => {
    state.telemetry = await api("/api/telemetry");
  });
  await refreshPart("devices", async () => {
    const dev = await api("/api/devices");
    state.devices = dev.devices || [];
    if (!state.selectedDisk && state.devices[0]) state.selectedDisk = state.devices[0].path;
  });
  await refreshPart("tools", async () => {
    state.tools = (await api("/api/tools")).tools || [];
  });
  await refreshPart("themes", async () => {
    state.enterpriseThemes = (await api("/api/themes")).themes || [];
  });
  await refreshPart("health", async () => {
    state.health = await api("/api/health-audit");
  });
  await refreshPart("zfs", async () => {
    const zfs = await api("/api/zfs/pools");
    state.zfs = zfs;
    if (zfs.error) log(`zfs: ${zfs.error}`, "warn");
  }, () => {
    state.zfs = { available: false, pools: [] };
  });
  await refreshPart("lvm", async () => {
    state.lvm = await api("/api/lvm");
  }, () => {
    state.lvm = { available: false, pvs: [], vgs: [], lvs: [] };
  });
  await refreshPart("vm-volumes", async () => {
    const vm = await api("/api/vm-volumes");
    state.vmVolumes = vm.volumes || [];
    if (vm.error) log(`vm-volumes: ${vm.error}`, "warn");
  }, () => {
    state.vmVolumes = [];
  });
  await refreshPart("partitions", () => loadPartitionMap());
  paintTools();
  render();
  bind();
  setTheme(state.uiTheme);
  log("Refresh complete", "info");
}

(async function boot() {
  document.documentElement.setAttribute("data-ui-theme", state.uiTheme);
  render();
  bind();
  try {
    await ensureExecuteCsrf();
    await refresh();
    setInterval(() => api("/api/telemetry").then((t) => { state.telemetry = t; }).catch(() => {}), 15000);
  } catch (e) {
    log(`Boot error: ${e.message}`, "warn");
  }
})();
