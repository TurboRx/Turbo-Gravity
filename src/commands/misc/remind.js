import { SlashCommandBuilder } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('remind')
    .setDescription('Set a one-time reminder')
    .addStringOption(option =>
      option.setName('message').setDescription('What to remind you about').setRequired(true)
    )
    .addIntegerOption(option =>
      option
        .setName('minutes')
        .setDescription('Minutes until reminder (1-10080)')
        .setRequired(true)
        .setMinValue(1)
        .setMaxValue(10080)
    ),
  async execute(interaction) {
    const text = interaction.options.getString('message');
    const minutes = interaction.options.getInteger('minutes');
    
    if (text.length > 1000) {
      return interaction.reply({ content: 'Reminder message is too long (max 1000 characters).', ephemeral: true });
    }
    
    const ms = minutes * 60 * 1000;

    await interaction.reply({ content: `⏰ Reminder set for ${minutes} minute(s). I'll DM you when it's time.`, ephemeral: true });

    setTimeout(async () => {
      try {
        await interaction.user.send(`⏰ **Reminder:** ${text}`);
      } catch (err) {
        try {
          await interaction.followUp({
            content: `⏰ Couldn't DM you, so here's your reminder: ${text}`,
            ephemeral: true
          });
        } catch (innerErr) {
          // final fallback: ignore if both fail
        }
      }
    }, ms);
  }
};
