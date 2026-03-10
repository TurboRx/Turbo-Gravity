import { SlashCommandBuilder, PermissionFlagsBits, ChatInputCommandInteraction, GuildMember } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('ban')
    .setDescription('Ban a user from the server')
    .setDefaultMemberPermissions(PermissionFlagsBits.BanMembers)
    .addUserOption(option =>
      option.setName('target').setDescription('User to ban').setRequired(true)
    )
    .addIntegerOption(option =>
      option
        .setName('delete_days')
        .setDescription('Days of messages to delete (0-7)')
        .setRequired(false)
        .setMinValue(0)
        .setMaxValue(7)
    )
    .addStringOption(option =>
      option.setName('reason').setDescription('Reason for ban').setRequired(false)
    ),
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    if (!interaction.guild) return;
    const user = interaction.options.getUser('target', true);
    const member = interaction.options.getMember('target') as GuildMember | null;
    const reason = interaction.options.getString('reason') || 'No reason provided';
    const deleteDays = interaction.options.getInteger('delete_days') ?? 0;

    if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.BanMembers)) {
      await interaction.reply({ content: 'I need ban permissions to do that.', ephemeral: true });
      return;
    }

    if (user.id === interaction.user.id) {
      await interaction.reply({ content: 'You cannot ban yourself.', ephemeral: true });
      return;
    }

    if (user.id === interaction.client.user?.id) {
      await interaction.reply({ content: 'I cannot ban myself.', ephemeral: true });
      return;
    }

    if (member) {
      if (!member.bannable) {
        await interaction.reply({ content: 'I cannot ban that member (role hierarchy or permissions).', ephemeral: true });
        return;
      }
      const issuer = interaction.member as GuildMember;
      if (issuer.roles.highest.position <= member.roles.highest.position) {
        await interaction.reply({ content: 'You cannot ban someone with an equal or higher role.', ephemeral: true });
        return;
      }
    }

    try {
      try {
        await user.send(`You were banned from **${interaction.guild.name}** | Reason: ${reason}`);
      } catch (_) {
        // ignore DM failures
      }
      await interaction.guild.members.ban(user, { reason, deleteMessageSeconds: deleteDays * 86400 });
      await interaction.reply({
        content: `Banned ${user.tag}${deleteDays ? ` and deleted ${deleteDays} day(s) of messages` : ''} | Reason: ${reason}`,
        ephemeral: true
      });
    } catch (err) {
      await interaction.reply({ content: `Failed to ban: ${(err as Error).message}`, ephemeral: true });
    }
  }
};
