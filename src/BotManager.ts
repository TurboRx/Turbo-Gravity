import {
  Client,
  Collection,
  GatewayIntentBits,
  Partials,
  REST,
  Routes,
  ActivityType,
  SlashCommandBuilder,
  ChatInputCommandInteraction
} from 'discord.js';
import mongoose from 'mongoose';
import path from 'path';
import { fileURLToPath, pathToFileURL } from 'url';
import { readdir, stat } from 'fs/promises';
import Config, { IConfig } from './models/Config.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

export interface BotCommand {
  data: SlashCommandBuilder | ReturnType<SlashCommandBuilder['toJSON']>;
  execute: (interaction: ChatInputCommandInteraction) => Promise<unknown>;
}

export interface BotManagerOptions {
  token?: string;
  clientId?: string;
  guildId?: string;
  mongoUri?: string;
  defaultPresence?: {
    activities: Array<{ name: string; type: number }>;
    status: string;
  };
}

export interface ActivityOptions {
  type: number | string;
  text: string;
}

export interface BotProfileOptions {
  username?: string;
  avatar?: string;
  banner?: string;
  bio?: string;
}

export interface InviteLinkOptions {
  permissions?: string;
  scopes?: string[];
}

export interface FallbackConfig {
  token?: string;
  clientId?: string;
  guildId?: string;
  invitePermissions: string;
  commandScope: string;
  presenceText: string;
  presenceType: number;
}

export default class BotManager {
  mongoUri?: string;
  fallback: FallbackConfig;
  token?: string;
  clientId?: string;
  clientSecret?: string;
  guildId?: string;
  commandScope: string;
  invitePermissions: string;
  callbackUrl?: string;
  defaultPresence: { activities: Array<{ name: string; type: number }>; status: string };
  client: Client | null;
  commands: Collection<string, BotCommand>;
  rest: REST | null;
  config: IConfig | null;
  autoRestartAttempts: number;
  readonly maxAutoRestartAttempts: number;

  constructor({ token, clientId, guildId, mongoUri, defaultPresence }: BotManagerOptions) {
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
    this.commands = new Collection<string, BotCommand>();
    this.rest = null;
    this.config = null;
    this.autoRestartAttempts = 0;
    this.maxAutoRestartAttempts = 5;
  }

  async start(): Promise<Client> {
    if (this.client) return this.client;

    if (!this.token) {
      throw new Error('Missing DISCORD_TOKEN. Please configure the bot token in setup.');
    }
    if (!this.clientId) {
      throw new Error('Missing DISCORD_CLIENT_ID. Please configure the client ID in setup.');
    }

    await this.connectMongo();
    await this.ensureConfig();

    await this.buildClient();
    await this.registerCommands();

    return this.client!;
  }

  async stop(): Promise<void> {
    if (!this.client) return;
    this.client.removeAllListeners();
    await this.client.destroy();
    this.client = null;
  }

  async restart(): Promise<Client> {
    await this.stop();
    return this.start();
  }

  async setActivity({ type, text }: ActivityOptions): Promise<void> {
    if (!this.client) return;

    const processedText = this.processPresenceText(text);

    this.client.user?.setPresence({
      activities: [{ name: processedText || 'online', type: (Number(type) || 0) as ActivityType }],
      status: 'online'
    });
  }

  processPresenceText(text: string): string {
    if (!text || !this.client) return text;

    let processed = text;
    const guild = this.client.guilds.cache.first();

    processed = processed
      .replace(/{members}/gi, guild?.memberCount?.toString() ?? this.client.guilds.cache.reduce((a, g) => a + g.memberCount, 0).toString())
      .replace(/{guilds}/gi, this.client.guilds.cache.size.toString())
      .replace(/{users}/gi, this.client.users.cache.size.toString())
      .replace(/{botname}/gi, this.client.user?.username ?? '')
      .replace(/{servername}/gi, guild?.name || 'Discord')
      .replace(/{prefix}/gi, '/')
      .replace(/{timestamp}/gi, new Date().toLocaleString());

    return processed;
  }

  async updateBotProfile({ username, avatar, bio }: BotProfileOptions): Promise<void> {
    if (!this.client?.user) {
      console.warn('⚠️  Cannot update profile: bot is not running');
      return;
    }

    try {
      const updateData: { username?: string; bio?: string; avatar?: string } = {};

      if (username && username.trim()) {
        updateData.username = username.trim();
      }

      if (bio && bio.trim()) {
        const processedBio = this.processPresenceText(bio.trim());
        updateData.bio = processedBio;
      }

      if (avatar && avatar.trim()) {
        const urlPattern = /^https?:\/\/.+\.(png|jpg|jpeg|gif|webp)(\?.*)?$/i;
        if (urlPattern.test(avatar.trim())) {
          updateData.avatar = avatar.trim();
        } else {
          console.error('❌ Invalid avatar URL format. Must be a valid image URL (PNG/JPG/JPEG/GIF/WEBP)');
        }
      }

      if (Object.keys(updateData).length > 0) {
        await this.client.user.edit(updateData);
        console.log('✅ Bot profile updated successfully');
      } else {
        console.log('ℹ️  No profile changes to apply');
      }
    } catch (err) {
      console.error('❌ Failed to update bot profile:', (err as Error).message);
      throw err;
    }
  }

  getInviteLink({ permissions, scopes = ['bot', 'applications.commands'] }: InviteLinkOptions = {}): string {
    if (!this.clientId) throw new Error('Missing DISCORD_CLIENT_ID');
    const params = new URLSearchParams({
      client_id: this.clientId,
      permissions: permissions || this.invitePermissions,
      scope: scopes.join(' ')
    });
    return `https://discord.com/api/oauth2/authorize?${params.toString()}`;
  }

  async buildClient(): Promise<void> {
    this.client = new Client({
      intents: [
        GatewayIntentBits.Guilds,
        GatewayIntentBits.GuildMembers,
        GatewayIntentBits.GuildMessages,
        GatewayIntentBits.GuildModeration,
        GatewayIntentBits.MessageContent
      ],
      partials: [Partials.Channel]
    });

    (this.client as Client & { commands: Collection<string, BotCommand> }).commands = this.commands;

    this.client.once('ready', () => {
      this.client!.user?.setPresence({
        activities: this.defaultPresence.activities.map(a => ({
          name: a.name,
          type: a.type as ActivityType
        })),
        status: 'online'
      });
      this.autoRestartAttempts = 0;
      console.log(`✅ Bot logged in as ${this.client!.user?.tag}`);
    });

    this.client.on('shardDisconnect', (event: { code: number }, id: number) => {
      console.warn(`⚠️  Shard ${id} disconnected (code: ${event.code})`);
      const fatalCodes = [4004, 4010, 4011, 4012, 4013, 4014];
      if (fatalCodes.includes(event.code)) {
        console.warn('Fatal disconnect code detected, attempting manual restart...');
        void this.autoRestart();
      }
    });

    this.client.on('error', (err: Error) => {
      console.error('❌ Client error:', err);
    });

    this.client.on('shardError', (err: Error) => {
      console.error('❌ Shard error:', err);
    });

    this.client.on('interactionCreate', async interaction => {
      if (!interaction.isChatInputCommand()) return;
      const command = this.commands.get(interaction.commandName);
      if (!command) return;
      try {
        await command.execute(interaction);
      } catch (err) {
        console.error('Command error:', err);
        const errorMessage = { content: 'An error occurred while executing this command.', ephemeral: true };
        try {
          if (interaction.deferred || interaction.replied) {
            await interaction.followUp(errorMessage);
          } else {
            await interaction.reply(errorMessage);
          }
        } catch (replyErr) {
          console.error('Failed to send error message:', replyErr);
        }
      }
    });

    await this.loadCommands();

    try {
      await this.client.login(this.token!);
    } catch (err) {
      console.error('❌ Failed to login to Discord:', (err as Error).message);
      throw new Error(`Discord login failed: ${(err as Error).message}. Please check your bot token.`);
    }
  }

  async autoRestart(): Promise<void> {
    if (this.autoRestartAttempts >= this.maxAutoRestartAttempts) {
      console.error(`Max auto-restart attempts (${this.maxAutoRestartAttempts}) reached. Manual restart required.`);
      return;
    }

    this.autoRestartAttempts++;
    const delay = Math.pow(2, this.autoRestartAttempts) * 1000;

    console.log(`Auto-restart attempt ${this.autoRestartAttempts}/${this.maxAutoRestartAttempts} in ${delay}ms...`);

    setTimeout(() => {
      void (async () => {
        try {
          await this.restart();
        } catch (err) {
          console.error('Auto-restart failed:', err);
          void this.autoRestart();
        }
      })();
    }, delay);
  }

  async connectMongo(): Promise<void> {
    if (!this.mongoUri) return;
    if (mongoose.connection.readyState === 1) return;

    try {
      await mongoose.connect(this.mongoUri);
      console.log('✅ MongoDB connected');
    } catch (err) {
      console.error('❌ MongoDB connection failed:', (err as Error).message);
      throw err;
    }
  }

  async loadCommands(): Promise<void> {
    this.commands.clear();
    const commandsPath = path.join(__dirname, 'commands');
    try {
      const files = await this.walk(commandsPath);
      for (const file of files) {
        if (!file.endsWith('.js') && !file.endsWith('.ts')) continue;
        const moduleUrl = pathToFileURL(file).href;
        const mod = (await import(moduleUrl)) as { default?: BotCommand };
        const command = mod.default;
        if (!command?.data || !command?.execute) continue;
        const name = typeof command.data === 'object' && 'name' in command.data
          ? (command.data as { name: string }).name
          : '';
        if (name) this.commands.set(name, command);
      }
    } catch (err) {
      console.warn('No commands directory found, skipping command load:', (err as Error).message);
    }
  }

  async registerCommands(): Promise<void> {
    if (!this.clientId || !this.token) {
      console.warn('⚠️  Skipping command registration: missing clientId or token');
      return;
    }

    const rest = new REST({ version: '10' }).setToken(this.token);
    this.rest = rest;
    const body = Array.from(this.commands.values()).map(cmd =>
      typeof cmd.data === 'object' && 'toJSON' in cmd.data
        ? (cmd.data as SlashCommandBuilder).toJSON()
        : cmd.data
    );

    if (body.length === 0) {
      console.log('ℹ️  No commands to register');
      return;
    }

    try {
      if (this.commandScope === 'guild' && this.guildId) {
        await rest.put(Routes.applicationGuildCommands(this.clientId, this.guildId), { body });
        console.log(`✅ Registered ${body.length} guild commands for guild ${this.guildId}`);
      } else {
        await rest.put(Routes.applicationCommands(this.clientId), { body });
        console.log(`✅ Registered ${body.length} global commands`);
      }
    } catch (err) {
      console.error('❌ Failed to register commands:', (err as Error).message);
      throw err;
    }
  }

  applyConfig(config: Partial<IConfig> & { botToken?: string; clientId?: string; clientSecret?: string; callbackUrl?: string; guildId?: string; commandScope?: string; invitePermissions?: string; presenceText?: string; presenceType?: number; autoStart?: boolean }): void {
    const presenceText = config.presenceText || this.fallback.presenceText;
    const presenceType = typeof config.presenceType === 'number'
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

  async ensureConfig(): Promise<IConfig | null> {
    if (!this.mongoUri || mongoose.connection.readyState !== 1) {
      this.config = null;
      return null;
    }
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

  async walk(dir: string): Promise<string[]> {
    const entries = await readdir(dir);
    const files = await Promise.all(entries.map(async entry => {
      const res = path.resolve(dir, entry);
      const stats = await stat(res);
      if (stats.isDirectory()) return this.walk(res);
      return [res];
    }));
    return files.flat();
  }
}
