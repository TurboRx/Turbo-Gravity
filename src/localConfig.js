import { readFile, writeFile, mkdir } from 'fs/promises';
import { existsSync } from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const CONFIG_DIR = path.join(__dirname, '..', '.config');
const CONFIG_FILE = path.join(CONFIG_DIR, 'settings.json');

const defaultConfig = {
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

export async function loadLocalConfig() {
  try {
    if (!existsSync(CONFIG_FILE)) return { ...defaultConfig };
    const data = await readFile(CONFIG_FILE, 'utf-8');
    return { ...defaultConfig, ...JSON.parse(data) };
  } catch (err) {
    console.warn('Failed to load local config, using defaults:', err.message);
    return { ...defaultConfig };
  }
}

export async function saveLocalConfig(config) {
  if (!existsSync(CONFIG_DIR)) {
    await mkdir(CONFIG_DIR, { recursive: true });
  }
  await writeFile(CONFIG_FILE, JSON.stringify(config, null, 2), 'utf-8');
}

export function isConfigured(config) {
  return !!(
    config.botToken &&
    config.clientId &&
    config.clientSecret &&
    config.sessionSecret
  );
}
