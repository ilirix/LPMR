const state = {
  lang: document.getElementById("app")?.dataset.lang || "en",
  theme: document.getElementById("app")?.dataset.theme || "midnight-enterprise",
  tab: "devices",
  selected: null,
  strings: {},
  devices: [],
  lvm: null,
  log: [],
};

const themes = [
  { id: "midnight-enterprise", window: "#100D1B", panel: "#191429", accent: "#A78BFA", text: "#F2EDFF", border: "#352B4B", success: "#6EE7B7", warning: "#FCD34D", danger: "#FDA4AF" },
  { id: "slate-pro", window: "#0F172A", panel: "#1E293B", accent: "#38BDF8", text: "#E2E8F0", border: "#334155", success: "#4ADE80", warning: "#FBBF24", danger: "#F87171" },
  { id: "azure-horizon", window: "#061525", panel: "#0B2545", accent: "#0078D4", text: "#F0F6FC", border: "#1B3A57", success: "#107C10", warning: "#FFB900", danger: "#D13438" },
  { id: "forest-admin", window: "#0B1F17", panel: "#143528", accent: "#34D399", text: "#ECFDF5", border: "#1F4535", success: "#22C55E", warning: "#EAB308", danger: "#EF4444" },
  { id: "ruby-executive", window: "#1A0B10", panel: "#2A121C", accent: "#FB7185", text: "#FFF1F2", border: "#4C1D2E", success: "#86EFAC", warning: "#FDE047", danger: "#FB7185" },
];

function t(key) {
  return state.strings[key] || key;
}

function applyTheme(id) {
  const theme = themes.find((x) => x.id === id) || themes[0];
  const root = document.documentElement;
  root.style.setProperty("--lpmr-window", theme.window);
  root.style.setProperty("--lpmr-panel", theme.panel);
  root.style.setProperty("--lpmr-accent", theme.accent);
  root.style.setProperty("--lpmr-text", theme.text);
  root.style.setProperty("--lpmr-border", theme.border);
  root.style.setProperty("--lpmr-success", theme.success);
  root.style.setProperty("--lpmr-warning", theme.warning);
  root.style.setProperty("--lpmr-danger", theme.danger);
  state.theme = theme.id;
}

async function api(path, opts) {
  const res = await fetch(path, opts);
  const data = await res.json();
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
  app.innerHTML = `
    <div class="shell">
      <header class="header">
        <h1>${t("app.title")}</h1>
        <div class="spacer"></div>
        <label>${t("lang.label")}
          <select id="langSel">
            ${["en","es","zh","hi","ar","fr"].map((l)=>`<option value="${l}" ${l===state.lang?"selected":""}>${l}</option>`).join("")}
          </select>
        </label>
        <label>${t("theme.label")}
          <select id="themeSel">
            ${themes.map((th)=>`<option value="${th.id}" ${th.id===state.theme?"selected":""}>${th.name}</option>`).join("")}
          </select>
        </label>
        <button class="primary" id="refreshBtn">${t("action.refresh")}</button>
      </header>
      <div class="warn">${t("warn.privilege")}</div>
      <div class="workspace">
        <section class="panel">
          <div class="tabs">${tabs.map(([id,label])=>`<div class="tab ${state.tab===id?"active":""}" data-tab="${id}">${label}</div>`).join("")}</div>
          <div class="panel-body" id="leftPanel"></div>
        </section>
        <section class="panel">
          <div class="panel-title">${t("panel.details")}</div>
          <div class="panel-body" id="detailPanel"></div>
        </section>
        <section class="panel">
          <div class="panel-title">${t("nav.operations")}</div>
          <div class="panel-body" id="opsPanel">
            <p>Dry-run operations (safe preview). Toggle server-side <code>dry_run</code> before execute.</p>
            <button id="dryMountBtn">${t("action.dry_run")}: mount example</button>
          </div>
        </section>
      </div>
      <div class="log" id="logPanel"><strong>${t("panel.log")}</strong><br/>${state.log.map((l)=>`<div>${l}</div>`).join("")}</div>
    </div>`;

  document.querySelectorAll(".tab").forEach((el) => {
    el.onclick = () => { state.tab = el.dataset.tab; renderMainPanels(); document.querySelectorAll(".tab").forEach((t)=>t.classList.toggle("active", t.dataset.tab===state.tab)); };
  });
  document.getElementById("langSel").onchange = async (e) => {
    state.lang = e.target.value;
    await loadI18n();
    render();
  };
  document.getElementById("themeSel").onchange = (e) => { applyTheme(e.target.value); render(); };
  document.getElementById("refreshBtn").onclick = () => refreshAll();
  document.getElementById("dryMountBtn").onclick = () => dryMountExample();
  renderMainPanels();
  renderDetail();
}

function renderMainPanels() {
  const left = document.getElementById("leftPanel");
  if (!left) return;
  if (state.tab === "devices") {
    const flat = flattenDevices(state.devices);
    left.innerHTML = flat.map((d)=>`
      <div class="tree-item ${state.selected===d.path?"selected":""}" data-path="${d.path}" style="padding-left:${8+d.depth*12}px">
        <div class="mono">${d.name}</div>
        <div style="color:var(--lpmr-muted);font-size:11px">${d.size_bytes} B ${d.fstype||""}</div>
      </div>`).join("") || "<em>No devices</em>";
    left.querySelectorAll(".tree-item").forEach((el)=>{
      el.onclick = ()=>{ state.selected = el.dataset.path; renderDetail(); renderMainPanels(); };
    });
  } else if (state.tab === "lvm") {
    const l = state.lvm;
    left.innerHTML = l ? `
      <h4>VGs</h4><pre class="mono">${JSON.stringify(l.vgs,null,2)}</pre>
      <h4>LVs</h4><pre class="mono">${JSON.stringify(l.lvs,null,2)}</pre>` : "Loading…";
  } else if (state.tab === "tools") {
    left.innerHTML = `<pre class="mono">${JSON.stringify(state.tools||[], null, 2)}</pre>`;
  } else {
    left.innerHTML = `<p>Feature tab: ${state.tab}. Use API /api/qemu/* and /api/proxmox/*.</p>`;
  }
}

function renderDetail() {
  const panel = document.getElementById("detailPanel");
  if (!panel) return;
  const flat = flattenDevices(state.devices);
  const dev = flat.find((d)=>d.path===state.selected);
  if (!dev) { panel.innerHTML = "<em>Select a device</em>"; return; }
  panel.innerHTML = `
    <div class="kv">
      <div class="k">Path</div><div>${dev.path}</div>
      <div class="k">Size</div><div>${dev.size_bytes}</div>
      <div class="k">Model</div><div>${dev.model||"-"}</div>
      <div class="k">Fstype</div><div>${dev.fstype||"-"}</div>
      <div class="k">Transport</div><div>${dev.transport||"-"}</div>
      <div class="k">Mountpoints</div><div>${(dev.mountpoints||[]).filter(Boolean).join(", ")||"-"}</div>
    </div>`;
}

async function loadI18n() {
  const bundle = await api(`/api/i18n/${state.lang}`);
  state.strings = bundle.strings;
}

async function refreshAll() {
  pushLog("Refreshing…");
  state.devices = (await api("/api/devices")).devices;
  try { state.lvm = await api("/api/lvm"); } catch (e) { pushLog("LVM: " + e.message); }
  try { state.tools = (await api("/api/tools")).tools; } catch (e) { pushLog("Tools: " + e.message); }
  renderMainPanels();
  renderDetail();
  pushLog("Refresh complete.");
}

function pushLog(msg) {
  const ts = new Date().toISOString();
  state.log.unshift(`[${ts}] ${msg}`);
  state.log = state.log.slice(0, 200);
  const log = document.getElementById("logPanel");
  if (log) log.innerHTML = `<strong>${t("panel.log")}</strong><br/>${state.log.map((l)=>`<div>${l}</div>`).join("")}`;
}

async function dryMountExample() {
  pushLog("Dry-run mount example requested");
  await api("/api/operations", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ kind: "Mount", dry_run: true, device: "/dev/sda1", mount_point: "/mnt/lpmr" }),
  });
  pushLog("Dry-run mount queued (see API response in devtools)");
}

(async function boot() {
  applyTheme(state.theme);
  await loadI18n();
  render();
  await refreshAll();
})();
