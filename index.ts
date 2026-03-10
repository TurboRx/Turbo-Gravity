import express, { Request, Response, NextFunction } from 'express';
import session from 'express-session';
import MongoStore from 'connect-mongo';
import helmet from 'helmet';
import rateLimit from 'express-rate-limit';
import dotenv from 'dotenv';
import path from 'path';
import { fileURLToPath } from 'url';
import mongoose from 'mongoose';
import crypto from 'crypto';
import { doubleCsrf } from 'csrf-csrf';
import cookieParser from 'cookie-parser';
import BotManager from './src/BotManager.js';
import Config, { IConfig } from './src/models/Config.js';
import { loadLocalConfig, saveLocalConfig, isConfigured, LocalConfig } from './src/localConfig.js';

dotenv.config();

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

let localConfig: LocalConfig = await loadLocalConfig();
let configured: boolean = isConfigured(localConfig);
let adminIds: string[] = (localConfig.adminIds || '')
  .split(',')
  .map(id => id.trim())
  .filter(id => id.length > 0);

if (!localConfig.sessionSecret || localConfig.sessionSecret === 'temp-secret') {
  localConfig.sessionSecret = crypto.randomBytes(32).toString('hex');
}

const {
  doubleCsrfProtection,
  generateCsrfToken
} = doubleCsrf({
  getSecret: () => localConfig.sessionSecret,
  cookieName: 'x-csrf-token',
  cookieOptions: {
    sameSite: 'lax' as const,
    secure: process.env.NODE_ENV === 'production',
    httpOnly: true
  },
  size: 64,
  ignoredMethods: ['GET', 'HEAD', 'OPTIONS'],
  getCsrfTokenFromRequest: (req: Request) => (req.body as Record<string, string>)?._csrf || req.headers['x-csrf-token'] as string,
  getSessionIdentifier: (req: Request) => (req.session as session.Session & { id?: string })?.id || 'anonymous',
});

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

let cachedConfig: IConfig | null = null;

interface DiscordTokenResponse {
  access_token?: string;
  token_type?: string;
  expires_in?: number;
  refresh_token?: string;
  scope?: string;
}

interface DiscordUser {
  id: string;
  username: string;
  discriminator?: string;
  avatar?: string | null;
  guilds?: DiscordGuild[];
  adminGuilds?: DiscordGuild[];
  displayedGuilds?: DiscordGuild[];
}

interface DiscordGuild {
  id: string;
  name: string;
  icon?: string | null;
  owner?: boolean;
  permissions?: string;
  member_count?: number;
}

declare module 'express-session' {
  interface SessionData {
    user?: DiscordUser;
  }
}

const getDiscordToken = async (code: string, config: LocalConfig): Promise<DiscordTokenResponse> => {
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

  return response.json() as Promise<DiscordTokenResponse>;
};

const getDiscordUser = async (accessToken: string): Promise<DiscordUser> => {
  const response = await fetch('https://discord.com/api/users/@me', {
    headers: { Authorization: `Bearer ${accessToken}` }
  });

  if (!response.ok) {
    throw new Error(`Discord API error: ${response.status}`);
  }

  return response.json() as Promise<DiscordUser>;
};

const configureOAuth = (config: LocalConfig): boolean => {
  if (!config?.clientId || !config?.clientSecret) return false;
  return true;
};

if (configured) configureOAuth(localConfig);

const app = express();

app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'", "https://fonts.googleapis.com"],
      fontSrc: ["'self'", "https://fonts.gstatic.com"],
      scriptSrc: ["'self'", "'unsafe-inline'"],
      imgSrc: ["'self'", "data:", "https://cdn.discordapp.com"],
      connectSrc: ["'self'"]
    }
  },
  crossOriginEmbedderPolicy: false
}));

const authLimiter = rateLimit({
  windowMs: 15 * 60 * 1000,
  max: 10,
  message: 'Too many authentication attempts, please try again later.',
  standardHeaders: true,
  legacyHeaders: false
});

const setupLimiter = rateLimit({
  windowMs: 60 * 60 * 1000,
  max: 5,
  message: 'Too many setup attempts, please try again later.',
  standardHeaders: true,
  legacyHeaders: false
});

const generalLimiter = rateLimit({
  windowMs: 15 * 60 * 1000,
  max: 100,
  standardHeaders: true,
  legacyHeaders: false
});

app.use(generalLimiter);

app.set('view engine', 'ejs');
app.set('views', path.join(__dirname, 'src', 'dashboard', 'views'));
app.use('/public', express.static(path.join(__dirname, 'src', 'dashboard', 'public')));
app.use(express.urlencoded({ extended: false }));
app.use(express.json());
app.use(cookieParser(localConfig.sessionSecret));

const sessionConfig: session.SessionOptions = {
  secret: localConfig.sessionSecret || 'temp-secret',
  resave: false,
  saveUninitialized: false,
  cookie: {
    maxAge: 24 * 60 * 60 * 1000,
    secure: process.env.NODE_ENV === 'production',
    httpOnly: true,
    sameSite: 'lax'
  }
};

if (localConfig.mongoUri) {
  sessionConfig.store = MongoStore.create({
    mongoUrl: localConfig.mongoUri,
    touchAfter: 24 * 3600,
    crypto: {
      secret: localConfig.sessionSecret
    }
  });
}

app.use(session(sessionConfig));

app.use(doubleCsrfProtection);

app.use((req: Request, res: Response, next: NextFunction) => {
  res.locals.csrfToken = generateCsrfToken(req, res);
  next();
});

const ensureConfigured = (req: Request, res: Response, next: NextFunction): void => {
  if (!configured) { res.redirect('/setup'); return; }
  next();
};

const ensureAuth = (req: Request, res: Response, next: NextFunction): void => {
  if (!configured) { res.redirect('/setup'); return; }
  if (req.session?.user) { next(); return; }
  res.redirect('/auth/discord');
};

const ensureAdmin = (req: Request, res: Response, next: NextFunction): void => {
  if (!configured) { res.redirect('/setup'); return; }
  if (!req.session?.user) { res.redirect('/auth/discord'); return; }
  if (adminIds.includes(req.session.user.id)) { next(); return; }
  res.status(403).send('<h1>403 Forbidden</h1><p>You do not have admin permissions to access this page.</p>');
};

const connectMongo = async (): Promise<void> => {
  if (!localConfig.mongoUri) return;
  if (mongoose.connection.readyState === 0) {
    try {
      await mongoose.connect(localConfig.mongoUri);
      console.log('✅ Dashboard MongoDB connected');
    } catch (err) {
      console.error('❌ Dashboard MongoDB connection failed:', (err as Error).message);
      throw err;
    }
  }
};

const getConfig = async (): Promise<IConfig | ReturnType<typeof configToView>> => {
  if (!localConfig.mongoUri) return configToView({} as Partial<IConfig>);
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

interface ConfigView {
  autoStart: boolean;
  presenceText: string;
  presenceType: number;
  commandScope: string;
  guildId: string;
  invitePermissions: string;
}

const configToView = (config: Partial<IConfig>): ConfigView => ({
  autoStart: config?.autoStart || false,
  presenceText: config?.presenceText || 'Ready to serve',
  presenceType: config?.presenceType ?? 0,
  commandScope: config?.commandScope || 'guild',
  guildId: config?.guildId || '',
  invitePermissions: config?.invitePermissions || '8'
});

// ─── Routes ──────────────────────────────────────────────────────────────────

app.get('/setup', (req: Request, res: Response) => {
  if (configured) { res.redirect('/'); return; }
  res.render('setup', { ownerId: req.query.ownerId || '' });
});

app.post('/setup', setupLimiter, async (req: Request, res: Response) => {
  try {
    const body = req.body as Record<string, string>;
    if (!body.botToken || !body.clientId || !body.clientSecret || !body.sessionSecret) {
      res.status(400).send('Missing required fields. Please fill in: Bot Token, Client ID, Client Secret, and Session Secret.');
      return;
    }

    const botToken = String(body.botToken).trim();
    const clientId = String(body.clientId).trim();
    const clientSecret = String(body.clientSecret).trim();
    const sessionSecret = String(body.sessionSecret).trim();
    const callbackUrl = String(body.callbackUrl || 'http://localhost:8080/auth/discord/callback').trim();
    const guildId = String(body.guildId || '').trim();
    const mongoUri = String(body.mongoUri || '').trim();
    const inputAdminIds = String(body.adminIds || '').trim();
    const port = Number.parseInt(body.port, 10);
    const presenceText = String(body.presenceText || 'Ready to serve').trim();
    const presenceType = Number.parseInt(body.presenceType, 10);
    const commandScope = String(body.commandScope || 'guild').trim() as 'global' | 'guild';
    const invitePermissions = String(body.invitePermissions || '8').trim();

    if (botToken.length < 50) {
      res.status(400).send('Bot Token appears to be invalid (too short).');
      return;
    }
    if (clientId.length < 10 || !/^\d+$/.test(clientId)) {
      res.status(400).send('Client ID must be a numeric Discord application ID.');
      return;
    }
    if (sessionSecret.length < 32) {
      res.status(400).send('Session Secret must be at least 32 characters for security.');
      return;
    }
    if (port && (port < 1 || port > 65535)) {
      res.status(400).send('Port must be between 1 and 65535.');
      return;
    }
    if (commandScope !== 'global' && commandScope !== 'guild') {
      res.status(400).send('Command scope must be either "global" or "guild".');
      return;
    }
    if (guildId && !/^\d+$/.test(guildId)) {
      res.status(400).send('Guild ID must be a numeric Discord server ID.');
      return;
    }
    if (mongoUri && !mongoUri.startsWith('mongodb://') && !mongoUri.startsWith('mongodb+srv://')) {
      res.status(400).send('MongoDB URI must start with mongodb:// or mongodb+srv://');
      return;
    }

    const newConfig: LocalConfig = {
      botToken,
      clientId,
      clientSecret,
      callbackUrl,
      guildId,
      mongoUri,
      sessionSecret,
      adminIds: inputAdminIds,
      port: port || 8080,
      autoStart: body.autoStart === 'on',
      presenceText,
      presenceType: presenceType || 0,
      commandScope,
      invitePermissions
    };

    await saveLocalConfig(newConfig);
    localConfig = newConfig;
    configured = isConfigured(localConfig);
    adminIds = (newConfig.adminIds || '')
      .split(',')
      .map(id => id.trim())
      .filter(id => id.length > 0);

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
      try {
        await connectMongo();
        await getConfig();
      } catch (err) {
        console.warn('MongoDB connection failed, continuing without database:', (err as Error).message);
      }
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
    console.error(err);
    res.status(500).send(`Setup failed: ${(err as Error).message}`);
  }
});

app.get('/', ensureAuth, async (_req: Request, res: Response) => {
  res.redirect('/selector');
});

app.get('/selector', ensureAuth, async (req: Request, res: Response) => {
  await getConfig();
  res.render('selector', {
    user: req.session.user,
    botStatus: botManager.client ? 'online' : 'offline'
  });
});

app.get('/auth/discord', authLimiter, (req: Request, res: Response) => {
  const config = localConfig;

  if (req.query.setup) {
    res.redirect('/setup?message=Please complete setup first, then you can login to get your Discord User ID.');
    return;
  }

  if (!config?.clientId) {
    res.status(500).send('OAuth not configured. Please complete setup first.');
    return;
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

app.get('/auth/discord/callback', authLimiter, async (req: Request, res: Response) => {
  const code = req.query.code as string | undefined;

  if (!code) {
    res.redirect('/');
    return;
  }

  try {
    const config = localConfig;
    if (!config?.clientId || !config?.clientSecret) {
      res.status(500).send('OAuth not configured. Please complete setup first.');
      return;
    }

    const tokenData = await getDiscordToken(code, config);
    if (!tokenData.access_token) {
      throw new Error('No access token received');
    }

    const userInfo = await getDiscordUser(tokenData.access_token);

    const guildsResponse = await fetch('https://discord.com/api/users/@me/guilds', {
      headers: { Authorization: `Bearer ${tokenData.access_token}` }
    });

    if (!guildsResponse.ok) {
      throw new Error(`Failed to fetch guilds: ${guildsResponse.status}`);
    }

    const guilds = await guildsResponse.json() as DiscordGuild[];

    const adminGuilds = guilds.filter(guild => {
      if (!guild.permissions) return false;
      const permissions = BigInt(guild.permissions);
      return (permissions & BigInt(32)) === BigInt(32);
    });

    req.session.user = {
      ...userInfo,
      guilds,
      adminGuilds,
      displayedGuilds: adminGuilds
    };

    res.redirect('/selector');
  } catch (err) {
    console.error('OAuth error:', err);
    res.status(500).send(`Authentication failed: ${(err as Error).message}. Please try again or check your OAuth configuration.`);
  }
});

app.get('/logout', (req: Request, res: Response) => {
  req.session.destroy(err => {
    if (err) {
      res.status(500).send('Failed to logout');
      return;
    }
    res.redirect(configured ? '/auth/discord' : '/setup');
  });
});

app.get('/manage/:guildId', ensureAuth, async (req: Request, res: Response) => {
  const { guildId } = req.params;

  const guild = req.session.user?.displayedGuilds?.find(g => g.id === guildId);
  if (!guild) {
    res.status(403).render('error', {
      error: 'Access Denied',
      message: 'You do not have permission to manage this guild.',
      user: req.session.user
    });
    return;
  }

  const config = cachedConfig || await getConfig();
  const inviteLink = botManager.getInviteLink({ permissions: (config as ConfigView).invitePermissions || '8' });

  res.render('dashboard', {
    user: req.session.user,
    guild,
    inviteLink,
    botStatus: botManager.client ? 'online' : 'offline',
    config: configToView(config as Partial<IConfig>)
  });
});

app.post('/control/start', ensureAdmin, async (_req: Request, res: Response) => {
  try {
    await botManager.start();
    res.redirect('/');
  } catch (err) {
    console.error(err);
    res.status(500).send('Failed to start bot');
  }
});

app.post('/control/stop', ensureAdmin, async (_req: Request, res: Response) => {
  try {
    await botManager.stop();
    res.redirect('/');
  } catch (err) {
    console.error(err);
    res.status(500).send('Failed to stop bot');
  }
});

app.post('/control/restart', ensureAdmin, async (_req: Request, res: Response) => {
  try {
    await botManager.restart();
    res.redirect('/');
  } catch (err) {
    console.error(err);
    res.status(500).send('Failed to restart bot');
  }
});

app.post('/control/status', ensureAdmin, async (req: Request, res: Response) => {
  const body = req.body as Record<string, string>;
  const { activityType, statusText } = body;

  if (!statusText || statusText.trim().length === 0) {
    res.status(400).send('Status text cannot be empty');
    return;
  }

  try {
    await botManager.setActivity({ type: Number(activityType), text: statusText });
    res.redirect('/');
  } catch (err) {
    console.error(err);
    res.status(500).send('Failed to update status');
  }
});

app.get('/control/invite', ensureAuth, async (_req: Request, res: Response) => {
  const config = cachedConfig || await getConfig();
  const link = botManager.getInviteLink({ permissions: (config as ConfigView).invitePermissions || '8' });
  res.redirect(link);
});

app.get('/control/config', ensureAdmin, async (req: Request, res: Response) => {
  const config = cachedConfig || await getConfig();
  res.render('dashboard', {
    user: req.session.user,
    guild: null,
    inviteLink: botManager.getInviteLink({ permissions: (config as ConfigView).invitePermissions || '8' }),
    botStatus: botManager.client ? 'online' : 'offline',
    config: configToView(config as Partial<IConfig>)
  });
});

app.post('/control/config', ensureAdmin, async (req: Request, res: Response) => {
  try {
    const body = req.body as Record<string, string>;
    const config = localConfig.mongoUri ? (cachedConfig || await getConfig() as IConfig) : null;
    const presenceType = Number.parseInt(body.presenceType, 10);
    const commandScope = (['global', 'guild'].includes(body.commandScope) ? body.commandScope : (
      (config as IConfig)?.commandScope || localConfig.commandScope || 'guild'
    )) as 'global' | 'guild';

    const previousToken = (config as IConfig)?.botToken || localConfig.botToken;
    const previousClientId = (config as IConfig)?.clientId || localConfig.clientId;
    const previousScope = (config as IConfig)?.commandScope || localConfig.commandScope;

    if (config) {
      const cfg = config as IConfig;
      cfg.autoStart = body.autoStart === 'on';
      cfg.presenceText = body.presenceText || cfg.presenceText;
      cfg.presenceType = Number.isInteger(presenceType) ? presenceType : cfg.presenceType;
      cfg.commandScope = commandScope;
      cfg.guildId = body.guildId?.trim() || '';
      cfg.invitePermissions = body.invitePermissions || cfg.invitePermissions;

      if (body.botToken) cfg.botToken = body.botToken;
      if (body.clientId) cfg.clientId = body.clientId;
      if (body.clientSecret) cfg.clientSecret = body.clientSecret;
      if (body.callbackUrl) cfg.callbackUrl = body.callbackUrl;

      await cfg.save();
      cachedConfig = cfg;
    }

    const updatedLocalConfig: LocalConfig = {
      ...localConfig,
      autoStart: body.autoStart === 'on',
      presenceText: body.presenceText || localConfig.presenceText,
      presenceType: Number.isInteger(presenceType) ? presenceType : localConfig.presenceType,
      commandScope,
      guildId: body.guildId?.trim() || localConfig.guildId || '',
      invitePermissions: body.invitePermissions || localConfig.invitePermissions
    };

    if (body.botToken) updatedLocalConfig.botToken = body.botToken;
    if (body.clientId) updatedLocalConfig.clientId = body.clientId;
    if (body.clientSecret) updatedLocalConfig.clientSecret = body.clientSecret;
    if (body.callbackUrl) updatedLocalConfig.callbackUrl = body.callbackUrl;

    await saveLocalConfig(updatedLocalConfig);
    localConfig = updatedLocalConfig;

    const cfg = config as IConfig | null;
    const configToApply = cfg ? {
      botToken: cfg.botToken || updatedLocalConfig.botToken,
      clientId: cfg.clientId || updatedLocalConfig.clientId,
      clientSecret: cfg.clientSecret || updatedLocalConfig.clientSecret,
      callbackUrl: cfg.callbackUrl || updatedLocalConfig.callbackUrl,
      guildId: cfg.guildId || updatedLocalConfig.guildId,
      commandScope: cfg.commandScope || updatedLocalConfig.commandScope,
      invitePermissions: cfg.invitePermissions || updatedLocalConfig.invitePermissions,
      presenceText: cfg.presenceText || updatedLocalConfig.presenceText,
      presenceType: cfg.presenceType ?? updatedLocalConfig.presenceType,
      autoStart: cfg.autoStart ?? updatedLocalConfig.autoStart
    } : updatedLocalConfig;

    botManager.applyConfig(configToApply);

    const needsRestart =
      botManager.client &&
      ((body.botToken && body.botToken !== previousToken) ||
        (body.clientId && body.clientId !== previousClientId) ||
        commandScope !== previousScope);

    if (botManager.client) {
      await botManager.setActivity({
        type: updatedLocalConfig.presenceType,
        text: updatedLocalConfig.presenceText
      });
      if (needsRestart) await botManager.restart();
    }

    res.redirect('/');
  } catch (err) {
    console.error(err);
    res.status(500).send('Failed to update configuration');
  }
});

app.post('/control/profile', ensureAdmin, async (req: Request, res: Response) => {
  try {
    if (botManager.client) {
      const body = req.body as Record<string, string>;
      const { username, avatar, banner, bio } = body;
      await botManager.updateBotProfile({ username, avatar, banner, bio });
    }
    res.redirect('/');
  } catch (err) {
    console.error(err);
    res.status(500).send('Failed to update bot profile');
  }
});

// ─── Bootstrap ───────────────────────────────────────────────────────────────

const bootstrap = async (): Promise<void> => {
  if (configured) {
    configureOAuth(localConfig);
    if (localConfig.mongoUri) {
      try {
        await connectMongo();
        await getConfig();
      } catch (err) {
        console.warn('Mongo connection failed, using local config only:', (err as Error).message);
      }
    }
  }

  const server = app.listen(localConfig.port || 8080, () => {
    console.log(`Dashboard running on port ${localConfig.port || 8080}`);
    if (!configured) {
      console.log(`Setup required: http://localhost:${localConfig.port || 8080}/setup`);
    }
  });

  if (configured && localConfig.autoStart) {
    botManager
      .start()
      .catch(err => console.error('Bot failed to start:', err));
  }

  const shutdown = async (signal: string): Promise<void> => {
    console.log(`\n${signal} received. Shutting down gracefully...`);

    server.close(() => {
      console.log('HTTP server closed.');
    });

    if (botManager.client) {
      await botManager.stop();
      console.log('Bot stopped.');
    }

    if (mongoose.connection.readyState === 1) {
      await mongoose.connection.close();
      console.log('MongoDB connection closed.');
    }

    process.exit(0);
  };

  process.on('SIGTERM', () => void shutdown('SIGTERM'));
  process.on('SIGINT', () => void shutdown('SIGINT'));
};

bootstrap().catch(err => {
  console.error('Failed to bootstrap app:', err);
  process.exit(1);
});
