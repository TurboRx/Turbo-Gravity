import { SlashCommandBuilder, PermissionFlagsBits } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('lock')
    .setDescription('Lock the channel')
    .setDefaultMemberPermissions(PermissionFlagsBits.ManageChannels),
  async execute(interaction) {
    const everyone = interaction.guild.roles.everyone;
    await interaction.channel.permissionOverwrites.edit(everyone, {
      SendMessages: false
    });
    return interaction.reply({ content: 'Channel locked.' });
  }
};
