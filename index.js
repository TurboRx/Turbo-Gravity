import express from 'express';
import session from 'express-session';
import MongoStore from 'connect-mongo';
import dotenv from 'dotenv';
import path from 'path';
import { fileURLToPath } from 'url';
import mongoose from 'mongoose';
import crypto from 'crypto';
import BotManager from './src/BotManager.js';
import Config from './src/models/Config.js';
import { loadLocalConfig, saveLocalConfig, isConfigured } from './src/localConfig.js';

dotenv.config();

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

let localConfig = await loadLocalConfig();
let configured = isConfigured(localConfig);
let adminIds = (localConfig.adminIds || '')
  .split(',')
  .map(id => id.trim())
  .filter(id => id.length > 0);

// Generate a strong session secret if not configured
if (!localConfig.sessionSecret || localConfig.sessionSecret === 'temp-secret') {
  localConfig.sessionSecret = crypto.randomBytes(32).toString('hex');
}

const botManager = new BotManager({
  token: localConfig.botToken,
  clientId: localConfig.clientId,
  guildId: localConfig.guildId,
  mongoUri: localConfig.mongoUri,
  defaultPresence: {
    activities: [{ name: localConfig.presenceText, type: localConfig.presenceType }],
    status: 'online'
  }
});

let cachedConfig = null;

const getDiscordToken = async (code, config) => {
  const params = new URLSearchParams();
  params.append('client_id', config.clientId);
  params.append('client_secret', config.clientSecret);
  params.append('grant_type', 'authorization_code');
  params.append('code', code);
  params.append('redirect_uri', config.callbackUrl);

  const response = await fetch('https://discord.com/api/oauth2/token', {
    method: 'POST',
    body: params,
    headers: { 'Content-Type': 'application/x-www-form-urlencoded' }
  });

  if (!response.ok) {
    throw new Error(`Discord OAuth error: ${response.status}`);
  }

  return response.json();
};

const getDiscordUser = async accessToken => {
  const response = await fetch('https://discord.com/api/users/@me', {
    headers: { Authorization: `Bearer ${accessToken}` }
  });

  if (!response.ok) {
    throw new Error(`Discord API error: ${response.status}`);
  }

  return response.json();
};

const configureOAuth = config => {
  // OAuth configuration is validated at runtime
  if (!config?.clientId || !config?.clientSecret) return false;
  return true;
};

if (configured) configureOAuth(localConfig);

const app = express();
app.set('view engine', 'ejs');
app.set('views', path.join(__dirname, 'src', 'dashboard', 'views'));
app.use('/public', express.static(path.join(__dirname, 'src', 'dashboard', 'public')));
app.use(express.urlencoded({ extended: false }));
app.use(express.json());

// Configure session store - use MongoStore if MongoDB is available, otherwise MemoryStore with warning suppression
const sessionConfig = {
  secret: localConfig.sessionSecret || 'temp-secret',
  resave: false,
  saveUninitialized: false,
  cookie: {
    maxAge: 24 * 60 * 60 * 1000, // 24 hours
    secure: process.env.NODE_ENV === 'production', // Use secure cookies in production
    httpOnly: true
  }
};

// Use MongoStore for production-ready session storage if MongoDB is configured
if (localConfig.mongoUri) {
  sessionConfig.store = MongoStore.create({
    mongoUrl: localConfig.mongoUri,
    touchAfter: 24 * 3600, // Lazy session update (in seconds)
    crypto: {
      secret: localConfig.sessionSecret
    }
  });
}

app.use(session(sessionConfig));

const ensureConfigured = (req, res, next) => {
  if (!configured) return res.redirect('/setup');
  next();
};

const ensureAuth = (req, res, next) => {
  if (!configured) return res.redirect('/setup');
  if (req.session?.user) return next();
  res.redirect('/auth/discord');
};

const ensureAdmin = (req, res, next) => {
  if (!configured) return res.redirect('/setup');
  if (!req.session?.user) return res.redirect('/auth/discord');
  if (adminIds.includes(req.session.user.id)) return next();
  return res.status(403).send('<h1>403 Forbidden</h1><p>You do not have admin permissions to access this page.</p>');
};

const connectMongo = async () => {
  if (!localConfig.mongoUri) return null;
  if (mongoose.connection.readyState === 0) {
    await mongoose.connect(localConfig.mongoUri);
  }
};

const getConfig = async () => {
  if (!localConfig.mongoUri) return configToView({});
  let config = await Config.findOne();
  if (!config) {
    config = await Config.create({
      botToken: localConfig.botToken,
      clientId: localConfig.clientId,
      clientSecret: localConfig.clientSecret,
      callbackUrl: localConfig.callbackUrl,
      guildId: localConfig.guildId,
      autoStart: localConfig.autoStart,
      presenceText: localConfig.presenceText,
      presenceType: localConfig.presenceType,
      commandScope: localConfig.commandScope,
      invitePermissions: localConfig.invitePermissions
    });
  }
  cachedConfig = config;
  return config;
};

const configToView = config => ({
  autoStart: config?.autoStart || false,
  presenceText: config?.presenceText || 'Ready to serve',
  presenceType: config?.presenceType ?? 0,
  commandScope: config?.commandScope || 'guild',
  guildId: config?.guildId || '',
  invitePermissions: config?.invitePermissions || '8'
});

app.get('/setup', (req, res) => {
  if (configured) return res.redirect('/');
  res.render('setup', { ownerId: req.query.ownerId || '' });
});

app.post('/setup', async (req, res) => {
  try {
    // Validate required fields (MongoDB is now optional)
    if (!req.body.botToken || !req.body.clientId || !req.body.clientSecret || !req.body.sessionSecret) {
      return res.status(400).send('Missing required fields. Please fill in: Bot Token, Client ID, Client Secret, and Session Secret.');
    }

    // Validate and sanitize inputs
    const botToken = String(req.body.botToken).trim();
    const clientId = String(req.body.clientId).trim();
    const clientSecret = String(req.body.clientSecret).trim();
    const sessionSecret = String(req.body.sessionSecret).trim();
    const callbackUrl = String(req.body.callbackUrl || 'http://localhost:8080/auth/discord/callback').trim();
    const guildId = String(req.body.guildId || '').trim();
    const mongoUri = String(req.body.mongoUri || '').trim();
    const adminIds = String(req.body.adminIds || '').trim();
    const port = Number.parseInt(req.body.port, 10);
    const presenceText = String(req.body.presenceText || 'Ready to serve').trim();
    const presenceType = Number.parseInt(req.body.presenceType, 10);
    const commandScope = String(req.body.commandScope || 'guild').trim();
    const invitePermissions = String(req.body.invitePermissions || '8').trim();

    // Validate field lengths and formats
    if (botToken.length < 50) {
      return res.status(400).send('Bot Token appears to be invalid (too short).');
    }
    if (clientId.length < 10 || !/^\d+$/.test(clientId)) {
      return res.status(400).send('Client ID must be a numeric Discord application ID.');
    }
    if (sessionSecret.length < 32) {
      return res.status(400).send('Session Secret must be at least 32 characters for security.');
    }
    if (port && (port < 1 || port > 65535)) {
      return res.status(400).send('Port must be between 1 and 65535.');
    }
    if (commandScope !== 'global' && commandScope !== 'guild') {
      return res.status(400).send('Command scope must be either "global" or "guild".');
    }
    if (guildId && !/^\d+$/.test(guildId)) {
      return res.status(400).send('Guild ID must be a numeric Discord server ID.');
    }
    if (mongoUri && !mongoUri.startsWith('mongodb://') && !mongoUri.startsWith('mongodb+srv://')) {
      return res.status(400).send('MongoDB URI must start with mongodb:// or mongodb+srv://');
    }

    const newConfig = {
      botToken,
      clientId,
      clientSecret,
      callbackUrl,
      guildId,
      mongoUri,
      sessionSecret,
      adminIds,
      port: port || 8080,
      autoStart: req.body.autoStart === 'on',
      presenceText,
      presenceType: presenceType || 0,
      commandScope,
      invitePermissions
    };

    await saveLocalConfig(newConfig);
    localConfig = newConfig;
    configured = isConfigured(localConfig);
    adminIds = (localConfig.adminIds || '')
      .split(',')
      .map(id => id.trim())
      .filter(id => id.length > 0);

    const PORT = process.env.PORT || newConfig.port || 8080;
    if (!app.listening) {
      app.listen(PORT, () => {
        console.log(`Dashboard running on port ${PORT}`);
      });
    }

    configureOAuth(localConfig);
    botManager.token = localConfig.botToken;
    botManager.clientId = localConfig.clientId;
    botManager.guildId = localConfig.guildId;
    botManager.mongoUri = localConfig.mongoUri;
    botManager.commandScope = localConfig.commandScope;
    botManager.invitePermissions = localConfig.invitePermissions;
    botManager.defaultPresence = {
      activities: [{ name: localConfig.presenceText, type: localConfig.presenceType }],
      status: 'online'
    };

    if (localConfig.mongoUri) {
      await connectMongo();
      await getConfig();
    }

    res.send(`
      <html>
        <head>
          <title>Setup Complete</title>
          <style>
            body { font-family: system-ui; display: flex; align-items: center; justify-content: center; min-height: 100vh; background: #050b18; color: #e5e9f0; }
            .box { text-align: center; padding: 40px; background: rgba(255,255,255,0.04); border-radius: 16px; border: 1px solid rgba(255,255,255,0.08); }
            h1 { margin: 0 0 16px; }
            p { margin: 0 0 24px; color: #8aa0b5; }
            a { display: inline-block; padding: 12px 24px; background: #0ea5e9; color: white; text-decoration: none; border-radius: 12px; }
          </style>
        </head>
        <body>
          <div class="box">
            <h1>✅ Setup Complete</h1>
            <p>Your bot configuration has been saved. Click below to access the control panel.</p>
            <a href="/">Go to Dashboard</a>
          </div>
        </body>
      </html>
    `);
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error(err);
    res.status(500).send(`Setup failed: ${err.message}`);
  }
});

app.get('/', ensureAuth, async (req, res) => {
  res.redirect('/selector');
});

app.get('/selector', ensureAuth, async (req, res) => {
  const config = cachedConfig || await getConfig();
  res.render('selector', {
    user: req.session.user,
    botStatus: botManager.client ? 'online' : 'offline'
  });
});

app.get('/auth/discord', (req, res) => {
  const config = localConfig;
  
  // Setup mode requires configuration first
  if (req.query.setup) {
    return res.redirect('/setup?message=Please complete setup first, then you can login to get your Discord User ID.');
  }

  // Normal login requires configuration
  if (!config?.clientId) {
    return res.status(500).send('OAuth not configured. Please complete setup first.');
  }

  const scopes = ['identify', 'guilds'];
  const baseUrl = 'https://discord.com/api/oauth2/authorize';
  const params = new URLSearchParams({
    client_id: config.clientId,
    redirect_uri: config.callbackUrl,
    response_type: 'code',
    scope: scopes.join(' ')
  });

  res.redirect(`${baseUrl}?${params.toString()}`);
});

app.get('/auth/discord/callback', async (req, res) => {
  const code = req.query.code;
  
  if (!code) {
    return res.redirect('/');
  }

  try {
    const config = localConfig;
    if (!config?.clientId || !config?.clientSecret) {
      return res.status(500).send('OAuth not configured. Please complete setup first.');
    }
    
    const tokenData = await getDiscordToken(code, config);
    if (!tokenData.access_token) {
      throw new Error('No access token received');
    }

    const userInfo = await getDiscordUser(tokenData.access_token);
    
    // Fetch user guilds
    const guildsResponse = await fetch('https://discord.com/api/users/@me/guilds', {
      headers: { Authorization: `Bearer ${tokenData.access_token}` }
    });
    
    if (!guildsResponse.ok) {
      throw new Error(`Failed to fetch guilds: ${guildsResponse.status}`);
    }
    
    const guilds = await guildsResponse.json();

    // Filter guilds where user is admin (has ManageGuild permission)
    const adminGuilds = guilds.filter(guild => {
      if (!guild.permissions) return false;
      const permissions = BigInt(guild.permissions);
      // Check for ManageGuild permission (0x00000020 = 32)
      return (permissions & BigInt(32)) === BigInt(32);
    });

    req.session.user = {
      ...userInfo,
      guilds: guilds,
      adminGuilds: adminGuilds,
      displayedGuilds: adminGuilds
    };
    
    res.redirect('/selector');
  } catch (err) {
    console.error('OAuth error:', err);
    res.status(500).send(`Authentication failed: ${err.message}. Please try again or check your OAuth configuration.`);
  }
});

app.get('/logout', (req, res) => {
  req.session.destroy(err => {
    if (err) {
      return res.status(500).send('Failed to logout');
    }
    res.redirect('/setup');
  });
});

app.get('/manage/:guildId', ensureAuth, async (req, res) => {
  const { guildId } = req.params;
  
  // Check if user has admin access to this guild
  const guild = req.session.user.displayedGuilds?.find(g => g.id === guildId);
  if (!guild) {
    return res.status(403).render('error', {
      error: 'Access Denied',
      message: 'You do not have permission to manage this guild.',
      user: req.session.user
    });
  }

  const config = cachedConfig || await getConfig();
  const inviteLink = botManager.getInviteLink({ permissions: config.invitePermissions });
  
  res.render('dashboard', {
    user: req.session.user,
    guild: guild,
    inviteLink,
    botStatus: botManager.client ? 'online' : 'offline',
    config: configToView(config)
  });
});

app.post('/control/start', ensureAdmin, async (req, res) => {
  try {
    await botManager.start();
    res.redirect('/');
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error(err);
    res.status(500).send('Failed to start bot');
  }
});

app.post('/control/stop', ensureAdmin, async (req, res) => {
  try {
    await botManager.stop();
    res.redirect('/');
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error(err);
    res.status(500).send('Failed to stop bot');
  }
});

app.post('/control/restart', ensureAdmin, async (req, res) => {
  try {
    await botManager.restart();
    res.redirect('/');
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error(err);
    res.status(500).send('Failed to restart bot');
  }
});

app.post('/control/status', ensureAdmin, async (req, res) => {
  const { activityType, statusText } = req.body;
  
  if (!statusText || statusText.trim().length === 0) {
    return res.status(400).send('Status text cannot be empty');
  }
  
  try {
    await botManager.setActivity({ type: activityType, text: statusText });
    res.redirect('/');
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error(err);
    res.status(500).send('Failed to update status');
  }
});

app.get('/control/invite', ensureAuth, async (req, res) => {
  const config = cachedConfig || await getConfig();
  const link = botManager.getInviteLink({ permissions: config.invitePermissions });
  res.redirect(link);
});

app.get('/control/config', ensureAdmin, async (req, res) => {
  const config = cachedConfig || await getConfig();
  res.render('dashboard', {
    user: req.session.user,
    inviteLink: botManager.getInviteLink({ permissions: config.invitePermissions }),
    botStatus: botManager.client ? 'online' : 'offline',
    config: configToView(config)
  });
});

app.post('/control/config', ensureAdmin, async (req, res) => {
  try {
    const config = cachedConfig || await getConfig();
    const presenceType = Number.parseInt(req.body.presenceType, 10);
    const commandScope = ['global', 'guild'].includes(req.body.commandScope)
      ? req.body.commandScope
      : config.commandScope;

    const previousToken = config.botToken;
    const previousClientId = config.clientId;
    const previousScope = config.commandScope;

    config.autoStart = req.body.autoStart === 'on';
    config.presenceText = req.body.presenceText || config.presenceText;
    config.presenceType = Number.isInteger(presenceType) ? presenceType : config.presenceType;
    config.commandScope = commandScope;
    config.guildId = req.body.guildId?.trim() || '';
    config.invitePermissions = req.body.invitePermissions || config.invitePermissions;

    if (req.body.botToken) config.botToken = req.body.botToken;
    if (req.body.clientId) config.clientId = req.body.clientId;
    if (req.body.clientSecret) config.clientSecret = req.body.clientSecret;
    if (req.body.callbackUrl) config.callbackUrl = req.body.callbackUrl;

    await config.save();
    cachedConfig = config;

    botManager.applyConfig(config);

    const needsRestart =
      botManager.client &&
      ((req.body.botToken && req.body.botToken !== previousToken) ||
        (req.body.clientId && req.body.clientId !== previousClientId) ||
        commandScope !== previousScope);

    if (botManager.client) {
      await botManager.setActivity({ type: config.presenceType, text: config.presenceText });
      if (needsRestart) await botManager.restart();
    }

    res.redirect('/');
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error(err);
    res.status(500).send('Failed to update configuration');
  }
});

app.post('/control/profile', ensureAdmin, async (req, res) => {
  try {
    if (botManager.client) {
      const { username, avatar, banner, bio } = req.body;
      await botManager.updateBotProfile({ username, avatar, banner, bio });
    }
    res.redirect('/');
  } catch (err) {
    // eslint-disable-next-line no-console
    console.error(err);
    res.status(500).send('Failed to update bot profile');
  }
});

const bootstrap = async () => {
  if (configured) {
    configureOAuth(localConfig);
    if (localConfig.mongoUri) {
      try {
        await connectMongo();
        await getConfig();
      } catch (err) {
        // eslint-disable-next-line no-console
        console.warn('Mongo connection failed, using local config only:', err.message);
      }
    }
  }

  app.listen(localConfig.port || 8080, () => {
    // eslint-disable-next-line no-console
    console.log(`Dashboard running on port ${localConfig.port || 8080}`);
    if (!configured) {
      // eslint-disable-next-line no-console
      console.log(`Setup required: http://localhost:${localConfig.port || 8080}/setup`);
    }
  });

  if (configured && localConfig.autoStart) {
    botManager
      .start()
      .catch(err => console.error('Bot failed to start:', err));
  }
};

bootstrap().catch(err => {
  // eslint-disable-next-line no-console
  console.error('Failed to bootstrap app:', err);
});
