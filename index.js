import express from 'express';
import session from 'express-session';
import passport from 'passport';
import { Strategy as DiscordStrategy } from 'passport-discord';
import dotenv from 'dotenv';
import path from 'path';
import { fileURLToPath } from 'url';
import mongoose from 'mongoose';
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

passport.serializeUser((user, done) => done(null, user));
passport.deserializeUser((obj, done) => done(null, obj));

const configurePassport = config => {
  if (!config?.clientId || !config?.clientSecret) return;
  passport.use(
    new DiscordStrategy(
      {
        clientID: config.clientId,
        clientSecret: config.clientSecret,
        callbackURL: config.callbackUrl,
        scope: ['identify', 'guilds']
      },
      (accessToken, refreshToken, profile, done) => done(null, profile)
    )
  );
};

if (configured) configurePassport(localConfig);

const app = express();
app.set('view engine', 'ejs');
app.set('views', path.join(__dirname, 'src', 'dashboard', 'views'));
app.use('/public', express.static(path.join(__dirname, 'src', 'dashboard', 'public')));
app.use(express.urlencoded({ extended: false }));
app.use(express.json());
app.use(
  session({
    secret: localConfig.sessionSecret || 'temp-secret',
    resave: false,
    saveUninitialized: false
  })
);
app.use(passport.initialize());
app.use(passport.session());

const ensureConfigured = (req, res, next) => {
  if (!configured) return res.redirect('/setup');
  next();
};

const ensureAuth = (req, res, next) => {
  if (!configured) return res.redirect('/setup');
  if (req.isAuthenticated()) return next();
  res.redirect('/auth/discord');
};

const ensureAdmin = (req, res, next) => {
  if (!configured) return res.redirect('/setup');
  if (!req.isAuthenticated()) return res.redirect('/auth/discord');
  if (adminIds.includes(req.user.id)) return next();
  return res.status(403).send('Forbidden');
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
  res.render('setup');
});

app.post('/setup', async (req, res) => {
  try {
    const newConfig = {
      botToken: req.body.botToken,
      clientId: req.body.clientId,
      clientSecret: req.body.clientSecret,
      callbackUrl: req.body.callbackUrl || 'http://localhost:3000/auth/discord/callback',
      guildId: req.body.guildId || '',
      mongoUri: req.body.mongoUri,
      sessionSecret: req.body.sessionSecret,
      adminIds: req.body.adminIds || '',
      port: Number.parseInt(req.body.port, 10) || 3000,
      autoStart: req.body.autoStart === 'on',
      presenceText: req.body.presenceText || 'Ready to serve',
      presenceType: Number.parseInt(req.body.presenceType, 10) || 0,
      commandScope: req.body.commandScope || 'guild',
      invitePermissions: req.body.invitePermissions || '8'
    };

    await saveLocalConfig(newConfig);
    localConfig = newConfig;
    configured = isConfigured(localConfig);
    adminIds = (localConfig.adminIds || '')
      .split(',')
      .map(id => id.trim())
      .filter(id => id.length > 0);

    configurePassport(localConfig);
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
  const config = cachedConfig || await getConfig();
  const inviteLink = botManager.getInviteLink({ permissions: config.invitePermissions });
  res.render('dashboard', {
    user: req.user,
    inviteLink,
    botStatus: botManager.client ? 'online' : 'offline',
    config: configToView(config)
  });
});

app.get('/auth/discord', ensureConfigured, passport.authenticate('discord'));
app.get(
  '/auth/discord/callback',
  passport.authenticate('discord', { failureRedirect: '/' }),
  (req, res) => res.redirect('/')
);

app.get('/logout', (req, res, next) => {
  req.logout(err => {
    if (err) return next(err);
    res.redirect('/');
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
    user: req.user,
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

    const updatedOAuth = req.body.clientId || req.body.clientSecret || req.body.callbackUrl;
    if (updatedOAuth) configurePassport(config);

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

const bootstrap = async () => {
  if (configured && localConfig.mongoUri) {
    try {
      await connectMongo();
      await getConfig();
    } catch (err) {
      // eslint-disable-next-line no-console
      console.warn('Mongo connection failed, using local config only:', err.message);
    }
  }

  app.listen(localConfig.port, () => {
    // eslint-disable-next-line no-console
    console.log(`Dashboard running on port ${localConfig.port}`);
    if (!configured) {
      // eslint-disable-next-line no-console
      console.log(`Setup required: http://localhost:${localConfig.port}/setup`);
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
