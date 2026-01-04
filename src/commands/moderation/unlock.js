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
    const canSend = interaction.channel.permissionsFor(everyone).has('SendMessages');

    if (canSend) {
      return interaction.reply({ content: 'Channel is already unlocked.', ephemeral: true });
    }

    await interaction.channel.permissionOverwrites.edit(everyone, {
      SendMessages: true
    }, { reason });

    return interaction.reply({ content: `Channel unlocked. Reason: ${reason}` });
  }
};
