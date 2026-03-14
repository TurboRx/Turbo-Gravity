// Inline HTML page generators for the Turbo Gravity dashboard.
//
// Modern redesign: fixed left sidebar, topbar, card-based layout.
// All CSS lives in STYLES (served at /styles.css).
// Theme JS lives in THEME_SCRIPT (raw str constant – no format-escape worries).

// ---------------------------------------------------------------------------
// Shared CSS  (served at /styles.css)
// ---------------------------------------------------------------------------

pub const STYLES: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&display=swap');

/* ── Variables: dark (default) ───────────────────────────────────────────── */
:root, html.dark {
  --bg:      #0b0e1a;
  --bg2:     #111527;
  --bg3:     #161b30;
  --card:    #13172a;
  --text:    #e8eaf0;
  --text2:   #9aa3be;
  --text3:   #5c6585;
  --purple:  #7c3aed;
  --purple2: #9d5cf0;
  --cyan:    #06b6d4;
  --green:   #10b981;
  --red:     #ef4444;
  --border:  rgba(255,255,255,0.07);
  --shadow:  0 4px 24px rgba(0,0,0,0.4);
}

/* ── Variables: light ────────────────────────────────────────────────────── */
html.light {
  --bg:     #f0f2f9;
  --bg2:    #ffffff;
  --bg3:    #f4f6fb;
  --card:   #ffffff;
  --text:   #0f1427;
  --text2:  #4b5379;
  --text3:  #9aa3be;
  --border: rgba(0,0,0,0.08);
  --shadow: 0 4px 24px rgba(0,0,0,0.1);
}

/* ── Variables: system/device default (no explicit class) ────────────────── */
@media (prefers-color-scheme: light) {
  :root:not(.dark):not(.light) {
    --bg:     #f0f2f9;
    --bg2:    #ffffff;
    --bg3:    #f4f6fb;
    --card:   #ffffff;
    --text:   #0f1427;
    --text2:  #4b5379;
    --text3:  #9aa3be;
    --border: rgba(0,0,0,0.08);
    --shadow: 0 4px 24px rgba(0,0,0,0.1);
  }
}

/* ── Reset ───────────────────────────────────────────────────────────────── */
*,*::before,*::after { box-sizing: border-box; margin: 0; padding: 0; }
body {
  font-family: 'Inter', system-ui, -apple-system, sans-serif;
  background: var(--bg);
  color: var(--text);
  min-height: 100vh;
  font-size: 14px;
  line-height: 1.5;
  /* Prevent Android browsers from auto-scaling text which causes a "zoomed in" look */
  -webkit-text-size-adjust: 100%;
  text-size-adjust: 100%;
}
a { color: inherit; text-decoration: none; touch-action: manipulation; }
/* Prevent double-tap zoom on buttons and links on Android/mobile browsers */
button { cursor: pointer; font-family: inherit; touch-action: manipulation; }
input, select, textarea { font-family: inherit; }

/* ── Layout ──────────────────────────────────────────────────────────────── */
.layout { display: flex; min-height: 100vh; }

/* ── Sidebar ─────────────────────────────────────────────────────────────── */
.sidebar {
  width: 220px;
  min-height: 100vh;
  background: var(--bg2);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  position: fixed;
  top: 0; left: 0;
  z-index: 100;
}
.sidebar-brand {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 20px 20px 16px;
  border-bottom: 1px solid var(--border);
}
.brand-dot {
  width: 34px; height: 34px;
  border-radius: 8px;
  background: linear-gradient(135deg, var(--purple), var(--cyan));
  flex-shrink: 0;
  display: flex; align-items: center; justify-content: center;
  font-size: 17px;
}
.sidebar-brand-name { font-weight: 700; font-size: 15px; color: var(--text); }
.sidebar-nav {
  padding: 12px 10px;
  flex: 1;
  display: flex; flex-direction: column; gap: 2px;
}
.nav-item {
  display: flex; align-items: center; gap: 10px;
  padding: 9px 12px;
  border-radius: 8px;
  color: var(--text2);
  font-size: 13.5px; font-weight: 500;
  transition: background 0.15s, color 0.15s;
}
.nav-item:hover { background: rgba(124,58,237,0.1); color: var(--text); }
.nav-item.active { background: rgba(124,58,237,0.15); color: var(--purple2); }
.nav-icon { font-size: 16px; width: 20px; text-align: center; }

/* ── Main wrapper ────────────────────────────────────────────────────────── */
.main-wrapper {
  margin-left: 220px; flex: 1;
  display: flex; flex-direction: column; min-height: 100vh;
}

/* ── Top bar ─────────────────────────────────────────────────────────────── */
.topbar {
  height: 60px;
  background: var(--bg2);
  border-bottom: 1px solid var(--border);
  display: flex; align-items: center; justify-content: space-between;
  padding: 0 24px;
  position: sticky; top: 0; z-index: 50;
}
.topbar-left { display: flex; align-items: center; gap: 8px; font-size: 14px; }
.topbar-site { font-weight: 700; color: var(--text); }
.topbar-sep  { color: var(--text3); }
.topbar-page { color: var(--text2); }
.topbar-right { display: flex; align-items: center; gap: 12px; }
.search-box {
  background: var(--bg3); border: 1px solid var(--border);
  border-radius: 8px; padding: 7px 12px;
  color: var(--text); font-size: 13px; width: 200px; outline: none;
}
.search-box::placeholder { color: var(--text3); }
.search-box:focus { border-color: var(--purple); }
.notif-btn, .theme-btn {
  background: var(--bg3); border: 1px solid var(--border);
  border-radius: 8px; width: 36px; height: 36px;
  display: flex; align-items: center; justify-content: center;
  color: var(--text2); font-size: 15px; position: relative;
  transition: background 0.15s, color 0.15s;
}
.notif-btn:hover, .theme-btn:hover { background: rgba(124,58,237,0.15); color: var(--text); }
.notif-dot {
  width: 7px; height: 7px; border-radius: 50%; background: var(--red);
  position: absolute; top: 6px; right: 6px;
}
.avatar {
  width: 36px; height: 36px; border-radius: 50%;
  background: linear-gradient(135deg, var(--purple), var(--cyan));
  display: flex; align-items: center; justify-content: center;
  font-weight: 700; font-size: 12px; color: #fff; cursor: pointer;
}

/* ── Content area ────────────────────────────────────────────────────────── */
.content { padding: 24px; flex: 1; }

/* ── Dashboard two-column grid ───────────────────────────────────────────── */
.dashboard-grid {
  display: grid;
  grid-template-columns: 1fr 360px;
  gap: 20px;
  align-items: start;
}
.col-left, .col-right { display: flex; flex-direction: column; gap: 20px; }

/* ── Cards ───────────────────────────────────────────────────────────────── */
.card {
  background: var(--card);
  border: 1px solid var(--border);
  border-radius: 14px;
  padding: 20px;
}
.card-header {
  display: flex; align-items: center; justify-content: space-between;
  margin-bottom: 16px;
}
.card-title {
  font-size: 15px; font-weight: 600; color: var(--text);
  display: flex; align-items: center; gap: 8px;
}
.card-badge {
  font-size: 11px; padding: 3px 8px; border-radius: 999px;
  background: rgba(124,58,237,0.15); color: var(--purple2); font-weight: 500;
}

/* ── SVG chart ───────────────────────────────────────────────────────────── */
.chart-wrap {
  width: 100%; height: 140px;
  border-radius: 8px; overflow: hidden; background: var(--bg3);
}
.chart-wrap svg { width: 100%; height: 100%; }
.chart-legend { display: flex; gap: 16px; margin-top: 10px; }
.legend-item {
  display: flex; align-items: center; gap: 6px;
  font-size: 12px; color: var(--text2);
}
.legend-dot { width: 8px; height: 8px; border-radius: 50%; }

/* ── Bot subgrid ─────────────────────────────────────────────────────────── */
.subgrid { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
.subcard {
  background: var(--card); border: 1px solid var(--border);
  border-radius: 12px; padding: 16px;
}
.subcard-title {
  font-size: 11px; font-weight: 600; color: var(--text3);
  text-transform: uppercase; letter-spacing: 0.05em; margin-bottom: 10px;
}
.server-list { display: flex; flex-direction: column; gap: 8px; }
.server-row {
  display: flex; align-items: center; gap: 8px;
  font-size: 13px; color: var(--text2);
}
.server-dot {
  width: 7px; height: 7px; border-radius: 50%;
  background: var(--green); flex-shrink: 0;
}
.uptime-val { font-size: 28px; font-weight: 700; color: var(--text); }
.uptime-label { font-size: 12px; color: var(--text3); margin-top: 2px; }
.uptime-bar-wrap {
  height: 6px; background: var(--bg3); border-radius: 3px;
  margin-top: 10px; overflow: hidden;
}
.uptime-bar {
  height: 100%; border-radius: 3px;
  background: linear-gradient(90deg, var(--purple), var(--cyan));
}

/* ── Status badge ────────────────────────────────────────────────────────── */
.status-badge {
  display: inline-flex; align-items: center; gap: 5px;
  padding: 3px 9px; border-radius: 999px;
  font-size: 12px; font-weight: 600;
}
.status-badge.online  { background: rgba(16,185,129,0.15); color: var(--green); }
.status-badge.offline { background: rgba(239,68,68,0.15);  color: var(--red); }
.status-dot { width: 6px; height: 6px; border-radius: 50%; background: currentColor; }

/* ── Buttons ─────────────────────────────────────────────────────────────── */
.btn {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 8px 16px; border-radius: 8px; border: none;
  font-size: 13px; font-weight: 500;
  transition: opacity 0.15s, transform 0.1s; cursor: pointer;
}
.btn:hover  { opacity: 0.85; transform: translateY(-1px); }
.btn:active { transform: translateY(0); }
.btn-primary { background: linear-gradient(135deg, var(--purple), var(--purple2)); color: #fff; }
.btn-cyan    { background: linear-gradient(135deg, var(--cyan), #0891b2); color: #fff; }
.btn-ghost   { background: var(--bg3); color: var(--text2); border: 1px solid var(--border); }
.btn-danger  { background: rgba(239,68,68,0.15); color: var(--red); border: 1px solid rgba(239,68,68,0.3); }
.btn-actions { display: flex; gap: 8px; flex-wrap: wrap; margin-bottom: 14px; }

/* ── Form elements ───────────────────────────────────────────────────────── */
.form-row   { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.form-group { display: flex; flex-direction: column; gap: 5px; }
.form-label { font-size: 12px; font-weight: 500; color: var(--text2); }
.form-input, .form-select {
  background: var(--bg3); border: 1px solid var(--border);
  border-radius: 8px; padding: 8px 12px;
  color: var(--text); font-size: 13px; outline: none;
}
.form-input:focus, .form-select:focus { border-color: var(--purple); }

/* ── Invite link ─────────────────────────────────────────────────────────── */
.invite-link-box {
  display: flex; align-items: center; gap: 10px;
  background: var(--bg3); border: 1px solid var(--border);
  border-radius: 8px; padding: 10px 14px; margin-top: 8px;
}
.invite-link-text {
  flex: 1; font-size: 12px; color: var(--text2);
  overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}

/* ── Module cards grid ───────────────────────────────────────────────────── */
.module-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 12px; }
.module-card {
  background: var(--bg3); border: 1px solid var(--border);
  border-radius: 12px; padding: 14px;
  display: flex; flex-direction: column; gap: 6px;
}
.module-header { display: flex; align-items: center; justify-content: space-between; }
.module-icon-name { display: flex; align-items: center; gap: 8px; }
.module-icon {
  font-size: 18px; width: 32px; height: 32px;
  display: flex; align-items: center; justify-content: center;
  background: rgba(124,58,237,0.12); border-radius: 8px;
}
.module-name { font-size: 13px; font-weight: 600; color: var(--text); }
.module-desc { font-size: 11.5px; color: var(--text3); line-height: 1.4; }

/* ── Toggle switch ───────────────────────────────────────────────────────── */
label.toggle {
  display: inline-flex; align-items: center;
  cursor: pointer; position: relative;
}
label.toggle input[type=checkbox] {
  position: absolute; opacity: 0; width: 0; height: 0;
}
.toggle-track {
  width: 36px; height: 20px; border-radius: 10px;
  background: var(--bg2); border: 1px solid var(--border);
  position: relative; transition: background 0.25s; display: block;
}
label.toggle input:checked + .toggle-track {
  background: linear-gradient(135deg, var(--purple), var(--cyan));
  border-color: transparent;
}
.toggle-thumb {
  width: 14px; height: 14px; border-radius: 50%;
  background: var(--text3);
  position: absolute; top: 2px; left: 2px;
  transition: transform 0.25s, background 0.25s;
}
label.toggle input:checked + .toggle-track .toggle-thumb {
  transform: translateX(16px); background: #fff;
}

/* ── Setup page (centered card, no sidebar) ──────────────────────────────── */
.setup-page {
  min-height: 100vh;
  display: flex; align-items: center; justify-content: center;
  background: var(--bg); padding: 40px 20px;
}
.setup-card {
  background: var(--card); border: 1px solid var(--border);
  border-radius: 18px; padding: 40px;
  width: 100%; max-width: 640px;
}
.setup-header { text-align: center; margin-bottom: 36px; }
.setup-logo {
  width: 56px; height: 56px; border-radius: 14px;
  background: linear-gradient(135deg, var(--purple), var(--cyan));
  display: flex; align-items: center; justify-content: center;
  font-size: 28px; margin: 0 auto 14px;
}
.setup-header h1 { font-size: 22px; font-weight: 700; margin-bottom: 6px; }
.setup-header p  { color: var(--text2); font-size: 14px; }
.setup-section   { margin-bottom: 28px; }
.setup-section-header {
  display: flex; align-items: center; gap: 10px;
  margin-bottom: 18px; padding-bottom: 10px;
  border-bottom: 1px solid var(--border);
}
.setup-section-icon  { font-size: 18px; }
.setup-section-title { font-size: 15px; font-weight: 600; }
.setup-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
.setup-field { display: flex; flex-direction: column; gap: 5px; }
.setup-field.full { grid-column: 1 / -1; }
.setup-field label { font-size: 12px; font-weight: 500; color: var(--text2); }
.setup-field input, .setup-field select {
  background: var(--bg3); border: 1px solid var(--border);
  border-radius: 8px; padding: 9px 12px;
  color: var(--text); font-size: 13px; outline: none; width: 100%;
}
.setup-field input:focus, .setup-field select:focus { border-color: var(--purple); }
.setup-submit {
  width: 100%; padding: 12px; border-radius: 10px;
  background: linear-gradient(135deg, var(--purple), var(--purple2));
  color: #fff; font-size: 15px; font-weight: 600;
  border: none; cursor: pointer; margin-top: 8px;
  transition: opacity 0.15s;
}
.setup-submit:hover { opacity: 0.9; }

/* ── Selector page ───────────────────────────────────────────────────────── */
.selector-page { min-height: 100vh; display: flex; flex-direction: column; background: var(--bg); }
.selector-content { flex: 1; padding: 32px 24px; margin-left: 220px; }
.selector-header { margin-bottom: 28px; }
.selector-header h1 { font-size: 22px; font-weight: 700; margin-bottom: 4px; }
.selector-header p  { color: var(--text2); }
.guild-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px,1fr)); gap: 16px; }
.guild-card {
  background: var(--card); border: 1px solid var(--border);
  border-radius: 12px; padding: 20px;
  display: flex; flex-direction: column; align-items: center; gap: 12px;
  text-align: center; transition: border-color 0.15s, transform 0.15s;
}
.guild-card:hover { border-color: var(--purple); transform: translateY(-2px); }
.guild-icon {
  width: 56px; height: 56px; border-radius: 14px;
  background: linear-gradient(135deg, var(--purple), var(--cyan));
  display: flex; align-items: center; justify-content: center;
  font-size: 22px; font-weight: 700; color: #fff; overflow: hidden;
}
.guild-icon img { width: 100%; height: 100%; object-fit: cover; }
.guild-name    { font-size: 14px; font-weight: 600; color: var(--text); }
.guild-members { font-size: 12px; color: var(--text3); }
.guild-select-btn {
  width: 100%; padding: 7px; border-radius: 7px;
  background: rgba(124,58,237,0.15); border: 1px solid rgba(124,58,237,0.3);
  color: var(--purple2); font-size: 12px; font-weight: 600;
  cursor: pointer; transition: background 0.15s;
}
.guild-select-btn:hover { background: rgba(124,58,237,0.25); }

/* ── Error page ──────────────────────────────────────────────────────────── */
.error-page {
  min-height: 100vh;
  display: flex; align-items: center; justify-content: center;
  background: var(--bg);
}
.error-card {
  text-align: center; padding: 48px 40px;
  background: var(--card); border: 1px solid var(--border);
  border-radius: 18px; max-width: 480px;
}
.error-code {
  font-size: 72px; font-weight: 800; line-height: 1; margin-bottom: 12px;
  background: linear-gradient(135deg, var(--purple), var(--cyan));
  -webkit-background-clip: text; -webkit-text-fill-color: transparent;
  background-clip: text;
}
.error-title   { font-size: 22px; font-weight: 700; margin-bottom: 8px; }
.error-message { color: var(--text2); margin-bottom: 24px; }

/* ── Material Symbols icon sizing ────────────────────────────────────────── */
.material-symbols-rounded {
  font-family: 'Material Symbols Rounded';
  font-weight: normal;
  font-style: normal;
  font-size: 20px;
  line-height: 1;
  letter-spacing: normal;
  text-transform: none;
  display: inline-block;
  white-space: nowrap;
  direction: ltr;
  -webkit-font-feature-settings: 'liga';
  font-feature-settings: 'liga' 1;
  -webkit-font-smoothing: antialiased;
  font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 20;
  vertical-align: middle;
  user-select: none;
}
.nav-icon .material-symbols-rounded  { font-size: 18px; }
.mi-brand  { font-size: 22px; font-variation-settings: 'FILL' 1, 'wght' 600, 'GRAD' 0, 'opsz' 20; }
.mi-card   { font-size: 18px; font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 20; }
.mi-module { font-size: 22px; font-variation-settings: 'FILL' 1, 'wght' 500, 'GRAD' 0, 'opsz' 24; }
.mi-btn    { font-size: 16px; font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 20; }
.mi-setup  { font-size: 24px; font-variation-settings: 'FILL' 1, 'wght' 500, 'GRAD' 0, 'opsz' 24; }
.mi-topbar { font-size: 18px; font-variation-settings: 'FILL' 0, 'wght' 400, 'GRAD' 0, 'opsz' 20; }

/* ── Scrollbar ───────────────────────────────────────────────────────────── */
::-webkit-scrollbar { width: 6px; height: 6px; }
::-webkit-scrollbar-track  { background: var(--bg); }
::-webkit-scrollbar-thumb  { background: var(--bg3); border-radius: 3px; }

/* ── Ensure minimum 44×44 px tap targets on all interactive elements ─────── */
.btn            { min-height: 44px; padding: 0 16px; }
.notif-btn, .theme-btn { width: 44px; height: 44px; }
.guild-select-btn { min-height: 44px; }
.setup-submit   { min-height: 44px; }

/* ── InfoTip (inline help tooltips next to form labels) ──────────────────── */
.infotip-wrap {
  position: relative;
  display: inline-flex; align-items: center;
}
.infotip-trigger {
  display: inline-flex; align-items: center; justify-content: center;
  width: 16px; height: 16px; border-radius: 50%;
  background: rgba(124,58,237,0.18); color: var(--purple2);
  font-size: 10px; font-weight: 700; line-height: 1;
  cursor: pointer; border: none; margin-left: 5px; flex-shrink: 0;
  transition: background 0.15s;
}
.infotip-trigger:hover  { background: rgba(124,58,237,0.35); }
.infotip-content {
  display: none;
  position: absolute; left: 0; top: calc(100% + 6px);
  z-index: 300;
  background: var(--bg2); border: 1px solid var(--border);
  border-radius: 8px; padding: 9px 13px;
  font-size: 12px; color: var(--text2); line-height: 1.5;
  width: 240px; box-shadow: var(--shadow);
}
.infotip-content a { color: var(--purple2); text-decoration: underline; }
.infotip-wrap:hover .infotip-content,
.infotip-wrap.open  .infotip-content { display: block; }

/* ── Toast notifications ─────────────────────────────────────────────────── */
.toast-container {
  position: fixed; bottom: 20px; right: 20px;
  display: flex; flex-direction: column; gap: 8px;
  z-index: 9999; pointer-events: none;
}
.toast {
  background: var(--bg2); border: 1px solid var(--border);
  border-radius: 10px; padding: 12px 16px;
  font-size: 13px; color: var(--text);
  box-shadow: var(--shadow);
  display: flex; align-items: center; gap: 8px;
  opacity: 0; transform: translateY(10px);
  transition: opacity 0.25s, transform 0.25s;
  pointer-events: auto; min-width: 200px; max-width: 320px;
}
.toast.show           { opacity: 1; transform: translateY(0); }
.toast.toast-success  { border-left: 3px solid var(--green); }
.toast.toast-error    { border-left: 3px solid var(--red); }
.toast.toast-info     { border-left: 3px solid var(--purple); }

/* ── Statistics popover (touch-friendly subcards) ────────────────────────── */
.subcard { position: relative; cursor: default; }
.stats-popover {
  display: none;
  position: absolute; left: 50%; bottom: calc(100% + 8px);
  transform: translateX(-50%);
  background: var(--bg2); border: 1px solid var(--border);
  border-radius: 10px; padding: 12px 16px;
  font-size: 12px; color: var(--text2); line-height: 1.8;
  width: 200px; box-shadow: var(--shadow);
  z-index: 200; pointer-events: none;
}
.stats-popover strong { color: var(--text); }
.subcard:hover  .stats-popover,
.subcard.pop-open .stats-popover { display: block; }

/* ── Hamburger button (hidden on desktop) ────────────────────────────────── */
.hamburger {
  display: none;
  flex-direction: column; align-items: center; justify-content: center; gap: 5px;
  width: 44px; height: 44px; border-radius: 8px;
  background: var(--bg3); border: 1px solid var(--border);
  cursor: pointer; flex-shrink: 0;
  transition: background 0.15s;
}
.hamburger:hover { background: rgba(124,58,237,0.15); }
.hamburger span {
  display: block; width: 18px; height: 2px;
  background: var(--text2); border-radius: 2px;
  transition: transform 0.25s, opacity 0.25s;
}
.hamburger.open span:nth-child(1) { transform: translateY(7px) rotate(45deg); }
.hamburger.open span:nth-child(2) { opacity: 0; }
.hamburger.open span:nth-child(3) { transform: translateY(-7px) rotate(-45deg); }

/* ── Sidebar overlay (mobile backdrop) ───────────────────────────────────── */
.sidebar-overlay {
  display: none;
  position: fixed; inset: 0;
  background: rgba(0,0,0,0.55);
  z-index: 99;
  backdrop-filter: blur(2px);
  -webkit-backdrop-filter: blur(2px);
}
.sidebar-overlay.open { display: block; }

/* ── Sidebar: add slide transition ───────────────────────────────────────── */
.sidebar { transition: transform 0.25s ease; }

/* ── Responsive: tablet (≤ 1100px) ──────────────────────────────────────── */
@media (max-width: 1100px) {
  .dashboard-grid { grid-template-columns: 1fr; }
}

/* ── Responsive: mobile (≤ 768px) ───────────────────────────────────────── */
@media (max-width: 768px) {
  .hamburger        { display: flex; }
  .sidebar          { transform: translateX(-220px); }
  .sidebar.open     { transform: translateX(0); }
  .main-wrapper     { margin-left: 0; }
  .selector-content { margin-left: 0; }
  .setup-grid       { grid-template-columns: 1fr; }
  .content          { padding: 16px; }
  .topbar           { padding: 0 14px; gap: 8px; }
  .topbar-site      { display: none; }
  .topbar-sep       { display: none; }
  .search-box       { display: none; }
  .subgrid          { grid-template-columns: 1fr; }
  .module-grid      { grid-template-columns: 1fr 1fr; }
  .btn-actions      { flex-wrap: wrap; }
  /* Prevent iOS Safari from auto-zooming when focusing inputs (font-size < 16px triggers zoom) */
  input, select, textarea { font-size: 16px; }
}

/* ── Responsive: small mobile (≤ 480px) ─────────────────────────────────── */
@media (max-width: 480px) {
  .content          { padding: 12px; }
  .card             { padding: 14px; }
  .module-grid      { grid-template-columns: 1fr; }
  .btn-actions      { flex-direction: column; }
  .btn-actions .btn { width: 100%; justify-content: center; }
  .uptime-val       { font-size: 22px; }
  .invite-link-box  { flex-direction: column; align-items: flex-start; }
  .invite-link-box .btn { width: 100%; justify-content: center; }
  .setup-card       { padding: 24px 16px; }
  .error-card       { padding: 32px 20px; }
  .error-code       { font-size: 56px; }
}
"#;

// ---------------------------------------------------------------------------
// Theme-toggle JS – raw string keeps JS braces away from format! escaping
// ---------------------------------------------------------------------------

const THEME_SCRIPT: &str = r#"<script>
(function() {
  const t = localStorage.getItem('theme');
  const h = document.documentElement;
  if (t === 'light')       { h.className = 'light'; }
  else if (t === 'system') { h.className = ''; }
  else                     { h.className = 'dark'; }
})();
function toggleTheme() {
  const h = document.documentElement;
  const t = localStorage.getItem('theme') || 'dark';
  if (t === 'dark') {
    h.className = 'light';
    localStorage.setItem('theme', 'light');
  } else if (t === 'light') {
    h.className = '';
    localStorage.setItem('theme', 'system');
  } else {
    h.className = 'dark';
    localStorage.setItem('theme', 'dark');
  }
}
function toggleSidebar() {
  const sidebar   = document.getElementById('sidebar');
  const overlay   = document.getElementById('sidebar-overlay');
  const hamburger = document.getElementById('hamburger-btn');
  if (!sidebar) return;
  const isOpen = sidebar.classList.toggle('open');
  if (overlay)   { overlay.classList.toggle('open', isOpen); }
  if (hamburger) { hamburger.classList.toggle('open', isOpen); }
}
</script>"#;

// ---------------------------------------------------------------------------
// Dashboard JS – fetch-based control actions, toasts, stats popovers, InfoTips
// ---------------------------------------------------------------------------

const DASHBOARD_SCRIPT: &str = r#"<script>
/* ── Toast helper ──────────────────────────────────────────────────────── */
function _toastContainer() {
  let c = document.getElementById('toast-container');
  if (!c) { c = document.createElement('div'); c.id = 'toast-container';
            c.className = 'toast-container'; document.body.appendChild(c); }
  return c;
}
function showToast(msg, type) {
  type = type || 'info';
  const t = document.createElement('div');
  t.className = 'toast toast-' + type;
  t.textContent = msg;
  _toastContainer().appendChild(t);
  requestAnimationFrame(function() { t.classList.add('show'); });
  setTimeout(function() {
    t.classList.remove('show');
    setTimeout(function() { t.remove(); }, 300);
  }, 3500);
}

/* ── Control button handler (fetch + toast, no full-page reload) ─────── */
function controlAction(form, event) {
  event.preventDefault();
  const btn = form.querySelector('button[type=submit]');
  if (btn) btn.disabled = true;
  fetch(form.action, { method: 'POST' })
    .then(function(r) { return r.json().catch(function() { return { message: r.ok ? 'Done' : 'Request failed', success: r.ok }; }); })
    .then(function(d) { showToast(d.message || 'Done', d.success !== false ? 'success' : 'error'); })
    .catch(function(e) { showToast('Network error: ' + e.message, 'error'); })
    .finally(function() { if (btn) btn.disabled = false; });
}

/* ── Stats subcard touch-friendly popovers ───────────────────────────── */
document.addEventListener('DOMContentLoaded', function() {
  document.querySelectorAll('.subcard[data-popover]').forEach(function(card) {
    function open()  { card.classList.add('pop-open'); }
    function close() { card.classList.remove('pop-open'); }
    card.addEventListener('mouseenter', open);
    card.addEventListener('mouseleave', close);
    card.addEventListener('touchstart', function(e) {
      e.preventDefault();
      card.classList.toggle('pop-open');
    }, { passive: false });
    // Close when tapping elsewhere
    document.addEventListener('touchstart', function(e) {
      if (!card.contains(e.target)) close();
    }, { passive: true });
  });

  /* ── InfoTip toggle for touch devices ─────────────────────────────── */
  document.querySelectorAll('.infotip-trigger').forEach(function(btn) {
    btn.addEventListener('click', function(e) {
      e.stopPropagation();
      const wrap = btn.closest('.infotip-wrap');
      if (wrap) wrap.classList.toggle('open');
    });
  });
  document.addEventListener('click', function() {
    document.querySelectorAll('.infotip-wrap.open').forEach(function(w) { w.classList.remove('open'); });
  });
});

/* ── Config restore upload handler ─────────────────────────────────────── */
function restoreConfig(input) {
  if (!input.files || !input.files[0]) return;
  var file = input.files[0];
  var statusEl = document.getElementById('restore-status');
  if (statusEl) { statusEl.style.display = 'block'; statusEl.textContent = 'Uploading…'; }
  var formData = new FormData();
  formData.append('file', file);
  fetch('/api/config/restore', { method: 'POST', body: formData })
    .then(function(r) { return r.json().catch(function() { return { success: r.ok, message: r.ok ? 'Restored.' : 'Request failed.' }; }); })
    .then(function(d) {
      var success = (typeof d.success === 'boolean') ? d.success : false;
      var message = d.message || d.error || (success ? 'Restored.' : 'Failed.');
      if (statusEl) { statusEl.textContent = message; }
      showToast(message || (success ? 'Configuration restored!' : 'Restore failed.'), success ? 'success' : 'error');
    })
    .catch(function(e) {
      if (statusEl) { statusEl.textContent = 'Network error: ' + e.message; }
      showToast('Network error: ' + e.message, 'error');
    })
    .finally(function() { input.value = ''; });
}
</script>"#;

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

fn html_head(title: &str, extra_script: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{title} | Turbo Gravity</title>
  <link rel="preconnect" href="https://fonts.googleapis.com" />
  <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin />
  <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&amp;display=swap" rel="stylesheet" />
  <link href="https://fonts.googleapis.com/css2?family=Material+Symbols+Rounded:opsz,wght,FILL,GRAD@20,400,0..1,0&amp;display=block" rel="stylesheet" />
  <link rel="stylesheet" href="/styles.css" />
  {THEME_SCRIPT}
  {extra_script}
</head>
<body>
"#
    )
}

fn build_sidebar(active: &str) -> String {
    let items: &[(&str, &str, &str)] = &[
        ("home",          "Home",      "/dashboard"),
        ("bar_chart",     "Analytics", "/dashboard"),
        ("dns",           "Servers",   "/selector"),
        ("extension",     "Modules",   "/dashboard"),
        ("settings",      "Settings",  "/setup"),
    ];
    let nav: String = items
        .iter()
        .map(|(icon, name, href)| {
            let cls = if *name == active { " active" } else { "" };
            format!(
                r#"    <a href="{href}" class="nav-item{cls}"><span class="nav-icon"><span class="material-symbols-rounded">{icon}</span></span>{name}</a>
"#
            )
        })
        .collect();
    format!(
        r#"<aside class="sidebar" id="sidebar">
  <div class="sidebar-brand">
    <div class="brand-dot"><span class="material-symbols-rounded mi-brand">bolt</span></div>
    <span class="sidebar-brand-name">Turbo Gravity</span>
  </div>
  <nav class="sidebar-nav">
{nav}  </nav>
</aside>
"#
    )
}

fn build_topbar(page_name: &str) -> String {
    format!(
        r#"<header class="topbar">
  <div class="topbar-left">
    <button class="hamburger" id="hamburger-btn" onclick="toggleSidebar()" aria-label="Toggle navigation">
      <span></span><span></span><span></span>
    </button>
    <span class="topbar-site">Turbo Gravity</span>
    <span class="topbar-sep"> | </span>
    <span class="topbar-page">{page_name}</span>
  </div>
  <div class="topbar-right">
    <input class="search-box" type="text" placeholder="Search&#x2026;" aria-label="Search" />
    <button class="notif-btn" title="Notifications" aria-label="Notifications"><span class="material-symbols-rounded mi-topbar">notifications</span><span class="notif-dot"></span></button>
    <button class="theme-btn" onclick="toggleTheme()" title="Toggle theme" aria-label="Toggle theme"><span class="material-symbols-rounded mi-topbar">dark_mode</span></button>
    <div class="avatar">TG</div>
  </div>
</header>
"#
    )
}

fn html_foot() -> &'static str {
    "</body>\n</html>\n"
}

// ---------------------------------------------------------------------------
// Dashboard page
// ---------------------------------------------------------------------------

pub struct DashboardData {
    pub bot_status:         &'static str,
    pub command_scope:      String,
    pub guild_id:           String,
    pub invite_link:        String,
    pub invite_permissions: String,
    pub online_status:      String,
    pub presence_text:      String,
    pub presence_type:      u8,
}

pub fn dashboard_page(data: &DashboardData) -> String {
    let head          = html_head("Dashboard", DASHBOARD_SCRIPT);
    let sidebar       = build_sidebar("Home");
    let topbar        = build_topbar("Dashboard");
    let foot          = html_foot();

    let status_class  = if data.bot_status == "online" { "online" } else { "offline" };
    let status_dot    = r#"<span class="status-dot"></span>"#;
    let status_text   = html_escape(data.bot_status);
    let invite_link   = html_escape(&data.invite_link);
    let invite_perms  = html_escape(&data.invite_permissions);
    let guild_id      = html_escape(&data.guild_id);
    let presence_text = html_escape(&data.presence_text);

    let scope_guild_sel  = if data.command_scope != "global" { " selected" } else { "" };
    let scope_global_sel = if data.command_scope == "global" { " selected" } else { "" };

    let ps_online    = if data.online_status == "online"    { " selected" } else { "" };
    let ps_dnd       = if data.online_status == "dnd"       { " selected" } else { "" };
    let ps_idle      = if data.online_status == "idle"      { " selected" } else { "" };
    let ps_invisible = if data.online_status == "invisible" { " selected" } else { "" };

    let pp0 = if data.presence_type == 0 { " selected" } else { "" };
    let pp1 = if data.presence_type == 1 { " selected" } else { "" };
    let pp2 = if data.presence_type == 2 { " selected" } else { "" };
    let pp3 = if data.presence_type == 3 { " selected" } else { "" };
    let pp4 = if data.presence_type == 4 { " selected" } else { "" };

    format!(
        r##"{head}
<div class="layout">
{sidebar}
<div class="sidebar-overlay" id="sidebar-overlay" onclick="toggleSidebar()"></div>
<div class="main-wrapper">
{topbar}
<main class="content">
  <div class="dashboard-grid">

    <!-- ── Left column ──────────────────────────────────────────────── -->
    <div class="col-left">

      <!-- Server Analytics -->
      <div class="card">
        <div class="card-header">
          <span class="card-title"><span class="material-symbols-rounded mi-card">show_chart</span> Server Analytics</span>
          <span class="card-badge">Last 30 days</span>
        </div>
        <div class="chart-wrap">
          <svg viewBox="0 0 400 140" xmlns="http://www.w3.org/2000/svg" preserveAspectRatio="none">
            <defs>
              <linearGradient id="purpleArea" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#7c3aed" stop-opacity="0.35"/>
                <stop offset="100%" stop-color="#7c3aed" stop-opacity="0"/>
              </linearGradient>
              <linearGradient id="cyanArea" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stop-color="#06b6d4" stop-opacity="0.3"/>
                <stop offset="100%" stop-color="#06b6d4" stop-opacity="0"/>
              </linearGradient>
            </defs>
            <line x1="0" y1="35"  x2="400" y2="35"  stroke="#1f2544" stroke-width="1"/>
            <line x1="0" y1="70"  x2="400" y2="70"  stroke="#1f2544" stroke-width="1"/>
            <line x1="0" y1="105" x2="400" y2="105" stroke="#1f2544" stroke-width="1"/>
            <path d="M0,112 C50,108 100,98 150,88 C200,78 250,65 300,52 C350,40 380,33 400,30 L400,140 L0,140 Z"
                  fill="url(#purpleArea)"/>
            <path d="M0,125 C50,120 100,115 150,105 C200,92 250,74 300,52 C350,32 380,22 400,18 L400,140 L0,140 Z"
                  fill="url(#cyanArea)"/>
            <path d="M0,112 C50,108 100,98 150,88 C200,78 250,65 300,52 C350,40 380,33 400,30"
                  fill="none" stroke="#7c3aed" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
            <path d="M0,125 C50,120 100,115 150,105 C200,92 250,74 300,52 C350,32 380,22 400,18"
                  fill="none" stroke="#06b6d4" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"/>
          </svg>
        </div>
        <div class="chart-legend">
          <div class="legend-item">
            <span class="legend-dot" style="background:#7c3aed"></span>
            Member growth
          </div>
          <div class="legend-item">
            <span class="legend-dot" style="background:#06b6d4"></span>
            Message activity
          </div>
        </div>
      </div>

      <!-- Bot Subgrid: Active Servers + Uptime (touch-friendly popovers) -->
      <div class="subgrid">
        <div class="subcard" data-popover="true" title="Tap for details">
          <div class="subcard-title">Active Servers</div>
          <div class="server-list">
            <div class="server-row">
              <span class="server-dot"></span>
              Server <code style="font-size:11px;color:var(--text3)">{guild_id}</code>
            </div>
            <div class="server-row">
              <span class="server-dot"></span>
              Bot status:
              <span class="status-badge {status_class}">{status_dot}{status_text}</span>
            </div>
          </div>
          <div class="stats-popover">
            <strong>Server Details</strong><br/>
            Guild ID: {guild_id}<br/>
            Status: {status_text}<br/>
            Region: Auto
          </div>
        </div>
        <div class="subcard" data-popover="true" title="Tap for details">
          <div class="subcard-title">Uptime</div>
          <div class="uptime-val">99.9%</div>
          <div class="uptime-label">Last 30 days</div>
          <div class="uptime-bar-wrap">
            <div class="uptime-bar" style="width:99.9%"></div>
          </div>
          <div class="stats-popover">
            <strong>Uptime Stats</strong><br/>
            30-day: 99.9%<br/>
            7-day: 100%<br/>
            Incidents: 0
          </div>
        </div>
      </div>

      <!-- Quick Actions -->
      <div class="card">
        <div class="card-header">
          <span class="card-title"><span class="material-symbols-rounded mi-card">bolt</span> Quick Actions</span>
        </div>
        <div class="btn-actions">
          <form method="POST" action="/control/restart" style="display:inline" onsubmit="controlAction(this,event)">
            <button class="btn btn-primary" type="submit"><span class="material-symbols-rounded mi-btn">restart_alt</span> Restart Bot</button>
          </form>
          <form method="POST" action="/control/stop" style="display:inline" onsubmit="controlAction(this,event)">
            <button class="btn btn-danger" type="submit"><span class="material-symbols-rounded mi-btn">stop_circle</span> Stop Bot</button>
          </form>
          <form method="POST" action="/control/clear-cache" style="display:inline" onsubmit="controlAction(this,event)">
            <button class="btn btn-ghost" type="submit"><span class="material-symbols-rounded mi-btn">mop</span> Clear Cache</button>
          </form>
          <form method="POST" action="/control/reload-commands" style="display:inline" onsubmit="controlAction(this,event)">
            <button class="btn btn-cyan" type="submit"><span class="material-symbols-rounded mi-btn">sync</span> Reload Commands</button>
          </form>
        </div>
        <form method="POST" action="/dashboard/settings">
          <div class="form-row">
            <div class="form-group">
              <label class="form-label">Command Scope</label>
              <select class="form-select" name="commandScope">
                <option value="guild"{scope_guild_sel}>Guild (recommended)</option>
                <option value="global"{scope_global_sel}>Global</option>
              </select>
            </div>
            <div class="form-group" style="align-self:flex-end">
              <button class="btn btn-primary" type="submit">Save Settings</button>
            </div>
          </div>
        </form>
      </div>

      <!-- Invite Link -->
      <div class="card">
        <div class="card-header">
          <span class="card-title"><span class="material-symbols-rounded mi-card">link</span> Invite Link</span>
          <span class="card-badge">permissions: {invite_perms}</span>
        </div>
        <p style="font-size:13px;color:var(--text2);margin-bottom:8px">
          Use this link to invite the bot to other servers.
        </p>
        <div class="invite-link-box">
          <span class="invite-link-text">{invite_link}</span>
          <a class="btn btn-primary" href="{invite_link}" target="_blank" rel="noopener noreferrer">
            Open <span class="material-symbols-rounded mi-btn">open_in_new</span>
          </a>
        </div>
      </div>

    </div><!-- /col-left -->

    <!-- ── Right column ─────────────────────────────────────────────── -->
    <div class="col-right">

      <!-- Bot Presence Settings -->
      <div class="card">
        <div class="card-header">
          <span class="card-title"><span class="material-symbols-rounded mi-card">emoji_emotions</span> Bot Presence</span>
        </div>
        <form method="POST" action="/dashboard/settings">
          <div class="setup-grid" style="margin-bottom:12px">
            <div class="setup-field">
              <label>Online Status</label>
              <select class="form-select" name="onlineStatus" style="width:100%">
                <option value="online"{ps_online}>Online</option>
                <option value="idle"{ps_idle}>Idle</option>
                <option value="dnd"{ps_dnd}>Do Not Disturb</option>
                <option value="invisible"{ps_invisible}>Invisible</option>
              </select>
            </div>
            <div class="setup-field">
              <label>Activity Type</label>
              <select class="form-select" name="presenceType" style="width:100%">
                <option value="0"{pp0}>Playing</option>
                <option value="1"{pp1}>Streaming</option>
                <option value="2"{pp2}>Listening to</option>
                <option value="3"{pp3}>Watching</option>
                <option value="4"{pp4}>Competing in</option>
              </select>
            </div>
            <div class="setup-field full">
              <label>Status Text
                <span class="infotip-wrap">
                  <button type="button" class="infotip-trigger" aria-label="Help for status text">?</button>
                  <span class="infotip-content" role="tooltip">
                    Supports dynamic variables:<br/>
                    <code>&#x7B;servers&#x7D;</code> — server count<br/>
                    <code>&#x7B;members&#x7D;</code> — total members (shows 0 at startup)
                  </span>
                </span>
              </label>
              <input class="form-input" type="text" name="presenceText" value="{presence_text}" placeholder="Ready to serve" style="width:100%" />
            </div>
          </div>
          <button class="btn btn-primary" type="submit" style="width:100%">
            <span class="material-symbols-rounded mi-btn">save</span> Save Presence
          </button>
        </form>
      </div>

      <!-- Bot Modules -->
      <div class="card">
        <div class="card-header">
          <span class="card-title"><span class="material-symbols-rounded mi-card">extension</span> Bot Modules</span>
          <span class="card-badge">6 modules</span>
        </div>
        <div class="module-grid">

          <div class="module-card">
            <div class="module-header">
              <div class="module-icon-name">
                <div class="module-icon"><span class="material-symbols-rounded mi-module">shield</span></div>
                <span class="module-name">Moderation</span>
              </div>
              <label class="toggle">
                <input type="checkbox" checked aria-label="Toggle Moderation module" />
                <span class="toggle-track"><span class="toggle-thumb"></span></span>
              </label>
            </div>
            <p class="module-desc">Kick, ban, mute &amp; warn members to keep your server safe.</p>
          </div>

          <div class="module-card">
            <div class="module-header">
              <div class="module-icon-name">
                <div class="module-icon"><span class="material-symbols-rounded mi-module">payments</span></div>
                <span class="module-name">Economy</span>
              </div>
              <label class="toggle">
                <input type="checkbox" checked aria-label="Toggle Economy module" />
                <span class="toggle-track"><span class="toggle-thumb"></span></span>
              </label>
            </div>
            <p class="module-desc">Virtual currency, shops, gambling &amp; leaderboards.</p>
          </div>

          <div class="module-card">
            <div class="module-header">
              <div class="module-icon-name">
                <div class="module-icon"><span class="material-symbols-rounded mi-module">celebration</span></div>
                <span class="module-name">Fun</span>
              </div>
              <label class="toggle">
                <input type="checkbox" checked aria-label="Toggle Fun module" />
                <span class="toggle-track"><span class="toggle-thumb"></span></span>
              </label>
            </div>
            <p class="module-desc">Games, memes &amp; entertainment for your community.</p>
          </div>

          <div class="module-card">
            <div class="module-header">
              <div class="module-icon-name">
                <div class="module-icon"><span class="material-symbols-rounded mi-module">military_tech</span></div>
                <span class="module-name">Leveling</span>
              </div>
              <label class="toggle">
                <input type="checkbox" aria-label="Toggle Leveling module" />
                <span class="toggle-track"><span class="toggle-thumb"></span></span>
              </label>
            </div>
            <p class="module-desc">XP system to reward your most active members.</p>
          </div>

          <div class="module-card">
            <div class="module-header">
              <div class="module-icon-name">
                <div class="module-icon"><span class="material-symbols-rounded mi-module">construction</span></div>
                <span class="module-name">Utilities</span>
              </div>
              <label class="toggle">
                <input type="checkbox" checked aria-label="Toggle Utilities module" />
                <span class="toggle-track"><span class="toggle-thumb"></span></span>
              </label>
            </div>
            <p class="module-desc">Polls, reminders, server info &amp; other handy tools.</p>
          </div>

          <div class="module-card">
            <div class="module-header">
              <div class="module-icon-name">
                <div class="module-icon"><span class="material-symbols-rounded mi-module">confirmation_number</span></div>
                <span class="module-name">Tickets</span>
              </div>
              <label class="toggle">
                <input type="checkbox" aria-label="Toggle Tickets module" />
                <span class="toggle-track"><span class="toggle-thumb"></span></span>
              </label>
            </div>
            <p class="module-desc">Support ticket system with categories &amp; transcripts.</p>
          </div>

        </div>
      </div>

      <!-- Configuration Backup / Restore -->
      <div class="card">
        <div class="card-header">
          <span class="card-title"><span class="material-symbols-rounded mi-card">backup</span> Configuration Backup</span>
        </div>
        <p style="font-size:13px;color:var(--text2);margin-bottom:14px">
          Download the current <code>config.toml</code> as a ZIP archive or
          restore a previously downloaded backup.
        </p>
        <div class="btn-actions" style="flex-wrap:wrap;gap:10px">
          <a class="btn btn-primary" href="/api/config/backup" download="config-backup.zip">
            <span class="material-symbols-rounded mi-btn">download</span> Download Backup
          </a>
          <label class="btn btn-ghost" style="cursor:pointer">
            <span class="material-symbols-rounded mi-btn">upload</span> Restore Backup
            <input type="file" accept=".zip" style="display:none" id="restore-file-input"
              onchange="restoreConfig(this)" />
          </label>
        </div>
        <p id="restore-status" style="font-size:12px;color:var(--text2);margin-top:10px;display:none"></p>
      </div>

    </div><!-- /col-right -->
  </div><!-- /dashboard-grid -->
</main>
</div><!-- /main-wrapper -->
</div><!-- /layout -->
<div id="toast-container" class="toast-container"></div>
{foot}"##
    )
}

// ---------------------------------------------------------------------------
// Setup page  (centered card, no sidebar)
// ---------------------------------------------------------------------------

pub struct SetupData {
    pub token:          String,
    pub owner_id:       String,
    pub client_id:      String,
    pub client_secret:  String,
    pub callback_url:   String,
    pub mongo_uri:      String,
    pub session_secret: String,
    pub guild_id:       String,
    pub port:           u16,
    pub presence_type:  u8,
    pub presence_text:  String,
    pub command_scope:  String,
    pub online_status:  String,
    pub avatar_url:     String,
}

pub fn setup_page(data: &SetupData) -> String {
    let head = html_head("Setup", DASHBOARD_SCRIPT);
    let foot = html_foot();

    let scope_guild_sel  = if data.command_scope != "global" { " selected" } else { "" };
    let scope_global_sel = if data.command_scope == "global" { " selected" } else { "" };
    let p0 = if data.presence_type == 0 { " selected" } else { "" };
    let p1 = if data.presence_type == 1 { " selected" } else { "" };
    let p2 = if data.presence_type == 2 { " selected" } else { "" };
    let p3 = if data.presence_type == 3 { " selected" } else { "" };
    let p4 = if data.presence_type == 4 { " selected" } else { "" };

    let ps_online    = if data.online_status == "online"    { " selected" } else { "" };
    let ps_dnd       = if data.online_status == "dnd"       { " selected" } else { "" };
    let ps_idle      = if data.online_status == "idle"      { " selected" } else { "" };
    let ps_invisible = if data.online_status == "invisible" { " selected" } else { "" };

    let callback_url = if data.callback_url.is_empty() {
        if data.port == crate::config::DEFAULT_PORT {
            crate::config::DEFAULT_CALLBACK_URL.to_string()
        } else {
            format!("http://localhost:{}/auth/discord/callback", data.port)
        }
    } else {
        data.callback_url.clone()
    };

    let token          = html_escape(&data.token);
    let client_id      = html_escape(&data.client_id);
    let client_secret  = html_escape(&data.client_secret);
    let callback_url_e = html_escape(&callback_url);
    let mongo_uri      = html_escape(&data.mongo_uri);
    let session_secret = html_escape(&data.session_secret);
    let owner_id       = html_escape(&data.owner_id);
    let guild_id       = html_escape(&data.guild_id);
    let port           = data.port;
    let presence_text  = html_escape(&data.presence_text);
    let avatar_url     = html_escape(&data.avatar_url);

    format!(
        r#"{head}
<div class="setup-page">
  <div class="setup-card">

    <div class="setup-header">
      <div class="setup-logo"><span class="material-symbols-rounded mi-setup">bolt</span></div>
      <h1>Turbo Gravity Setup</h1>
      <p>Configure your Discord bot &#x2014; no coding required.</p>
      <p style="margin-top:8px;font-size:12px;color:var(--text3)">
        Need your credentials?
        <a href="https://discord.com/developers/applications" target="_blank" rel="noopener noreferrer"
           style="color:var(--purple2);text-decoration:underline">
          Open Discord Developer Portal &#x2197;
        </a>
      </p>
    </div>

    <form method="POST" action="/setup">

      <!-- ── Section 1: Discord Credentials ──────────────────────── -->
      <div class="setup-section">
        <div class="setup-section-header">
          <span class="setup-section-icon"><span class="material-symbols-rounded mi-setup">smart_toy</span></span>
          <span class="setup-section-title">Discord Credentials</span>
        </div>
        <div class="setup-grid">
          <div class="setup-field full">
            <label>
              Bot Token
              <span class="infotip-wrap">
                <button type="button" class="infotip-trigger" aria-label="Help: where to find Bot Token">?</button>
                <span class="infotip-content" role="tooltip">
                  In the <a href="https://discord.com/developers/applications" target="_blank" rel="noopener">Developer Portal</a>
                  &#x2192; Your App &#x2192; <strong>Bot</strong> &#x2192; Reset Token.
                </span>
              </span>
            </label>
            <input type="password" name="botToken" value="{token}" placeholder="Bot token from the Developer Portal" autocomplete="new-password" required />
          </div>
          <div class="setup-field">
            <label>
              Client ID (Application ID)
              <span class="infotip-wrap">
                <button type="button" class="infotip-trigger" aria-label="Help: where to find Client ID">?</button>
                <span class="infotip-content" role="tooltip">
                  In the <a href="https://discord.com/developers/applications" target="_blank" rel="noopener">Developer Portal</a>
                  &#x2192; Your App &#x2192; <strong>General Information</strong> &#x2192; Application ID.
                </span>
              </span>
            </label>
            <input type="text" name="clientId" value="{client_id}" placeholder="e.g. 1234567890" required />
          </div>
          <div class="setup-field">
            <label>
              Client Secret
              <span class="infotip-wrap">
                <button type="button" class="infotip-trigger" aria-label="Help: where to find Client Secret">?</button>
                <span class="infotip-content" role="tooltip">
                  In the <a href="https://discord.com/developers/applications" target="_blank" rel="noopener">Developer Portal</a>
                  &#x2192; Your App &#x2192; <strong>OAuth2</strong> &#x2192; Client Secret (reset if hidden).
                </span>
              </span>
            </label>
            <input type="password" name="clientSecret" value="{client_secret}" placeholder="OAuth2 client secret" autocomplete="new-password" />
          </div>
          <div class="setup-field full">
            <label>
              OAuth2 Callback URL
              <span class="infotip-wrap">
                <button type="button" class="infotip-trigger" aria-label="Help: OAuth2 Callback URL">?</button>
                <span class="infotip-content" role="tooltip">
                  Add this URL under <strong>OAuth2 &#x2192; Redirects</strong> in the Developer Portal.
                  Must match exactly.
                </span>
              </span>
            </label>
            <input type="url" name="callbackUrl" value="{callback_url_e}" required />
          </div>
        </div>
      </div>

      <!-- ── Section 2: Database &amp; Security ─────────────────── -->
      <div class="setup-section">
        <div class="setup-section-header">
          <span class="setup-section-icon"><span class="material-symbols-rounded mi-setup">storage</span></span>
          <span class="setup-section-title">Database &amp; Security</span>
        </div>
        <div class="setup-grid">
          <div class="setup-field full">
            <label>
              MongoDB URI
              <span class="infotip-wrap">
                <button type="button" class="infotip-trigger" aria-label="Help: MongoDB URI">?</button>
                <span class="infotip-content" role="tooltip">
                  Leave blank to run without a database. Format:<br/>
                  <code>mongodb://localhost:27017</code><br/>
                  or <code>mongodb+srv://user:pass@cluster</code>
                </span>
              </span>
            </label>
            <input type="text" name="mongoUri" value="{mongo_uri}" placeholder="mongodb://localhost:27017/turbogravity" />
          </div>
          <div class="setup-field full">
            <label>Session Secret</label>
            <input type="password" name="sessionSecret" value="{session_secret}" placeholder="Random secure string" autocomplete="new-password" />
          </div>
          <div class="setup-field full">
            <label>
              Admin User IDs (comma-separated Discord user IDs)
              <span class="infotip-wrap">
                <button type="button" class="infotip-trigger" aria-label="Help: Admin User IDs">?</button>
                <span class="infotip-content" role="tooltip">
                  Your Discord user ID. Enable <strong>Developer Mode</strong> in Discord settings,
                  then right-click your username &#x2192; Copy User ID.
                </span>
              </span>
            </label>
            <input type="text" name="adminIds" value="{owner_id}" placeholder="123456789,987654321" />
          </div>
        </div>
      </div>

      <!-- ── Section 3: Bot Settings ─────────────────────────────── -->
      <div class="setup-section">
        <div class="setup-section-header">
          <span class="setup-section-icon"><span class="material-symbols-rounded mi-setup">settings</span></span>
          <span class="setup-section-title">Bot Settings</span>
        </div>
        <div class="setup-grid">
          <div class="setup-field">
            <label>
              Guild ID (leave blank for global commands)
              <span class="infotip-wrap">
                <button type="button" class="infotip-trigger" aria-label="Help: Guild ID">?</button>
                <span class="infotip-content" role="tooltip">
                  Your Discord server ID. Enable Developer Mode, then right-click your server &#x2192; Copy Server ID.
                </span>
              </span>
            </label>
            <input type="text" name="guildId" value="{guild_id}" placeholder="Your server&#x27;s ID" />
          </div>
          <div class="setup-field">
            <label>Dashboard Port</label>
            <input type="number" name="port" value="{port}" min="1" max="65535" required />
          </div>
          <div class="setup-field">
            <label>Online Status</label>
            <select name="onlineStatus">
              <option value="online"{ps_online}>Online</option>
              <option value="idle"{ps_idle}>Idle</option>
              <option value="dnd"{ps_dnd}>Do Not Disturb</option>
              <option value="invisible"{ps_invisible}>Invisible</option>
            </select>
          </div>
          <div class="setup-field">
            <label>Presence Type</label>
            <select name="presenceType">
              <option value="0"{p0}>Playing</option>
              <option value="1"{p1}>Streaming</option>
              <option value="2"{p2}>Listening</option>
              <option value="3"{p3}>Watching</option>
              <option value="4"{p4}>Competing</option>
            </select>
          </div>
          <div class="setup-field">
            <label>Command Scope</label>
            <select name="commandScope">
              <option value="guild"{scope_guild_sel}>Guild (recommended)</option>
              <option value="global"{scope_global_sel}>Global</option>
            </select>
          </div>
          <div class="setup-field">
            <label>
              Avatar URL
              <span class="infotip-wrap">
                <button type="button" class="infotip-trigger" aria-label="Help: Avatar URL">?</button>
                <span class="infotip-content" role="tooltip">
                  Optional URL to a PNG/JPG image used as the bot&#x27;s avatar.
                  Leave blank to keep the current avatar.
                </span>
              </span>
            </label>
            <input type="url" name="avatarUrl" value="{avatar_url}" placeholder="https://example.com/avatar.png" />
          </div>
          <div class="setup-field full">
            <label>
              Presence Text
              <span class="infotip-wrap">
                <button type="button" class="infotip-trigger" aria-label="Help: Presence Text">?</button>
                <span class="infotip-content" role="tooltip">
                  Supports dynamic variables:<br/>
                  <code>&#x7B;servers&#x7D;</code> &#x2014; server count<br/>
                  <code>&#x7B;members&#x7D;</code> &#x2014; total members (shows 0 at startup)
                </span>
              </span>
            </label>
            <input type="text" name="presenceText" value="{presence_text}" placeholder="Ready to serve" />
          </div>
        </div>
      </div>

      <button class="setup-submit" type="submit"><span class="material-symbols-rounded mi-btn">save</span> Save &amp; Start</button>
    </form>

  </div>
</div>
{foot}"#
    )
}

// ---------------------------------------------------------------------------
// Guild selector page
// ---------------------------------------------------------------------------

pub struct GuildInfo {
    pub id:           String,
    pub name:         String,
    pub icon:         Option<String>,
    pub member_count: Option<u64>,
}

pub struct SelectorData {
    pub username:   String,
    pub guilds:     Vec<GuildInfo>,
    pub bot_status: &'static str,
}

pub fn selector_page(data: &SelectorData) -> String {
    let head    = html_head("Select Server", "");
    let sidebar = build_sidebar("Servers");
    let topbar  = build_topbar("Select Server");
    let foot    = html_foot();

    let status_class = if data.bot_status == "online" { "online" } else { "offline" };
    let username     = html_escape(&data.username);

    let guild_cards: String = data
        .guilds
        .iter()
        .map(|g| {
            let name  = html_escape(&g.name);
            let gid   = html_escape(&g.id);
            let icon_html = match &g.icon {
                Some(hash) => format!(
                    r#"<img src="https://cdn.discordapp.com/icons/{gid}/{hash}.webp?size=128" alt="{name}" />"#,
                    hash = html_escape(hash)
                ),
                None => {
                    let initial: String = g.name.chars().next().unwrap_or('?').to_uppercase().collect();
                    format!(r#"<span style="font-size:24px;font-weight:700">{initial}</span>"#)
                }
            };
            let members_html = match g.member_count {
                Some(n) => format!(r#"<span class="guild-members">{n} members</span>"#),
                None    => String::new(),
            };
            format!(
                r#"<a class="guild-card" href="/dashboard?guild={gid}" aria-label="Select server {name}">
  <div class="guild-icon">{icon_html}</div>
  <span class="guild-name">{name}</span>
  {members_html}
  <span class="guild-select-btn" aria-hidden="true">Select Server</span>
</a>
"#
            )
        })
        .collect();

    let empty_msg = if data.guilds.is_empty() {
        r#"<p style="color:var(--text2);text-align:center;padding:40px 0">
  No servers found where you have Administrator permissions.
</p>"#
    } else {
        ""
    };

    format!(
        r#"{head}
<div class="layout">
{sidebar}
<div class="sidebar-overlay" id="sidebar-overlay" onclick="toggleSidebar()"></div>
<div class="main-wrapper">
{topbar}
<div class="selector-content">
  <div class="selector-header">
    <h1>Select a Server</h1>
    <p>Welcome back, <strong>{username}</strong>. Bot is
      <span class="status-badge {status_class}">
        <span class="status-dot"></span>{bot_status}
      </span>
    </p>
  </div>
  <div class="guild-grid">
    {guild_cards}
  </div>
  {empty_msg}
</div>
</div>
</div>
{foot}"#,
        bot_status = html_escape(data.bot_status),
    )
}

// ---------------------------------------------------------------------------
// Setup-complete page  (centered card, shown after the setup wizard is saved)
// ---------------------------------------------------------------------------

/// Rendered after a successful `POST /setup` submission.
/// The bot starts automatically — no manual restart is required.
pub fn setup_complete_page(dashboard_port: u16) -> String {
    let poll_script = format!(
        r#"<script>
(function() {{
  var port = {dashboard_port};
  var proto = window.location.protocol;
  var url = new URL(window.location.href);
  var isDefaultPort = (proto === 'https:' && port === 443) || (proto === 'http:' && port === 80);
  url.port = isDefaultPort ? '' : String(port);
  var origin = url.origin;
  var attempts = 0;
  var maxAttempts = 60;

  function updateStatus(msg) {{
    var el = document.getElementById('starting-status');
    if (el) {{ el.textContent = msg; }}
  }}

  function poll() {{
    attempts += 1;
    if (attempts > maxAttempts) {{
      updateStatus('Bot startup is taking longer than expected. Please navigate to the dashboard manually.');
      return;
    }}
    fetch(origin + '/health', {{ cache: 'no-store' }})
      .then(function(r) {{
        if (r.ok) {{
          updateStatus('Bot is online! Redirecting to dashboard…');
          window.location.href = origin + '/dashboard';
        }} else {{
          setTimeout(poll, 2000);
        }}
      }})
      .catch(function() {{
        setTimeout(poll, 2000);
      }});
  }}

  document.addEventListener('DOMContentLoaded', function() {{
    setTimeout(poll, 3000);
  }});
}})();
</script>"#,
        dashboard_port = dashboard_port
    );

    let head = html_head("Setup Complete", &poll_script);
    let foot = html_foot();

    format!(
        r#"{head}
<div class="setup-page">
  <div class="setup-card">
    <div class="setup-header">
      <div class="setup-logo"><span class="material-symbols-rounded mi-setup" style="color:var(--green)">check_circle</span></div>
      <h1>Setup Complete!</h1>
      <p>Your configuration has been saved to <code>config.toml</code>.</p>
    </div>

    <div class="setup-section">
      <div class="setup-section-header">
        <span class="setup-section-icon"><span class="material-symbols-rounded mi-setup">rocket_launch</span></span>
        <span class="setup-section-title">Bot is Starting&hellip;</span>
      </div>
      <p style="color:var(--text2);margin-bottom:14px">
        The bot is connecting to Discord automatically. This page will
        redirect to the dashboard automatically once the bot is online.
      </p>
      <p id="starting-status" aria-live="polite" style="color:var(--text2);font-size:13px;margin-top:8px">
        Waiting for bot to start&hellip;
      </p>
    </div>

    <div style="margin-top:28px;display:flex;gap:12px;justify-content:center">
      <a class="btn btn-ghost" href="/setup">&#x2190; Edit Configuration</a>
    </div>
  </div>
</div>
{foot}"#
    )
}

// ---------------------------------------------------------------------------
// Error page
// ---------------------------------------------------------------------------

pub struct ErrorData {
    pub code:    u16,
    pub title:   String,
    pub message: String,
}

pub fn error_page(data: &ErrorData) -> String {
    let head  = html_head("Error", "");
    let foot  = html_foot();
    let code  = data.code;
    let title = html_escape(&data.title);
    let msg   = html_escape(&data.message);

    format!(
        r#"{head}
<div class="error-page">
  <div class="error-card">
    <div class="error-code">{code}</div>
    <div class="error-title">{title}</div>
    <div class="error-message">{msg}</div>
    <a class="btn btn-primary" href="/dashboard"><span class="material-symbols-rounded mi-btn">arrow_back</span> Back to Dashboard</a>
  </div>
</div>
{foot}"#
    )
}

// ---------------------------------------------------------------------------
// HTML escape helper
// ---------------------------------------------------------------------------

pub fn html_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&'  => out.push_str("&amp;"),
            '<'  => out.push_str("&lt;"),
            '>'  => out.push_str("&gt;"),
            '"'  => out.push_str("&quot;"),
            '\'' => out.push_str("&#x27;"),
            _    => out.push(ch),
        }
    }
    out
}
