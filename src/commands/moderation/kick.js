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

    if (!target.kickable) {
      return interaction.reply({ content: 'I cannot kick that member.', ephemeral: true });
    }

    await target.kick(reason);
    return interaction.reply({
      content: `Kicked ${target.user.tag} | Reason: ${reason}`,
      ephemeral: true
    });
  }
};
