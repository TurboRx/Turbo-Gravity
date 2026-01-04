import { SlashCommandBuilder, PermissionFlagsBits } from 'discord.js';

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
  async execute(interaction) {
    const userId = interaction.options.getString('userid');
    const reason = interaction.options.getString('reason') || 'No reason provided';

    if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.BanMembers)) {
      return interaction.reply({ content: 'I need ban permissions to do that.', ephemeral: true });
    }

    try {
      const bans = await interaction.guild.bans.fetch();
      const bannedUser = bans.get(userId);
      if (!bannedUser) {
        return interaction.reply({ content: 'That user is not banned.', ephemeral: true });
      }

      await interaction.guild.bans.remove(userId, reason);
      return interaction.reply({
        content: `Unbanned user ${bannedUser.user.tag} | Reason: ${reason}`,
        ephemeral: true
      });
    } catch (err) {
      return interaction.reply({
        content: `Failed to unban user: ${err.message}`,
        ephemeral: true
      });
    }
  }
};
