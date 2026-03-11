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
}
a { color: inherit; text-decoration: none; }
button { cursor: pointer; font-family: inherit; }
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

/* ── Scrollbar ───────────────────────────────────────────────────────────── */
::-webkit-scrollbar { width: 6px; height: 6px; }
::-webkit-scrollbar-track  { background: var(--bg); }
::-webkit-scrollbar-thumb  { background: var(--bg3); border-radius: 3px; }

/* ── Responsive ──────────────────────────────────────────────────────────── */
@media (max-width: 1100px) {
  .dashboard-grid { grid-template-columns: 1fr; }
}
@media (max-width: 768px) {
  .sidebar         { transform: translateX(-220px); }
  .main-wrapper    { margin-left: 0; }
  .selector-content { margin-left: 0; }
  .setup-grid      { grid-template-columns: 1fr; }
}
"#;

// ---------------------------------------------------------------------------
// Theme-toggle JS – raw string keeps JS braces away from format! escaping
// ---------------------------------------------------------------------------

const THEME_SCRIPT: &str = r#"<script>
(function() {
  var t = localStorage.getItem('theme');
  var h = document.documentElement;
  if (t === 'light')       { h.className = 'light'; }
  else if (t === 'system') { h.className = ''; }
  else                     { h.className = 'dark'; }
})();
function toggleTheme() {
  var h = document.documentElement;
  var t = localStorage.getItem('theme') || 'dark';
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
        ("\u{1F3E0}", "Home",      "/dashboard"),
        ("\u{1F4CA}", "Analytics", "/dashboard"),
        ("\u{1F5A5}", "Servers",   "/selector"),
        ("\u{1F9E9}", "Modules",   "/dashboard"),
        ("\u{2699}",  "Settings",  "/setup"),
    ];
    let nav: String = items
        .iter()
        .map(|(icon, name, href)| {
            let cls = if *name == active { " active" } else { "" };
            format!(
                r#"    <a href="{href}" class="nav-item{cls}"><span class="nav-icon">{icon}</span>{name}</a>
"#
            )
        })
        .collect();
    format!(
        r#"<aside class="sidebar">
  <div class="sidebar-brand">
    <div class="brand-dot">&#x26A1;</div>
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
    <span class="topbar-site">Turbo Gravity</span>
    <span class="topbar-sep"> | </span>
    <span class="topbar-page">{page_name}</span>
  </div>
  <div class="topbar-right">
    <input class="search-box" type="text" placeholder="Search&#x2026;" />
    <button class="notif-btn" title="Notifications">&#x1F514;<span class="notif-dot"></span></button>
    <button class="theme-btn" onclick="toggleTheme()" title="Toggle theme">&#x1F319;</button>
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
    pub bot_status:        &'static str,
    pub command_scope:     String,
    pub guild_id:          String,
    pub invite_link:       String,
    pub invite_permissions: String,
}

pub fn dashboard_page(data: &DashboardData) -> String {
    let head          = html_head("Dashboard", "");
    let sidebar       = build_sidebar("Home");
    let topbar        = build_topbar("Dashboard");
    let foot          = html_foot();

    let status_class  = if data.bot_status == "online" { "online" } else { "offline" };
    let status_dot    = r#"<span class="status-dot"></span>"#;
    let status_text   = html_escape(data.bot_status);
    let invite_link   = html_escape(&data.invite_link);
    let invite_perms  = html_escape(&data.invite_permissions);
    let guild_id      = html_escape(&data.guild_id);

    let scope_guild_sel  = if data.command_scope != "global" { " selected" } else { "" };
    let scope_global_sel = if data.command_scope == "global" { " selected" } else { "" };

    format!(
        r##"{head}
<div class="layout">
{sidebar}
<div class="main-wrapper">
{topbar}
<main class="content">
  <div class="dashboard-grid">

    <!-- ── Left column ──────────────────────────────────────────────── -->
    <div class="col-left">

      <!-- Server Analytics -->
      <div class="card">
        <div class="card-header">
          <span class="card-title">&#x1F4C8; Server Analytics</span>
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

      <!-- Bot Subgrid: Active Servers + Uptime -->
      <div class="subgrid">
        <div class="subcard">
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
        </div>
        <div class="subcard">
          <div class="subcard-title">Uptime</div>
          <div class="uptime-val">99.9%</div>
          <div class="uptime-label">Last 30 days</div>
          <div class="uptime-bar-wrap">
            <div class="uptime-bar" style="width:99.9%"></div>
          </div>
        </div>
      </div>

      <!-- Quick Actions -->
      <div class="card">
        <div class="card-header">
          <span class="card-title">&#x26A1; Quick Actions</span>
        </div>
        <div class="btn-actions">
          <form method="POST" action="/control/restart" style="display:inline">
            <button class="btn btn-primary" type="submit">&#x1F504; Restart Bot</button>
          </form>
          <form method="POST" action="/control/stop" style="display:inline">
            <button class="btn btn-danger" type="submit">&#x23F9; Stop Bot</button>
          </form>
          <form method="POST" action="/control/clear-cache" style="display:inline">
            <button class="btn btn-ghost" type="submit">&#x1F9F9; Clear Cache</button>
          </form>
          <form method="POST" action="/control/reload-commands" style="display:inline">
            <button class="btn btn-cyan" type="submit">&#x1F501; Reload Commands</button>
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
          <span class="card-title">&#x1F517; Invite Link</span>
          <span class="card-badge">permissions: {invite_perms}</span>
        </div>
        <p style="font-size:13px;color:var(--text2);margin-bottom:8px">
          Use this link to invite the bot to other servers.
        </p>
        <div class="invite-link-box">
          <span class="invite-link-text">{invite_link}</span>
          <a class="btn btn-primary" href="{invite_link}" target="_blank" rel="noopener noreferrer">
            Open &#x2197;
          </a>
        </div>
      </div>

    </div><!-- /col-left -->

    <!-- ── Right column ─────────────────────────────────────────────── -->
    <div class="col-right">

      <!-- Bot Modules -->
      <div class="card">
        <div class="card-header">
          <span class="card-title">&#x1F9E9; Bot Modules</span>
          <span class="card-badge">6 modules</span>
        </div>
        <div class="module-grid">

          <div class="module-card">
            <div class="module-header">
              <div class="module-icon-name">
                <div class="module-icon">&#x1F6E1;</div>
                <span class="module-name">Moderation</span>
              </div>
              <label class="toggle">
                <input type="checkbox" checked />
                <span class="toggle-track"><span class="toggle-thumb"></span></span>
              </label>
            </div>
            <p class="module-desc">Kick, ban, mute &amp; warn members to keep your server safe.</p>
          </div>

          <div class="module-card">
            <div class="module-header">
              <div class="module-icon-name">
                <div class="module-icon">&#x1F4B0;</div>
                <span class="module-name">Economy</span>
              </div>
              <label class="toggle">
                <input type="checkbox" checked />
                <span class="toggle-track"><span class="toggle-thumb"></span></span>
              </label>
            </div>
            <p class="module-desc">Virtual currency, shops, gambling &amp; leaderboards.</p>
          </div>

          <div class="module-card">
            <div class="module-header">
              <div class="module-icon-name">
                <div class="module-icon">&#x1F389;</div>
                <span class="module-name">Fun</span>
              </div>
              <label class="toggle">
                <input type="checkbox" checked />
                <span class="toggle-track"><span class="toggle-thumb"></span></span>
              </label>
            </div>
            <p class="module-desc">Games, memes &amp; entertainment for your community.</p>
          </div>

          <div class="module-card">
            <div class="module-header">
              <div class="module-icon-name">
                <div class="module-icon">&#x2B50;</div>
                <span class="module-name">Leveling</span>
              </div>
              <label class="toggle">
                <input type="checkbox" />
                <span class="toggle-track"><span class="toggle-thumb"></span></span>
              </label>
            </div>
            <p class="module-desc">XP &amp; rank system to reward your most active members.</p>
          </div>

          <div class="module-card">
            <div class="module-header">
              <div class="module-icon-name">
                <div class="module-icon">&#x1F527;</div>
                <span class="module-name">Utilities</span>
              </div>
              <label class="toggle">
                <input type="checkbox" checked />
                <span class="toggle-track"><span class="toggle-thumb"></span></span>
              </label>
            </div>
            <p class="module-desc">Polls, reminders, server info &amp; other handy tools.</p>
          </div>

          <div class="module-card">
            <div class="module-header">
              <div class="module-icon-name">
                <div class="module-icon">&#x1F3AB;</div>
                <span class="module-name">Tickets</span>
              </div>
              <label class="toggle">
                <input type="checkbox" />
                <span class="toggle-track"><span class="toggle-thumb"></span></span>
              </label>
            </div>
            <p class="module-desc">Support ticket system with categories &amp; transcripts.</p>
          </div>

        </div>
      </div>

    </div><!-- /col-right -->
  </div><!-- /dashboard-grid -->
</main>
</div><!-- /main-wrapper -->
</div><!-- /layout -->
{foot}"##
    )
}

// ---------------------------------------------------------------------------
// Setup page  (centered card, no sidebar)
// ---------------------------------------------------------------------------

pub struct SetupData {
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
}

pub fn setup_page(data: &SetupData) -> String {
    let head = html_head("Setup", "");
    let foot = html_foot();

    let scope_guild_sel  = if data.command_scope != "global" { " selected" } else { "" };
    let scope_global_sel = if data.command_scope == "global" { " selected" } else { "" };
    let p0 = if data.presence_type == 0 { " selected" } else { "" };
    let p1 = if data.presence_type == 1 { " selected" } else { "" };
    let p2 = if data.presence_type == 2 { " selected" } else { "" };
    let p3 = if data.presence_type == 3 { " selected" } else { "" };
    let p4 = if data.presence_type == 4 { " selected" } else { "" };

    let callback_url = if data.callback_url.is_empty() {
        if data.port == crate::config::DEFAULT_PORT {
            crate::config::DEFAULT_CALLBACK_URL.to_string()
        } else {
            format!("http://localhost:{}/auth/discord/callback", data.port)
        }
    } else {
        data.callback_url.clone()
    };

    let client_id      = html_escape(&data.client_id);
    let client_secret  = html_escape(&data.client_secret);
    let callback_url_e = html_escape(&callback_url);
    let mongo_uri      = html_escape(&data.mongo_uri);
    let session_secret = html_escape(&data.session_secret);
    let owner_id       = html_escape(&data.owner_id);
    let guild_id       = html_escape(&data.guild_id);
    let port           = data.port;
    let presence_text  = html_escape(&data.presence_text);

    format!(
        r#"{head}
<div class="setup-page">
  <div class="setup-card">

    <div class="setup-header">
      <div class="setup-logo">&#x26A1;</div>
      <h1>Turbo Gravity Setup</h1>
      <p>Configure your Discord bot &#x2014; no coding required.</p>
    </div>

    <form method="POST" action="/setup">

      <!-- ── Section 1: Discord Credentials ──────────────────────── -->
      <div class="setup-section">
        <div class="setup-section-header">
          <span class="setup-section-icon">&#x1F916;</span>
          <span class="setup-section-title">Discord Credentials</span>
        </div>
        <div class="setup-grid">
          <div class="setup-field">
            <label>Client ID (Application ID)</label>
            <input type="text" name="clientId" value="{client_id}" placeholder="e.g. 1234567890" required />
          </div>
          <div class="setup-field">
            <label>Client Secret</label>
            <input type="password" name="clientSecret" value="{client_secret}" placeholder="OAuth2 client secret" autocomplete="new-password" />
          </div>
          <div class="setup-field full">
            <label>OAuth2 Callback URL</label>
            <input type="url" name="callbackUrl" value="{callback_url_e}" required />
          </div>
        </div>
      </div>

      <!-- ── Section 2: Database &amp; Security ─────────────────── -->
      <div class="setup-section">
        <div class="setup-section-header">
          <span class="setup-section-icon">&#x1F5C4;</span>
          <span class="setup-section-title">Database &amp; Security</span>
        </div>
        <div class="setup-grid">
          <div class="setup-field full">
            <label>MongoDB URI</label>
            <input type="text" name="mongoUri" value="{mongo_uri}" placeholder="mongodb://localhost:27017/turbogravity" />
          </div>
          <div class="setup-field full">
            <label>Session Secret</label>
            <input type="password" name="sessionSecret" value="{session_secret}" placeholder="Random secure string" autocomplete="new-password" />
          </div>
          <div class="setup-field full">
            <label>Admin User IDs (comma-separated Discord user IDs)</label>
            <input type="text" name="adminIds" value="{owner_id}" placeholder="123456789,987654321" />
          </div>
        </div>
      </div>

      <!-- ── Section 3: Bot Settings ─────────────────────────────── -->
      <div class="setup-section">
        <div class="setup-section-header">
          <span class="setup-section-icon">&#x2699;</span>
          <span class="setup-section-title">Bot Settings</span>
        </div>
        <div class="setup-grid">
          <div class="setup-field">
            <label>Guild ID (leave blank for global commands)</label>
            <input type="text" name="guildId" value="{guild_id}" placeholder="Your server&#x27;s ID" />
          </div>
          <div class="setup-field">
            <label>Dashboard Port</label>
            <input type="number" name="port" value="{port}" min="1" max="65535" required />
          </div>
          <div class="setup-field">
            <label>Presence Type</label>
            <select name="presenceType">
              <option value="0"{p0}>&#x1F3AE; Playing</option>
              <option value="1"{p1}>&#x1F4FA; Streaming</option>
              <option value="2"{p2}>&#x1F3B5; Listening</option>
              <option value="3"{p3}>&#x1F440; Watching</option>
              <option value="4"{p4}>&#x1F3C6; Competing</option>
            </select>
          </div>
          <div class="setup-field">
            <label>Command Scope</label>
            <select name="commandScope">
              <option value="guild"{scope_guild_sel}>Guild (recommended)</option>
              <option value="global"{scope_global_sel}>Global</option>
            </select>
          </div>
          <div class="setup-field full">
            <label>Presence Text</label>
            <input type="text" name="presenceText" value="{presence_text}" placeholder="Ready to serve" />
          </div>
        </div>
      </div>

      <button class="setup-submit" type="submit">&#x1F4BE; Save &amp; Start</button>
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
                r#"<a class="guild-card" href="/dashboard?guild={gid}">
  <div class="guild-icon">{icon_html}</div>
  <span class="guild-name">{name}</span>
  {members_html}
  <button class="guild-select-btn" type="button">Select Server</button>
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
    <a class="btn btn-primary" href="/dashboard">&#x2190; Back to Dashboard</a>
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
