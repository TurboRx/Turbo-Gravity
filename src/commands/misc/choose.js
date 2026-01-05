import { SlashCommandBuilder } from 'discord.js';

function parseOptions(raw) {
  return raw
    .split(/[,|]/)
    .map(opt => opt.trim())
    .filter(Boolean);
}

export default {
  data: new SlashCommandBuilder()
    .setName('choose')
    .setDescription('Let the bot pick from your options')
    .addStringOption(option =>
      option.setName('options').setDescription('Comma or pipe-separated choices (min 2)').setRequired(true)
    )
    .addStringOption(option =>
      option.setName('question').setDescription('Optional question/context').setRequired(false)
    ),
  async execute(interaction) {
    const raw = interaction.options.getString('options');
    const question = interaction.options.getString('question');
    const options = parseOptions(raw);

    if (options.length < 2) {
      return interaction.reply({ content: 'Please provide at least two distinct options separated by commas or pipes.', ephemeral: true });
    }

    if (options.length > 25) {
      return interaction.reply({ content: 'Too many options! Please provide 25 or fewer choices.', ephemeral: true });
    }

    const choice = options[Math.floor(Math.random() * options.length)];
    const prompt = question ? `**${question}**\n` : '';

    return interaction.reply({ content: `${prompt}🎲 I choose: **${choice}**`, ephemeral: true });
  }
};
