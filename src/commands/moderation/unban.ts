import { SlashCommandBuilder, PermissionFlagsBits, ChatInputCommandInteraction } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('unban')
    .setDescription('Unban a user from the server')
    .setDefaultMemberPermissions(PermissionFlagsBits.BanMembers)
    .addStringOption(option =>
      option.setName('userid').setDescription('User ID to unban').setRequired(true)
    )
    .addStringOption(option =>
      option.setName('reason').setDescription('Reason for unban').setRequired(false)
    ),
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    if (!interaction.guild) return;
    const userId = interaction.options.getString('userid', true);
    const reason = interaction.options.getString('reason') || 'No reason provided';

    if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.BanMembers)) {
      await interaction.reply({ content: 'I need ban permissions to do that.', ephemeral: true });
      return;
    }

    try {
      const bans = await interaction.guild.bans.fetch();
      const bannedUser = bans.get(userId);
      if (!bannedUser) {
        await interaction.reply({ content: 'That user is not banned.', ephemeral: true });
        return;
      }

      await interaction.guild.bans.remove(userId, reason);
      await interaction.reply({
        content: `Unbanned user ${bannedUser.user.tag} | Reason: ${reason}`,
        ephemeral: true
      });
    } catch (err) {
      await interaction.reply({
        content: `Failed to unban user: ${(err as Error).message}`,
        ephemeral: true
      });
    }
  }
};
