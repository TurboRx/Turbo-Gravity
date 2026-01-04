import { SlashCommandBuilder } from 'discord.js';

function formatDuration(ms) {
  const totalSeconds = Math.floor(ms / 1000);
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const parts = [];
  if (days) parts.push(`${days}d`);
  if (hours) parts.push(`${hours}h`);
  if (minutes) parts.push(`${minutes}m`);
  parts.push(`${seconds}s`);
  return parts.join(' ');
}

export default {
  data: new SlashCommandBuilder()
    .setName('contime')
    .setDescription('Show current Discord connection time'),
  async execute(interaction) {
    if (!interaction.client.readyTimestamp) {
      return interaction.reply({ content: 'Bot is not connected yet.', ephemeral: true });
    }

    const connectedMs = Date.now() - interaction.client.readyTimestamp;
    const shardPing = Math.round(interaction.client.ws.ping);

    return interaction.reply({
      content: `Connected for ${formatDuration(connectedMs)} | API ping: ${shardPing}ms`,
      ephemeral: true
    });
  }
};
