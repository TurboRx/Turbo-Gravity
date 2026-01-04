import { SlashCommandBuilder } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('roll')
    .setDescription('Roll a dice')
    .addIntegerOption(option =>
      option
        .setName('sides')
        .setDescription('Number of sides on the die (2-1000)')
        .setRequired(false)
        .setMinValue(2)
        .setMaxValue(1000)
    ),
  async execute(interaction) {
    const sides = interaction.options.getInteger('sides') || 6;
    const result = Math.floor(Math.random() * sides) + 1;
    return interaction.reply({ content: `🎲 Rolled a ${sides}-sided die: **${result}**`, ephemeral: true });
  }
};
