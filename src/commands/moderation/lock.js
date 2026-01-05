import { SlashCommandBuilder, PermissionFlagsBits } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('lock')
    .setDescription('Lock the channel to stop new messages')
    .setDefaultMemberPermissions(PermissionFlagsBits.ManageChannels)
    .addStringOption(option =>
      option.setName('reason').setDescription('Reason for locking the channel').setRequired(false)
    ),
  async execute(interaction) {
    const reason = interaction.options.getString('reason') || 'No reason provided';
    const everyone = interaction.guild.roles.everyone;

    if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.ManageChannels)) {
      return interaction.reply({ content: 'I need manage channels permission to do that.', ephemeral: true });
    }

    const overwrite = interaction.channel.permissionOverwrites.cache.get(everyone.id);
    const alreadyLocked = overwrite?.deny?.has('SendMessages');

    if (alreadyLocked) {
      return interaction.reply({ content: 'Channel is already locked.', ephemeral: true });
    }

    try {
      await interaction.channel.permissionOverwrites.edit(everyone, {
        SendMessages: false
      }, { reason });
      return interaction.reply({ content: `🔒 Channel locked. Reason: ${reason}` });
    } catch (err) {
      return interaction.reply({ content: `Failed to lock channel: ${err.message}`, ephemeral: true });
    }
  }
};
