# Turbo Gravity

A powerful, feature-rich Discord bot with an **integrated web-based Control Panel**. Built with Node.js, discord.js v14, Express, and MongoDB.

## Features

### Web Dashboard
- **Zero-config Setup Wizard** - Configure everything through the browser
- **OAuth2 Authentication** - Secure Discord login for admins
- **Real-time Bot Control** - Start, stop, and restart the bot without CLI
- **Dynamic Configuration** - Update credentials, presence, and settings on-the-fly
- **Dark/Light Mode** - Beautiful glassmorphic UI with theme toggle

### Technical Highlights
- **Independent Architecture** - Dashboard runs independently of bot lifecycle
- **Persistent Configuration** - Settings stored in MongoDB + local file fallback
- **Hot Reload** - Update bot settings without restarting the dashboard
- **Secure Credentials** - Write-only password fields, no secret exposure
- **Docker Ready** - Dockerfile and .dockerignore included

---

## Quick Start

### Prerequisites
- Node.js 18+ 
- MongoDB (local or hosted)
- Discord Application ([Discord Developer Portal](https://discord.com/developers/applications))

### Installation

1. **Clone the repository**
   ```bash
   git clone https://github.com/TurboRx/Turbo-Gravity.git
   cd Turbo-Gravity-
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Start the application**
   ```bash
   npm start
   ```

4. **Open the setup wizard**
   Navigate to `http://localhost:3000/setup` in your browser.

---

## Configuration

### Setup Wizard (Recommended)

The first time you run the app, you'll be redirected to `/setup` where you can configure:

#### Discord Bot Credentials
- **Bot Token** - From your Discord application
- **Client ID** - Your application ID
- **Client Secret** - OAuth2 client secret
- **OAuth Callback URL** - Default: `http://localhost:3000/auth/discord/callback`

#### Database & Security
- **MongoDB URI** - Connection string (e.g., `mongodb://localhost:27017/turbogravity`)
- **Session Secret** - Random secure string for session encryption
- **Admin User IDs** - Comma-separated Discord user IDs with dashboard access

#### Bot Settings
- **Guild ID** - Optional, for guild-only command registration
- **Port** - Dashboard port (default: 3000)
- **Presence Type** - Playing, Listening, Watching, Competing
- **Presence Text** - Default bot status text
- **Command Scope** - Guild (instant) or Global (up to 1 hour)
- **Invite Permissions** - Permission integer for invite link
- **Auto-start** - Whether to start the bot on app launch

### Manual Configuration (Optional)

Copy `.env.example` to `.env` and fill values (settings will still be imported into the setup wizard).

---

## Usage

### Dashboard Access
1. Visit `http://localhost:3000`
2. Login with Discord OAuth
3. Only configured admin users can access controls

### Control Panel Features
- **Lifecycle Controls** - Start/Stop/Restart buttons
- **Status Manager** - Update bot presence in real-time
- **Configuration Editor** - Modify all settings including credentials
- **Invite Link** - One-click bot invite with configured permissions

### Bot Commands
All commands are slash commands (`/`). Once the bot is running, they'll be registered automatically based on your scope setting (guild or global).

Example: `/kick @user reason:spamming`

---

## Docker Deployment

### Build Image
```bash
docker build -t turbo-gravity .
```

### Run Container
```bash
docker run -d \
  -p 3000:3000 \
  --name turbo-gravity \
  turbo-gravity
```

Then visit `http://localhost:3000/setup` to configure via the web interface.

---

## Development

### Run with Auto-restart
```bash
npm run dev
```

### Add a New Command
1. Create a file in `src/commands/<category>/commandname.js`
2. Export an object with `data` (SlashCommandBuilder) and `execute` function
3. Restart the bot or trigger a reload from the dashboard

Example:
```javascript
import { SlashCommandBuilder } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('hello')
    .setDescription('Says hello'),
  async execute(interaction) {
    await interaction.reply('Hello!');
  }
};
```

---

## Security Notes

- Admin IDs are checked on every protected route
- Secrets (tokens, passwords) are write-only in the dashboard
- Session secret should be a strong random string
- MongoDB URI should use authentication in production
- Use HTTPS in production (reverse proxy like nginx/caddy)

---

## License

[MIT](LICENSE)

---

## Contributing

Pull requests are welcome! For major changes, please open an issue first to discuss what you'd like to change.

---

## Support

For issues or questions, open an issue on [GitHub](https://github.com/TurboRx/Turbo-Gravity-/issues).
