import { SlashCommandBuilder, ChatInputCommandInteraction } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('coinflip')
    .setDescription('Flip a coin'),
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    const result = Math.random() < 0.5 ? 'Heads' : 'Tails';
    await interaction.reply({ content: `🪙 The coin landed on **${result}**.`, ephemeral: true });
  }
};
