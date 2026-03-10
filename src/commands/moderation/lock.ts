import {
  SlashCommandBuilder,
  PermissionFlagsBits,
  ChatInputCommandInteraction,
  TextChannel
} from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('lock')
    .setDescription('Lock the channel to stop new messages')
    .setDefaultMemberPermissions(PermissionFlagsBits.ManageChannels)
    .addStringOption(option =>
      option.setName('reason').setDescription('Reason for locking the channel').setRequired(false)
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

    const overwrite = channel.permissionOverwrites.cache.get(everyone.id);
    const alreadyLocked = overwrite?.deny?.has('SendMessages');

    if (alreadyLocked) {
      await interaction.reply({ content: 'Channel is already locked.', ephemeral: true });
      return;
    }

    try {
      await channel.permissionOverwrites.edit(everyone, { SendMessages: false }, { reason });
      await interaction.reply({ content: `🔒 Channel locked. Reason: ${reason}` });
    } catch (err) {
      await interaction.reply({ content: `Failed to lock channel: ${(err as Error).message}`, ephemeral: true });
    }
  }
};
