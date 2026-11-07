/**
 * LPMR — Proxmox VE Storage Orchestrator (Raven // Baremetal UI)
 */
const state = {
  view: "disks",
  uiTheme: localStorage.getItem("lpmr.ui-theme") || "orange",
  selectedDisk: null,
  telemetry: null,
  devices: [],
  partitionMap: null,
  vmVolumes: [],
  zfs: { available: false, pools: [] },
  health: null,
  tools: [],
  dryRun: true,
  log: [],
  modal: null,
  jobId: null,
  jobPct: 0,
  search: "",
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

async function api(path, opts) {
  const r = await fetch(path, opts);
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

function setTheme(t) {
  state.uiTheme = t;
  localStorage.setItem("lpmr.ui-theme", t);
  document.documentElement.setAttribute("data-ui-theme", t);
  document.querySelectorAll(".theme-switch button").forEach((b) => {
    b.classList.toggle("on", b.dataset.theme === t);
  });
}

function render() {
  const host = state.telemetry?.host;
  const app = document.getElementById("app");
  app.innerHTML = `
    <header class="top-brand">
      <span class="badge-raven">RAVEN // BAREMETAL</span>
      <h1>PROXMOX VE STORAGE ORCHESTRATOR</h1>
      <div class="grow"></div>
      <input type="search" id="globalSearch" placeholder="Search nodes, pools, disks, VM images…" value="${esc(state.search)}"/>
      <button type="button" class="btn btn-ghost" id="btnTui">Open Terminal</button>
      <button type="button" class="btn btn-primary" id="btnCreateVol">+ Create Volume</button>
    </header>
    <nav class="tab-bar" id="tabBar">
      ${["disks", "vmimages", "zfs", "health"].map((v) => `
        <button type="button" data-view="${v}" class="${state.view === v ? "active" : ""}">${
          { disks: "Disks", vmimages: "VM Images", zfs: "ZFS Pools", health: "Health Audit" }[v]
        }</button>`).join("")}
    </nav>
    <div class="metrics">
      <span class="metric-pill ok"><strong>${esc(host?.hostname || "node")}</strong> ONLINE · ${esc(host?.kernel || "linux")}</span>
      <span class="metric-pill"><strong>Rust RSS</strong> ${(host?.rss_mb ?? 0).toFixed(1)} MB</span>
      <span class="metric-pill"><strong>Devices</strong> ${state.devices.length}</span>
      <span class="metric-pill"><strong>Tools</strong> ${state.tools.length}</span>
      <span class="metric-pill"><strong>Dry-run</strong> ${state.dryRun ? "ON" : "OFF"}</span>
    </div>
    <div class="workspace">
      <aside class="col-nav">
        <h4>Storage infrastructure</h4>
        <button type="button" class="nav-link on" data-goto="disks">Block Devices & NVMe</button>
        <button type="button" class="nav-link" data-goto="vmimages">Proxmox VM Volumes</button>
        <button type="button" class="nav-link" data-goto="zfs">ZFS Pools & Datasets</button>
        <h4>Operations</h4>
        <button type="button" class="nav-link" id="openVhdx">VHDX Convert & Resize</button>
        <button type="button" class="nav-link" id="openRaw">RAW / QCOW2 Pipeline</button>
        <h4>UI</h4>
        <div class="theme-switch">
          <button type="button" data-theme="orange" class="${state.uiTheme === "orange" ? "on" : ""}">Orange</button>
          <button type="button" data-theme="obsidian" class="${state.uiTheme === "obsidian" ? "on" : ""}">Obsidian</button>
          <button type="button" data-theme="arctic" class="${state.uiTheme === "arctic" ? "on" : ""}">Arctic</button>
        </div>
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
    b.onclick = () => { state.view = b.dataset.view; render(); bind(); paintMain(); };
  });
  document.querySelectorAll("[data-goto]").forEach((b) => {
    b.onclick = () => { state.view = b.dataset.goto; render(); bind(); paintMain(); };
  });
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
  const list = state.health?.tools_ok || state.tools.map((t) => ({ name: t.name, ok: true }));
  grid.innerHTML = (list || [])
    .slice(0, 12)
    .map((t) => `<div class="tool-chip ok"><span>${esc(t.name)}</span><span>OK</span></div>`)
    .join("");
}

function paintMain() {
  const main = document.getElementById("mainView");
  if (!main) return;
  if (state.view === "disks") main.innerHTML = viewDisks();
  else if (state.view === "vmimages") main.innerHTML = viewVm();
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
                <td><button type="button" class="btn btn-sm" data-convert="${esc(v.volid)}">Convert</button></td>
              </tr>`).join("") : `<tr><td colspan="5" style="color:var(--muted)">No VM volumes (not on PVE or empty)</td></tr>`}
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
  return `
    <div class="modal-grid">
      <div class="card"><div class="card-head">Devices</div><div class="card-body">${h.device_count} top-level block devices</div></div>
      <div class="card"><div class="card-head">Proxmox</div><div class="card-body">${h.proxmox ? "pvesm detected" : "Not a PVE node"}</div></div>
      <div class="card"><div class="card-head">Missing tools</div><div class="card-body"><ul>${(h.tools_missing || []).map((t) => `<li>${esc(t.name)}</li>`).join("") || "<li>None</li>"}</ul></div></div>
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
    el.onclick = () => runOp(el.dataset.op);
  });
  document.querySelectorAll("[data-convert]").forEach((el) => {
    el.onclick = () => { state.convertSource = el.dataset.convert; openModal("vhdx"); };
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
          <label>Source path<input id="mSource" value="${esc(state.convertSource || "")}" placeholder="/path/source.vhdx"/></label>
          <label>Destination<input id="mDest" placeholder="/var/lib/vz/images/out.qcow2"/></label>
          <label>Output format<select id="mFmt"><option>qcow2</option><option>raw</option><option>vmdk</option></select></label>
          <label><input type="checkbox" id="mDry" ${state.dryRun ? "checked" : ""}/> Dry-run (recommended)</label>
          ${state.jobId ? `<div class="progress-bar"><div class="progress-fill" style="width:${state.jobPct}%"></div></div>` : ""}
          <div class="btn-row" style="margin-top:16px">
            <button type="button" class="btn" id="modalCancel">Cancel</button>
            <button type="button" class="btn btn-primary" id="modalRun">Start conversion</button>
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
  if (runBtn) runBtn.onclick = () => submitJob();
  const dry = document.getElementById("mDry");
  if (dry) dry.onchange = () => { state.dryRun = dry.checked; };
}

async function submitJob() {
  const source = document.getElementById("mSource")?.value?.trim();
  const dest = document.getElementById("mDest")?.value?.trim();
  const format = document.getElementById("mFmt")?.value || "qcow2";
  if (!source || !dest) { log("Source and destination required", "warn"); return; }
  const dry = document.getElementById("mDry")?.checked ?? true;
  try {
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
  for (let i = 0; i < 120; i++) {
    await new Promise((r) => setTimeout(r, 1000));
    const job = await api(`/api/jobs/${id}`);
    state.jobPct = job.progress_pct || 0;
    job.logs?.slice(-3).forEach((l) => log(l, "info"));
    renderModal();
    if (job.status === "done" || job.status === "failed") {
      log(job.message, job.status === "done" ? "info" : "warn");
      state.jobId = null;
      closeModal();
      return;
    }
  }
}

async function refresh() {
  log("Refreshing telemetry…", "info");
  state.telemetry = await api("/api/telemetry");
  const dev = await api("/api/devices");
  state.devices = dev.devices || [];
  if (!state.selectedDisk && state.devices[0]) state.selectedDisk = state.devices[0].path;
  state.tools = (await api("/api/tools")).tools || [];
  state.health = await api("/api/health-audit");
  state.zfs = await api("/api/zfs/pools");
  state.vmVolumes = (await api("/api/vm-volumes")).volumes || [];
  await loadPartitionMap();
  paintTools();
  document.querySelectorAll(".metric-pill strong");
  render();
  bind();
  log("Refresh complete", "info");
}

(async function boot() {
  document.documentElement.setAttribute("data-ui-theme", state.uiTheme);
  render();
  bind();
  try {
    await refresh();
    setInterval(() => api("/api/telemetry").then((t) => { state.telemetry = t; }).catch(() => {}), 15000);
  } catch (e) {
    log(`Boot error: ${e.message}`, "warn");
  }
})();
