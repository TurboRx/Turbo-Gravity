import { SlashCommandBuilder, PermissionFlagsBits } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('slowmode')
    .setDescription('Set channel slowmode')
    .setDefaultMemberPermissions(PermissionFlagsBits.ManageChannels)
    .addIntegerOption(option =>
      option
        .setName('seconds')
        .setDescription('Slowmode duration in seconds (0 to disable)')
        .setRequired(true)
        .setMinValue(0)
        .setMaxValue(21600)
    )
    .addStringOption(option =>
      option.setName('reason').setDescription('Reason for slowmode change').setRequired(false)
    ),
  async execute(interaction) {
    const seconds = interaction.options.getInteger('seconds');
    const reason = interaction.options.getString('reason') || 'No reason provided';

    if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.ManageChannels)) {
      return interaction.reply({ content: 'I need manage channels permission to do that.', ephemeral: true });
    }

    try {
      await interaction.channel.setRateLimitPerUser(seconds, reason);
      return interaction.reply({
        content: seconds === 0
          ? 'Slowmode disabled for this channel.'
          : `Slowmode set to ${seconds} second(s). Reason: ${reason}`,
        ephemeral: true
      });
    } catch (err) {
      return interaction.reply({ content: `Failed to update slowmode: ${err.message}`, ephemeral: true });
    }
  }
};
