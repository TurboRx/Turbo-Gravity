import { SlashCommandBuilder, EmbedBuilder } from 'discord.js';

const numberEmojis = ['1️⃣','2️⃣','3️⃣','4️⃣','5️⃣','6️⃣','7️⃣','8️⃣','9️⃣','🔟'];

export default {
  data: new SlashCommandBuilder()
    .setName('poll')
    .setDescription('Create a quick reaction poll')
    .addStringOption(option =>
      option.setName('question').setDescription('Poll question').setRequired(true)
    )
    .addStringOption(option =>
      option
        .setName('choices')
        .setDescription('Comma-separated choices (2-10)')
        .setRequired(true)
    ),
  async execute(interaction) {
    const question = interaction.options.getString('question');
    const rawChoices = interaction.options.getString('choices');
    const choices = rawChoices
      .split(',')
      .map(c => c.trim())
      .filter(Boolean);

    if (choices.length < 2 || choices.length > 10) {
      return interaction.reply({ content: 'Please provide between 2 and 10 choices separated by commas.', ephemeral: true });
    }

    const lines = choices.map((choice, idx) => `${numberEmojis[idx]} ${choice}`);
    const embed = new EmbedBuilder()
      .setTitle('📊 ' + question)
      .setDescription(lines.join('\n'))
      .setColor('#f59e0b')
      .setFooter({ text: `Poll by ${interaction.user.tag}` });

    const message = await interaction.reply({ embeds: [embed], fetchReply: true });

    for (let i = 0; i < choices.length; i += 1) {
      await message.react(numberEmojis[i]);
    }
  }
};
