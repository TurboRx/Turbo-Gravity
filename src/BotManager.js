import { Client, Collection, GatewayIntentBits, Partials, REST, Routes } from 'discord.js';
import mongoose from 'mongoose';
import path from 'path';
import { fileURLToPath, pathToFileURL } from 'url';
import { readdir, stat } from 'fs/promises';
import Config from './models/Config.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export default class BotManager {
  constructor({ token, clientId, guildId, mongoUri, defaultPresence }) {
    this.mongoUri = mongoUri;

    this.fallback = {
      token,
      clientId,
      guildId,
      invitePermissions: '8',
      commandScope: 'guild',
      presenceText: defaultPresence?.activities?.[0]?.name || 'online',
      presenceType: defaultPresence?.activities?.[0]?.type || 0
    };

    this.token = token;
    this.clientId = clientId;
    this.guildId = guildId;
    this.commandScope = 'guild';
    this.invitePermissions = '8';
    this.defaultPresence = defaultPresence || {
      activities: [{ name: 'online', type: 0 }],
      status: 'online'
    };

    this.client = null;
    this.commands = new Collection();
    this.rest = null;
    this.config = null;
  }

  async start() {
    if (this.client) return this.client;

    await this.connectMongo();
    await this.ensureConfig();

    if (!this.token) throw new Error('Missing DISCORD_TOKEN');

    await this.buildClient();
    await this.registerCommands();

    return this.client;
  }

  async stop() {
    if (!this.client) return;
    this.client.removeAllListeners();
    await this.client.destroy();
    this.client = null;
  }

  async restart() {
    await this.stop();
    return this.start();
  }

  async setActivity({ type, text }) {
    if (!this.client) return;
    
    // Replace variables in presence text
    const processedText = this.processPresenceText(text);
    
    this.client.user.setPresence({
      activities: [{ name: processedText || 'online', type: Number(type) || 0 }],
      status: 'online'
    });
  }

  processPresenceText(text) {
    if (!text || !this.client) return text;
    
    let processed = text;
    const guild = this.client.guilds.cache.first();
    
    // Replace variables
    processed = processed
      .replace(/{members}/gi, this.client.users.cache.size.toString())
      .replace(/{guilds}/gi, this.client.guilds.cache.size.toString())
      .replace(/{users}/gi, this.client.users.cache.size.toString())
      .replace(/{botname}/gi, this.client.user.username)
      .replace(/{servername}/gi, guild?.name || 'Discord')
      .replace(/{prefix}/gi, '/')
      .replace(/{timestamp}/gi, new Date().toLocaleString());
    
    return processed;
  }

  async updateBotProfile({ username, avatar, banner, bio }) {
    if (!this.client?.user) return;
    
    try {
      const updateData = {};
      
      if (username) updateData.username = username;
      if (bio) updateData.bio = bio;
      
      if (avatar) {
        // Assume avatar is a data URL or image URL
        updateData.avatar = avatar;
      }
      
      // Note: banner can only be set via user account, not bot user
      // We'll store it in config but can't apply it directly
      
      if (Object.keys(updateData).length > 0) {
        await this.client.user.edit(updateData);
        // eslint-disable-next-line no-console
        console.log('Bot profile updated');
      }
    } catch (err) {
      // eslint-disable-next-line no-console
      console.error('Failed to update bot profile:', err.message);
    }
  }

  getInviteLink({ permissions, scopes = ['bot', 'applications.commands'] } = {}) {
    if (!this.clientId) throw new Error('Missing DISCORD_CLIENT_ID');
    const params = new URLSearchParams({
      client_id: this.clientId,
      permissions: permissions || this.invitePermissions,
      scope: scopes.join(' ')
    });
    return `https://discord.com/api/oauth2/authorize?${params.toString()}`;
  }

  async buildClient() {
    this.client = new Client({
      intents: [
        GatewayIntentBits.Guilds,
        GatewayIntentBits.GuildMembers,
        GatewayIntentBits.GuildMessages,
        GatewayIntentBits.MessageContent
      ],
      partials: [Partials.Channel]
    });

    this.client.commands = this.commands;
    this.autoRestartAttempts = 0;
    this.maxAutoRestartAttempts = 5;
    
    this.client.once('ready', () => {
      this.client.user.setPresence(this.defaultPresence);
      this.autoRestartAttempts = 0; // Reset counter on successful connection
      // eslint-disable-next-line no-console
      console.log(`Logged in as ${this.client.user.tag}`);
    });

    this.client.on('disconnect', () => {
      // eslint-disable-next-line no-console
      console.warn('Bot disconnected, attempting auto-restart...');
      this.autoRestart();
    });

    this.client.on('error', err => {
      // eslint-disable-next-line no-console
      console.error('Client error:', err);
    });

    this.client.on('interactionCreate', async interaction => {
      if (!interaction.isChatInputCommand()) return;
      const command = this.commands.get(interaction.commandName);
      if (!command) return;
      try {
        await command.execute(interaction);
      } catch (err) {
        // eslint-disable-next-line no-console
        console.error('Command error:', err);
        if (interaction.deferred || interaction.replied) {
          await interaction.followUp({ content: 'Error executing command.', ephemeral: true });
        } else {
          await interaction.reply({ content: 'Error executing command.', ephemeral: true });
        }
      }
    });

    await this.loadCommands();
    await this.client.login(this.token);
  }

  async autoRestart() {
    if (this.autoRestartAttempts >= this.maxAutoRestartAttempts) {
      // eslint-disable-next-line no-console
      console.error(`Max auto-restart attempts (${this.maxAutoRestartAttempts}) reached. Manual restart required.`);
      return;
    }

    this.autoRestartAttempts++;
    const delay = Math.pow(2, this.autoRestartAttempts) * 1000; // Exponential backoff
    
    // eslint-disable-next-line no-console
    console.log(`Auto-restart attempt ${this.autoRestartAttempts}/${this.maxAutoRestartAttempts} in ${delay}ms...`);
    
    setTimeout(async () => {
      try {
        await this.restart();
      } catch (err) {
        // eslint-disable-next-line no-console
        console.error('Auto-restart failed:', err);
        this.autoRestart();
      }
    }, delay);
  }

  async connectMongo() {
    if (!this.mongoUri) return;
    if (mongoose.connection.readyState === 1) return;
    await mongoose.connect(this.mongoUri);
  }

  async loadCommands() {
    this.commands.clear();
    const commandsPath = path.join(__dirname, 'commands');
    try {
      const files = await this.walk(commandsPath);
      for (const file of files) {
        if (!file.endsWith('.js')) continue;
        const moduleUrl = pathToFileURL(file).href;
        const command = (await import(moduleUrl)).default;
        if (!command?.data || !command?.execute) continue;
        this.commands.set(command.data.name, command);
      }
    } catch (err) {
      // eslint-disable-next-line no-console
      console.warn('No commands directory found, skipping command load:', err.message);
    }
  }

  async registerCommands() {
    if (!this.clientId || !this.token) return;
    const rest = new REST({ version: '10' }).setToken(this.token);
    this.rest = rest;
    const body = Array.from(this.commands.values()).map(cmd => cmd.data.toJSON());
    if (body.length === 0) return;

    if (this.commandScope === 'guild' && this.guildId) {
      await rest.put(Routes.applicationGuildCommands(this.clientId, this.guildId), { body });
    } else {
      await rest.put(Routes.applicationCommands(this.clientId), { body });
    }
  }

  applyConfig(config) {
    const presenceText = config.presenceText || this.fallback.presenceText;
    const presenceType = Number.isInteger(config.presenceType)
      ? config.presenceType
      : this.fallback.presenceType;

    this.token = config.botToken || this.fallback.token;
    this.clientId = config.clientId || this.fallback.clientId;
    this.clientSecret = config.clientSecret;
    this.guildId = config.guildId || this.fallback.guildId;
    this.commandScope = config.commandScope || this.fallback.commandScope;
    this.invitePermissions = config.invitePermissions || this.fallback.invitePermissions;
    this.callbackUrl = config.callbackUrl;
    this.defaultPresence = {
      activities: [{ name: presenceText, type: presenceType || 0 }],
      status: 'online'
    };
  }

  async ensureConfig() {
    let config = await Config.findOne();
    if (!config) {
      config = await Config.create({
        botToken: this.fallback.token,
        clientId: this.fallback.clientId,
        guildId: this.fallback.guildId,
        presenceText: this.fallback.presenceText,
        presenceType: this.fallback.presenceType,
        invitePermissions: this.fallback.invitePermissions
      });
    }
    this.config = config;
    this.applyConfig(config);
    return config;
  }

  async walk(dir) {
    const entries = await readdir(dir);
    const files = await Promise.all(entries.map(async entry => {
      const res = path.resolve(dir, entry);
      const stats = await stat(res);
      if (stats.isDirectory()) return this.walk(res);
      return res;
    }));
    return files.flat();
  }
}
