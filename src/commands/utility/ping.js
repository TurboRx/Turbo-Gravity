import { SlashCommandBuilder } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('ping')
    .setDescription('Check bot and API latency'),
  async execute(interaction) {
    const sent = await interaction.reply({ content: 'Pinging...', ephemeral: true, fetchReply: true });
    const latency = sent.createdTimestamp - interaction.createdTimestamp;
    const apiLatency = Math.round(interaction.client.ws.ping);
    return interaction.editReply(`Pong! Client latency: ${latency}ms | API latency: ${apiLatency}ms`);
  }
};
