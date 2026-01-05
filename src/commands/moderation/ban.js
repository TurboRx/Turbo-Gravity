import { SlashCommandBuilder, PermissionFlagsBits } from 'discord.js';

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
  async execute(interaction) {
    const user = interaction.options.getUser('target');
    const member = interaction.options.getMember('target');
    const reason = interaction.options.getString('reason') || 'No reason provided';
    const deleteDays = interaction.options.getInteger('delete_days') ?? 0;

    if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.BanMembers)) {
      return interaction.reply({ content: 'I need ban permissions to do that.', ephemeral: true });
    }

    if (user.id === interaction.user.id) {
      return interaction.reply({ content: 'You cannot ban yourself.', ephemeral: true });
    }

    if (user.id === interaction.client.user.id) {
      return interaction.reply({ content: 'I cannot ban myself.', ephemeral: true });
    }

    if (member) {
      if (!member.bannable) {
        return interaction.reply({ content: 'I cannot ban that member (role hierarchy or permissions).', ephemeral: true });
      }
      const issuer = interaction.member;
      if (issuer.roles.highest.position <= member.roles.highest.position) {
        return interaction.reply({ content: 'You cannot ban someone with an equal or higher role.', ephemeral: true });
      }
    }

    try {
      await interaction.guild.members.ban(user, { reason, deleteMessageSeconds: deleteDays * 86400 });
      try {
        await user.send(`You were banned from **${interaction.guild.name}** | Reason: ${reason}`);
      } catch (err) {
        // ignore DM failures
      }
      return interaction.reply({
        content: `Banned ${user.tag}${deleteDays ? ` and deleted ${deleteDays} day(s) of messages` : ''} | Reason: ${reason}`,
        ephemeral: true
      });
    } catch (err) {
      return interaction.reply({ content: `Failed to ban: ${err.message}`, ephemeral: true });
    }
  }
};
