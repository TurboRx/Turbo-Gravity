import { SlashCommandBuilder, ChatInputCommandInteraction } from 'discord.js';

function parseOptions(raw: string): string[] {
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
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    const raw = interaction.options.getString('options') ?? '';
    const question = interaction.options.getString('question');
    const options = parseOptions(raw);

    if (options.length < 2) {
      await interaction.reply({ content: 'Please provide at least two distinct options separated by commas or pipes.', ephemeral: true });
      return;
    }

    if (options.length > 25) {
      await interaction.reply({ content: 'Too many options! Please provide 25 or fewer choices.', ephemeral: true });
      return;
    }

    const choice = options[Math.floor(Math.random() * options.length)];
    const prompt = question ? `**${question}**\n` : '';

    await interaction.reply({ content: `${prompt}🎲 I choose: **${choice}**`, ephemeral: true });
  }
};
