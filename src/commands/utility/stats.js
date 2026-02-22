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
    .setName('stats')
    .setDescription('Show bot statistics'),
  async execute(interaction) {
    const client = interaction.client;
    const processMs = Math.floor(process.uptime() * 1000);
    const connectedMs = client.readyTimestamp ? Date.now() - client.readyTimestamp : 0;
    const totalMembers = client.guilds.cache.reduce((a, g) => a + g.memberCount, 0);

    const used = process.memoryUsage();
    const heapMB = (used.heapUsed / 1024 / 1024).toFixed(1);

    const embed = new EmbedBuilder()
      .setTitle(`📊 ${client.user.username} Statistics`)
      .setThumbnail(client.user.displayAvatarURL({ size: 128 }))
      .setColor('#5865f2')
      .addFields(
        { name: '🌐 Servers', value: client.guilds.cache.size.toString(), inline: true },
        { name: '👥 Members', value: totalMembers.toLocaleString(), inline: true },
        { name: '📡 API Ping', value: `${Math.round(client.ws.ping)}ms`, inline: true },
        { name: '⏱️ Uptime', value: connectedMs ? formatDuration(connectedMs) : 'Unknown', inline: true },
        { name: '🕐 Process', value: formatDuration(processMs), inline: true },
        { name: '🧠 Memory', value: `${heapMB} MB`, inline: true }
      )
      .setTimestamp(new Date());

    return interaction.reply({ embeds: [embed], ephemeral: true });
  }
};
