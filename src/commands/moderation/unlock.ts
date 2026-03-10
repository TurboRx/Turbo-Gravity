import {
  SlashCommandBuilder,
  PermissionFlagsBits,
  ChatInputCommandInteraction,
  TextChannel
} from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('unlock')
    .setDescription('Unlock the channel and allow messages')
    .setDefaultMemberPermissions(PermissionFlagsBits.ManageChannels)
    .addStringOption(option =>
      option.setName('reason').setDescription('Reason for unlocking the channel').setRequired(false)
    ),
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    if (!interaction.guild) return;
    const reason = interaction.options.getString('reason') || 'No reason provided';
    const everyone = interaction.guild.roles.everyone;
    const channel = interaction.channel as TextChannel;

    if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.ManageChannels)) {
      await interaction.reply({ content: 'I need manage channels permission to do that.', ephemeral: true });
      return;
    }

    const canSend = channel.permissionsFor(everyone)?.has('SendMessages');

    if (canSend) {
      await interaction.reply({ content: 'Channel is already unlocked.', ephemeral: true });
      return;
    }

    try {
      await channel.permissionOverwrites.edit(everyone, { SendMessages: true }, { reason });
      await interaction.reply({ content: `🔓 Channel unlocked. Reason: ${reason}` });
    } catch (err) {
      await interaction.reply({ content: `Failed to unlock channel: ${(err as Error).message}`, ephemeral: true });
    }
  }
};
