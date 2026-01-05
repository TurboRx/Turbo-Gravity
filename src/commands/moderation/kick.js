import { SlashCommandBuilder, PermissionFlagsBits } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('kick')
    .setDescription('Kick a member from the server')
    .setDefaultMemberPermissions(PermissionFlagsBits.KickMembers)
    .addUserOption(option =>
      option.setName('target').setDescription('Member to kick').setRequired(true)
    )
    .addStringOption(option =>
      option.setName('reason').setDescription('Reason for kick').setRequired(false)
    ),
  async execute(interaction) {
    const target = interaction.options.getMember('target');
    const reason = interaction.options.getString('reason') || 'No reason provided';

    if (!target) {
      return interaction.reply({ content: 'Unable to find that member.', ephemeral: true });
    }

    if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.KickMembers)) {
      return interaction.reply({ content: 'I need kick permissions to do that.', ephemeral: true });
    }

    if (target.id === interaction.user.id) {
      return interaction.reply({ content: 'You cannot kick yourself.', ephemeral: true });
    }

    if (target.id === interaction.client.user.id) {
      return interaction.reply({ content: 'I cannot kick myself.', ephemeral: true });
    }

    if (!target.kickable) {
      return interaction.reply({ content: 'I cannot kick that member.', ephemeral: true });
    }

    const issuer = interaction.member;
    if (issuer.roles.highest.position <= target.roles.highest.position) {
      return interaction.reply({ content: 'You cannot kick someone with an equal or higher role.', ephemeral: true });
    }

    try {
      await target.kick(reason);
      try {
        await target.send(`You were kicked from **${interaction.guild.name}** | Reason: ${reason}`);
      } catch (err) {
        // ignore DM failures
      }
      return interaction.reply({
        content: `Kicked ${target.user.tag} | Reason: ${reason}`,
        ephemeral: true
      });
    } catch (err) {
      return interaction.reply({ content: `Failed to kick: ${err.message}`, ephemeral: true });
    }
  }
};
