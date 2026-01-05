import { SlashCommandBuilder, PermissionFlagsBits } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('unlock')
    .setDescription('Unlock the channel and allow messages')
    .setDefaultMemberPermissions(PermissionFlagsBits.ManageChannels)
    .addStringOption(option =>
      option.setName('reason').setDescription('Reason for unlocking the channel').setRequired(false)
    ),
  async execute(interaction) {
    const reason = interaction.options.getString('reason') || 'No reason provided';
    const everyone = interaction.guild.roles.everyone;

    if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.ManageChannels)) {
      return interaction.reply({ content: 'I need manage channels permission to do that.', ephemeral: true });
    }

    const canSend = interaction.channel.permissionsFor(everyone).has('SendMessages');

    if (canSend) {
      return interaction.reply({ content: 'Channel is already unlocked.', ephemeral: true });
    }

    try {
      await interaction.channel.permissionOverwrites.edit(everyone, {
        SendMessages: true
      }, { reason });
      return interaction.reply({ content: `🔓 Channel unlocked. Reason: ${reason}` });
    } catch (err) {
      return interaction.reply({ content: `Failed to unlock channel: ${err.message}`, ephemeral: true });
    }
  }
};
