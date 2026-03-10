import { readFile, writeFile, mkdir } from 'fs/promises';
import { existsSync } from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const CONFIG_DIR = path.join(__dirname, '..', '.config');
const CONFIG_FILE = path.join(CONFIG_DIR, 'settings.json');

export interface LocalConfig {
  botToken: string;
  clientId: string;
  clientSecret: string;
  callbackUrl: string;
  guildId: string;
  mongoUri: string;
  sessionSecret: string;
  adminIds: string;
  port: number;
  autoStart: boolean;
  presenceText: string;
  presenceType: number;
  commandScope: 'global' | 'guild';
  invitePermissions: string;
}

const defaultConfig: LocalConfig = {
  botToken: '',
  clientId: '',
  clientSecret: '',
  callbackUrl: 'http://localhost:8080/auth/discord/callback',
  guildId: '',
  mongoUri: '',
  sessionSecret: '',
  adminIds: '',
  port: 8080,
  autoStart: false,
  presenceText: 'Ready to serve',
  presenceType: 0,
  commandScope: 'guild',
  invitePermissions: '8'
};

export async function loadLocalConfig(): Promise<LocalConfig> {
  try {
    if (!existsSync(CONFIG_FILE)) return { ...defaultConfig };
    const data = await readFile(CONFIG_FILE, 'utf-8');
    return { ...defaultConfig, ...JSON.parse(data) } as LocalConfig;
  } catch (err) {
    console.warn('Failed to load local config, using defaults:', (err as Error).message);
    return { ...defaultConfig };
  }
}

export async function saveLocalConfig(config: LocalConfig): Promise<void> {
  if (!existsSync(CONFIG_DIR)) {
    await mkdir(CONFIG_DIR, { recursive: true });
  }
  await writeFile(CONFIG_FILE, JSON.stringify(config, null, 2), 'utf-8');
}

export function isConfigured(config: LocalConfig): boolean {
  return !!(
    config.botToken &&
    config.clientId &&
    config.clientSecret &&
    config.sessionSecret
  );
}
