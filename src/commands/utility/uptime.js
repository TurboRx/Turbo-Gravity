import { SlashCommandBuilder, EmbedBuilder } from 'discord.js';

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
    .setName('uptime')
    .setDescription('Show bot process and connection uptime'),
  async execute(interaction) {
    const processMs = Math.floor(process.uptime() * 1000);
    const connectedMs = interaction.client.readyTimestamp
      ? Date.now() - interaction.client.readyTimestamp
      : 0;

    const embed = new EmbedBuilder()
      .setTitle('Uptime')
      .setColor('#22c55e')
      .addFields(
        { name: 'Process', value: formatDuration(processMs), inline: true },
        { name: 'Connected', value: connectedMs ? formatDuration(connectedMs) : 'Not connected', inline: true },
        { name: 'API Ping', value: `${Math.round(interaction.client.ws.ping)}ms`, inline: true }
      );

    return interaction.reply({ embeds: [embed], ephemeral: true });
  }
};
