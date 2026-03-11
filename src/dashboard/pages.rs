/// Inline HTML page generators for the Turbo Gravity dashboard.
///
/// These replace the former EJS templates (`views/*.ejs`) and the external
/// CSS file (`public/styles.css`).  All markup is rendered by pure Rust string
/// formatting so no template-engine dependency is needed.

// ---------------------------------------------------------------------------
// Shared CSS (was public/styles.css + per-page inline styles)
// ---------------------------------------------------------------------------

pub const STYLES: &str = r#":root {
  --bg: #050b18;
  --bg-accent: linear-gradient(135deg, rgba(20, 184, 166, 0.18), rgba(234, 179, 8, 0.2));
  --panel: rgba(255, 255, 255, 0.04);
  --panel-strong: rgba(255, 255, 255, 0.08);
  --text: #e5e9f0;
  --muted: #8aa0b5;
  --primary: #0ea5e9;
  --primary-strong: #0284c7;
  --success: #22c55e;
  --danger: #ef4444;
  --shadow: 0 15px 50px rgba(0, 0, 0, 0.35);
}

:root:not(.dark) {
  --bg: #f6f9fc;
  --bg-accent: linear-gradient(135deg, rgba(14, 165, 233, 0.12), rgba(234, 179, 8, 0.12));
  --panel: #ffffff;
  --panel-strong: #ffffff;
  --text: #0b1628;
  --muted: #52627a;
  --primary: #0284c7;
  --primary-strong: #0369a1;
  --success: #16a34a;
  --danger: #dc2626;
  --shadow: 0 16px 40px rgba(0, 0, 0, 0.12);
}

* { box-sizing: border-box; }
body {
  margin: 0;
  font-family: 'Inter', system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
  background: var(--bg);
  color: var(--text);
  min-height: 100vh;
}

a { color: inherit; text-decoration: none; }

.backdrop {
  position: fixed;
  inset: 0;
  background: var(--bg-accent);
  filter: blur(80px);
  z-index: 0;
}

.topbar {
  position: sticky;
  top: 0;
  z-index: 10;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 18px 24px;
  background: rgba(5, 11, 24, 0.6);
  backdrop-filter: blur(12px);
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.brand { display: flex; gap: 12px; align-items: center; }
.brand h1 { margin: 0; font-size: 18px; }
.brand p { margin: 2px 0 0; color: var(--muted); }
.dot { width: 12px; height: 12px; border-radius: 50%; background: var(--primary); box-shadow: 0 0 16px var(--primary); }

.topbar-actions { display: flex; gap: 12px; align-items: center; }
.user-chip { display: flex; gap: 8px; align-items: center; padding: 8px 12px; background: var(--panel); border: 1px solid rgba(255, 255, 255, 0.08); border-radius: 999px; }
.pill { padding: 4px 8px; border-radius: 999px; background: var(--panel-strong); font-size: 12px; color: var(--muted); }

.page { position: relative; z-index: 1; padding: 24px; display: flex; flex-direction: column; gap: 20px; }

.hero {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 20px;
  padding: 24px;
  background: var(--panel);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 18px;
  box-shadow: var(--shadow);
}

.hero h2 { margin: 6px 0 10px; }
.hero .muted { color: var(--muted); margin: 0; }
.hero-actions { display: flex; gap: 12px; align-items: center; flex-wrap: wrap; margin-top: 12px; }

.hero-card {
  background: var(--panel-strong);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 14px;
  padding: 18px;
  box-shadow: var(--shadow);
}

.grid.two { display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 20px; }

.card {
  background: var(--panel);
  border-radius: 16px;
  padding: 18px;
  box-shadow: var(--shadow);
  border: 1px solid rgba(255, 255, 255, 0.06);
}

.card-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
.eyebrow { text-transform: uppercase; letter-spacing: 0.1em; font-size: 11px; color: var(--muted); margin: 0; }

.actions { display: flex; gap: 12px; flex-wrap: wrap; margin-top: 10px; }
.button {
  border: none;
  border-radius: 12px;
  padding: 12px 16px;
  background: var(--panel-strong);
  color: #fff;
  cursor: pointer;
  transition: transform 0.12s ease, box-shadow 0.2s ease, background 0.2s ease;
  text-align: center;
  border: 1px solid rgba(255, 255, 255, 0.06);
  font-size: 14px;
  font-weight: 500;
  display: inline-block;
  text-decoration: none;
}

.button:hover { transform: translateY(-1px); box-shadow: var(--shadow); }
.button.primary { background: var(--primary); border-color: var(--primary-strong); }
.button.success { background: var(--success); border-color: rgba(34, 197, 94, 0.4); }
.button.danger { background: var(--danger); border-color: rgba(239, 68, 68, 0.4); }
.button.ghost { background: transparent; border: 1px solid rgba(255, 255, 255, 0.16); color: var(--text); }

.form { display: grid; gap: 12px; }
.dual { display: grid; gap: 12px; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); }
label { display: grid; gap: 6px; color: var(--muted); font-size: 14px; }
input, select {
  padding: 11px 12px;
  border-radius: 10px;
  border: 1px solid rgba(255, 255, 255, 0.12);
  background: rgba(255, 255, 255, 0.03);
  color: var(--text);
  font-family: inherit;
  font-size: 14px;
}

.checkbox { align-items: center; grid-template-columns: auto 1fr; gap: 10px; }

.status-chip {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 999px;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.08);
  text-transform: capitalize;
}
.status-chip .pulse {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--success);
  box-shadow: 0 0 12px rgba(34, 197, 94, 0.6);
  animation: pulse 2s infinite;
}
.status-chip.offline .pulse { background: var(--danger); box-shadow: 0 0 12px rgba(239, 68, 68, 0.6); }

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.5; }
}

@media (max-width: 768px) {
  .topbar {
    flex-direction: column;
    align-items: flex-start;
    gap: 12px;
    padding: 12px 16px;
  }
  .actions { width: 100%; flex-direction: column; }
  .actions form { flex: 1; width: 100%; }
  .actions .button { width: 100%; }
  .hero { padding: 16px; }
}

@media (max-width: 480px) {
  .brand h1 { font-size: 20px; }
  .brand p { font-size: 13px; }
  input, select { font-size: 13px; padding: 9px; }
}
"#;

// ---------------------------------------------------------------------------
// Shared layout helpers
// ---------------------------------------------------------------------------

fn html_head(title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en" class="dark">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{title} | Turbo Gravity</title>
  <link rel="stylesheet" href="/styles.css" />
</head>
<body>
  <div class="backdrop"></div>
"#
    )
}

fn html_foot() -> &'static str {
    "</body>\n</html>\n"
}

fn topbar(back_href: Option<&str>) -> String {
    let back_btn = match back_href {
        Some(href) => format!(r#"<a class="button ghost" href="{href}">&larr; Back</a>"#),
        None => String::new(),
    };
    format!(
        r#"  <div class="topbar">
    <div class="brand">
      <div class="dot"></div>
      <div>
        <h1>Turbo Gravity</h1>
        <p>Control Panel</p>
      </div>
    </div>
    <div class="topbar-actions">
      {back_btn}
    </div>
  </div>
"#
    )
}

// ---------------------------------------------------------------------------
// Dashboard page (was dashboard.ejs)
// ---------------------------------------------------------------------------

pub struct DashboardData {
    pub bot_status: &'static str,
    pub command_scope: String,
    pub guild_id: String,
    pub invite_link: String,
    pub invite_permissions: String,
}

pub fn dashboard_page(data: &DashboardData) -> String {
    let status_class = if data.bot_status == "online" { "" } else { "offline" };
    let status_help = if data.bot_status == "online" {
        "Bot is connected and ready"
    } else {
        "Bot is not connected"
    };

    let scope_guild_sel = if data.command_scope == "guild" { " selected" } else { "" };
    let scope_global_sel = if data.command_scope == "global" { " selected" } else { "" };

    format!(
        r#"{head}{topbar}
  <main class="page">
    <!-- Status card -->
    <div class="grid two">
      <div class="card">
        <div class="card-header">
          <div>
            <p class="eyebrow">Current Status</p>
            <h3>Bot Health</h3>
          </div>
        </div>
        <div style="margin-bottom:16px">
          <span class="status-chip {status_class}">
            <span class="pulse"></span>
            {bot_status_upper}
          </span>
        </div>
        <p style="color:var(--muted);font-size:13px">{status_help}</p>
      </div>

      <div class="card">
        <div class="card-header">
          <div>
            <p class="eyebrow">Quick Actions</p>
            <h3>Bot Control</h3>
          </div>
        </div>
        <div class="actions">
          <a class="button success" href="/control/start">Start</a>
          <a class="button danger" href="/control/stop">Stop</a>
          <a class="button primary" href="/control/restart">Restart</a>
        </div>
      </div>
    </div>

    <!-- Settings card -->
    <div class="card">
      <div class="card-header">
        <div>
          <p class="eyebrow">Bot Settings</p>
          <h3>Configuration</h3>
        </div>
      </div>
      <form class="form" method="POST" action="/control/config">
        <div class="dual">
          <label>Command Scope
            <select name="commandScope">
              <option value="guild"{scope_guild_sel}>Guild (Fastest)</option>
              <option value="global"{scope_global_sel}>Global</option>
            </select>
          </label>
          <label>Guild ID
            <input type="text" name="guildId" value="{guild_id}" placeholder="Optional" />
          </label>
        </div>
        <label>Invite Permissions
          <input type="text" name="invitePermissions" value="{invite_permissions}" />
        </label>
        <button class="button primary" type="submit">Save Settings</button>
      </form>
    </div>

    <!-- Invite card -->
    <div class="card">
      <div class="card-header">
        <div>
          <p class="eyebrow">Invite Link</p>
          <h3>Add Bot to Server</h3>
        </div>
      </div>
      <p style="color:var(--muted);font-size:13px">Share this link to invite the bot to your servers:</p>
      <div style="word-break:break-all;padding:12px;background:rgba(0,0,0,0.3);border-radius:6px;margin-top:12px;font-size:12px">
        <a href="{invite_link}" target="_blank" style="color:var(--primary)">{invite_link}</a>
      </div>
      <a class="button primary" href="{invite_link}" target="_blank" style="margin-top:12px">Open Invite Link</a>
    </div>
  </main>
{foot}"#,
        head = html_head("Dashboard"),
        topbar = topbar(None),
        status_class = status_class,
        bot_status_upper = data.bot_status.to_uppercase(),
        status_help = status_help,
        scope_guild_sel = scope_guild_sel,
        scope_global_sel = scope_global_sel,
        guild_id = html_escape(&data.guild_id),
        invite_permissions = html_escape(&data.invite_permissions),
        invite_link = html_escape(&data.invite_link),
        foot = html_foot(),
    )
}

// ---------------------------------------------------------------------------
// Setup page (was setup.ejs)
// ---------------------------------------------------------------------------

pub struct SetupData {
    pub owner_id: String,
}

pub fn setup_page(data: &SetupData) -> String {
    format!(
        r#"{head}
  <style>
    .setup-page {{
      display: flex;
      align-items: center;
      justify-content: center;
      min-height: 100vh;
      padding: 24px;
    }}
    .setup-card {{
      width: 100%;
      max-width: 720px;
      background: var(--panel);
      border: 1px solid rgba(255,255,255,0.08);
      border-radius: 20px;
      padding: 32px;
      box-shadow: var(--shadow);
    }}
    .setup-card .brand {{ margin-bottom: 24px; }}
    .setup-form {{ gap: 24px; }}
    .form-section {{
      display: grid;
      gap: 12px;
      padding: 16px;
      background: var(--panel-strong);
      border: 1px solid rgba(255,255,255,0.06);
      border-radius: 14px;
    }}
    .form-section h3 {{ margin: 0 0 8px; font-size: 16px; }}
  </style>
  <main class="page setup-page">
    <section class="setup-card">
      <div class="brand">
        <div class="dot"></div>
        <div>
          <h1>Turbo Gravity Setup</h1>
          <p>Configure your Discord bot to get started</p>
        </div>
      </div>

      <form class="form setup-form" method="POST" action="/setup">
        <div class="form-section">
          <h3>Discord Bot Credentials</h3>
          <label>Bot Token
            <input type="password" name="botToken" placeholder="Your Discord bot token" required />
          </label>
          <div class="dual">
            <label>Client ID
              <input type="text" name="clientId" placeholder="Application client ID" required />
            </label>
            <label>Client Secret
              <input type="password" name="clientSecret" placeholder="OAuth2 client secret" required />
            </label>
          </div>
          <label>OAuth Callback URL
            <input type="url" name="callbackUrl" value="http://localhost:8080/auth/discord/callback" required />
          </label>
        </div>

        <div class="form-section">
          <h3>Database &amp; Security (MongoDB Optional)</h3>
          <label>MongoDB URI (optional)
            <input type="text" name="mongoUri" placeholder="mongodb://localhost:27017/turbogravity" />
          </label>
          <label>Session Secret
            <input type="password" name="sessionSecret" placeholder="Random secure string" required />
          </label>
          <label>Admin User IDs (comma-separated)
            <input type="text" name="adminIds" placeholder="123456789,987654321" value="{owner_id}" required />
          </label>
        </div>

        <div class="form-section">
          <h3>Bot Settings</h3>
          <div class="dual">
            <label>Guild ID (optional)
              <input type="text" name="guildId" placeholder="For guild-only commands" />
            </label>
            <label>Port
              <input type="number" name="port" value="8080" required />
            </label>
          </div>
          <div class="dual">
            <label>Presence Type
              <select name="presenceType">
                <option value="0">Playing</option>
                <option value="2">Listening</option>
                <option value="3">Watching</option>
                <option value="5">Competing</option>
              </select>
            </label>
            <label>Command Scope
              <select name="commandScope">
                <option value="guild">Guild</option>
                <option value="global">Global</option>
              </select>
            </label>
          </div>
          <label>Presence Text
            <input type="text" name="presenceText" value="Ready to serve" />
          </label>
          <label>Invite Permissions
            <input type="text" name="invitePermissions" value="8" />
          </label>
          <label class="checkbox">
            <input type="checkbox" name="autoStart" />
            <span>Auto-start bot on launch</span>
          </label>
        </div>

        <button class="button primary" type="submit">Save &amp; Start</button>
      </form>
    </section>
  </main>
{foot}"#,
        head = html_head("Setup"),
        owner_id = html_escape(&data.owner_id),
        foot = html_foot(),
    )
}

// ---------------------------------------------------------------------------
// Guild selector page (was selector.ejs)
// ---------------------------------------------------------------------------

pub struct GuildInfo {
    pub id: String,
    pub name: String,
    pub icon: Option<String>,
    pub member_count: Option<u64>,
}

pub struct SelectorData {
    pub username: String,
    pub guilds: Vec<GuildInfo>,
    pub bot_status: &'static str,
}

pub fn selector_page(data: &SelectorData) -> String {
    let status_class = if data.bot_status == "online" {
        "status-online"
    } else {
        "status-offline"
    };

    let guild_cards: String = data
        .guilds
        .iter()
        .map(|g| {
            let icon_html = match &g.icon {
                Some(icon) => format!(
                    r#"<img src="https://cdn.discordapp.com/icons/{id}/{icon}.png?size=256" alt="{name}" class="guild-icon">"#,
                    id = html_escape(&g.id),
                    icon = html_escape(icon),
                    name = html_escape(&g.name),
                ),
                None => {
                    let initials: String = g
                        .name
                        .split_whitespace()
                        .filter_map(|w| w.chars().next())
                        .map(|c| c.to_uppercase().to_string())
                        .take(2)
                        .collect();
                    format!(r#"<div class="guild-placeholder">{initials}</div>"#)
                }
            };
            let members = g
                .member_count
                .map(|n| n.to_string())
                .unwrap_or_else(|| "?".into());
            format!(
                r#"<div class="guild-card">
          <div class="guild-icon-wrapper">{icon_html}</div>
          <div class="guild-info">
            <div class="guild-name">{name}</div>
            <div class="guild-stats"><div class="stat">👥 {members} Members</div></div>
            <div class="guild-action">
              <a href="/manage/{id}" class="btn btn-manage">Manage</a>
            </div>
          </div>
        </div>"#,
                icon_html = icon_html,
                name = html_escape(&g.name),
                members = members,
                id = html_escape(&g.id),
            )
        })
        .collect();

    let content = if data.guilds.is_empty() {
        r#"<div class="empty-state">
          <div class="empty-state-icon">🔍</div>
          <h2>No Servers Found</h2>
          <p>You don't have permission to manage any servers yet.</p>
        </div>"#
            .to_string()
    } else {
        format!(r#"<div class="guilds-grid">{guild_cards}</div>"#)
    };

    format!(
        r#"{head}
  <style>
    body {{
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      min-height: 100vh;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      display: flex;
      flex-direction: column;
      padding: 20px;
    }}
    .navbar {{
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 20px 40px;
      background: rgba(0,0,0,0.3);
      border-radius: 12px;
      margin-bottom: 40px;
      backdrop-filter: blur(10px);
    }}
    .navbar-brand {{ color: white; font-size: 24px; font-weight: bold; text-decoration: none; }}
    .navbar-right {{ display: flex; align-items: center; gap: 20px; }}
    .user-name {{ color: white; font-weight: 500; }}
    .logout-btn {{
      padding: 10px 20px;
      background: rgba(255,59,48,0.8);
      color: white;
      border: none;
      border-radius: 8px;
      cursor: pointer;
      font-weight: 600;
      transition: all 0.3s ease;
      text-decoration: none;
      display: inline-block;
    }}
    .logout-btn:hover {{ background: rgba(255,59,48,1); transform: translateY(-2px); }}
    .container {{ max-width: 1200px; margin: 0 auto; flex: 1; }}
    .header {{ text-align: center; color: white; margin-bottom: 40px; }}
    .header h1 {{ font-size: 36px; margin-bottom: 10px; }}
    .header p {{ font-size: 16px; opacity: 0.9; }}
    .guilds-grid {{
      display: grid;
      grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
      gap: 20px;
      margin-bottom: 20px;
    }}
    .guild-card {{
      background: rgba(255,255,255,0.95);
      border-radius: 12px;
      overflow: hidden;
      transition: all 0.3s ease;
      box-shadow: 0 10px 30px rgba(0,0,0,0.2);
    }}
    .guild-card:hover {{ transform: translateY(-8px); box-shadow: 0 15px 40px rgba(0,0,0,0.3); }}
    .guild-icon-wrapper {{
      position: relative;
      width: 100%;
      padding-top: 100%;
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      overflow: hidden;
    }}
    .guild-icon {{ position: absolute; top: 0; left: 0; width: 100%; height: 100%; object-fit: cover; }}
    .guild-placeholder {{
      position: absolute;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      font-size: 32px;
      color: white;
      font-weight: bold;
    }}
    .guild-info {{ padding: 20px; }}
    .guild-name {{ font-size: 18px; font-weight: 600; color: #333; margin-bottom: 10px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }}
    .guild-stats {{ display: flex; gap: 15px; font-size: 14px; color: #666; margin-bottom: 15px; }}
    .guild-action {{ display: flex; gap: 10px; }}
    .btn {{ flex: 1; padding: 10px; border: none; border-radius: 8px; font-weight: 600; cursor: pointer; text-decoration: none; text-align: center; transition: all 0.3s ease; font-size: 14px; }}
    .btn-manage {{ background: #667eea; color: white; }}
    .btn-manage:hover {{ background: #5568d3; }}
    .empty-state {{ text-align: center; padding: 60px 20px; color: white; }}
    .empty-state-icon {{ font-size: 64px; margin-bottom: 20px; }}
    .empty-state h2 {{ font-size: 28px; margin-bottom: 10px; }}
    .empty-state p {{ font-size: 16px; opacity: 0.9; margin-bottom: 30px; }}
    .status-badge {{
      display: inline-block;
      padding: 6px 12px;
      border-radius: 20px;
      font-size: 12px;
      font-weight: 600;
      margin-bottom: 15px;
    }}
    .status-online {{ background: rgba(34,197,94,0.2); color: #22c55e; }}
    .status-offline {{ background: rgba(107,114,128,0.2); color: #6b7280; }}
  </style>
  <div class="navbar">
    <div class="navbar-brand">&#128640; Turbo Gravity</div>
    <div class="navbar-right">
      <span class="user-name">{username}</span>
      <a href="/logout" class="logout-btn">Logout</a>
    </div>
  </div>

  <div class="container">
    <div class="header">
      <h1>Select a Guild</h1>
      <p>Choose a server to manage</p>
      <div class="status-badge {status_class}">
        Bot Status: {bot_status_upper}
      </div>
    </div>
    {content}
  </div>
{foot}"#,
        head = html_head("Guild Selector"),
        username = html_escape(&data.username),
        status_class = status_class,
        bot_status_upper = data.bot_status.to_uppercase(),
        content = content,
        foot = html_foot(),
    )
}

// ---------------------------------------------------------------------------
// Error page (was error.ejs)
// ---------------------------------------------------------------------------

pub struct ErrorData {
    pub code: u16,
    pub title: String,
    pub message: String,
}

pub fn error_page(data: &ErrorData) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <title>{code} - Turbo Gravity</title>
  <style>
    * {{ margin: 0; padding: 0; box-sizing: border-box; }}
    body {{
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      min-height: 100vh;
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      display: flex;
      align-items: center;
      justify-content: center;
      padding: 20px;
    }}
    .error-container {{
      text-align: center;
      background: rgba(255,255,255,0.95);
      border-radius: 12px;
      padding: 60px 40px;
      box-shadow: 0 20px 60px rgba(0,0,0,0.3);
      max-width: 500px;
      width: 100%;
    }}
    .error-code {{ font-size: 72px; font-weight: 700; color: #667eea; margin-bottom: 20px; }}
    .error-title {{ font-size: 24px; color: #333; margin-bottom: 10px; font-weight: 600; }}
    .error-message {{ font-size: 16px; color: #666; margin-bottom: 30px; line-height: 1.6; }}
    .error-actions {{ display: flex; gap: 10px; justify-content: center; flex-wrap: wrap; }}
    .btn {{ padding: 12px 24px; border: none; border-radius: 8px; font-weight: 600; cursor: pointer; text-decoration: none; transition: all 0.3s ease; font-size: 14px; }}
    .btn-primary {{ background: #667eea; color: white; }}
    .btn-primary:hover {{ background: #5568d3; transform: translateY(-2px); }}
    .btn-secondary {{ background: #f3f4f6; color: #333; }}
    .btn-secondary:hover {{ background: #e5e7eb; transform: translateY(-2px); }}
  </style>
</head>
<body>
  <div class="error-container">
    <div class="error-code">{code}</div>
    <div class="error-title">{title}</div>
    <div class="error-message">{message}</div>
    <div class="error-actions">
      <a href="/selector" class="btn btn-primary">Back to Servers</a>
      <a href="/" class="btn btn-secondary">Home</a>
    </div>
  </div>
</body>
</html>
"#,
        code = data.code,
        title = html_escape(&data.title),
        message = html_escape(&data.message),
    )
}

// ---------------------------------------------------------------------------
// Utility: minimal HTML escaping to prevent XSS in dynamic values
// ---------------------------------------------------------------------------

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
