import { SlashCommandBuilder, ChatInputCommandInteraction } from 'discord.js';

function formatDuration(ms: number): string {
  const totalSeconds = Math.floor(ms / 1000);
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const parts: string[] = [];
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
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    if (!interaction.client.readyTimestamp) {
      await interaction.reply({ content: 'Bot is not connected yet.', ephemeral: true });
      return;
    }

    const connectedMs = Date.now() - interaction.client.readyTimestamp;
    const shardPing = Math.round(interaction.client.ws.ping);

    await interaction.reply({
      content: `Connected for ${formatDuration(connectedMs)} | API ping: ${shardPing}ms`,
      ephemeral: true
    });
  }
};
