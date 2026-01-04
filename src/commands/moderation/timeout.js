import { SlashCommandBuilder, PermissionFlagsBits } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('timeout')
    .setDescription('Timeout a member')
    .setDefaultMemberPermissions(PermissionFlagsBits.ModerateMembers)
    .addUserOption(option =>
      option.setName('target').setDescription('Member to timeout').setRequired(true)
    )
    .addNumberOption(option =>
      option.setName('duration').setDescription('Duration in minutes').setRequired(true).setMinValue(1).setMaxValue(40320)
    )
    .addStringOption(option =>
      option.setName('reason').setDescription('Reason for timeout').setRequired(false)
    ),
  async execute(interaction) {
    const target = interaction.options.getMember('target');
    const duration = interaction.options.getNumber('duration') * 60 * 1000;
    const reason = interaction.options.getString('reason') || 'No reason provided';

    if (!target) {
      return interaction.reply({ content: 'Unable to find that member.', ephemeral: true });
    }

    if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.ModerateMembers)) {
      return interaction.reply({ content: 'I need moderate members permission to do that.', ephemeral: true });
    }

    if (target.id === interaction.user.id) {
      return interaction.reply({ content: 'You cannot timeout yourself.', ephemeral: true });
    }

    if (!target.moderatable) {
      return interaction.reply({ content: 'I cannot timeout that member.', ephemeral: true });
    }

    try {
      await target.timeout(duration, reason);
      return interaction.reply({
        content: `Timed out ${target.user.tag} for ${interaction.options.getNumber('duration')} minutes | Reason: ${reason}`,
        ephemeral: true
      });
    } catch (err) {
      return interaction.reply({ content: `Failed to timeout: ${err.message}`, ephemeral: true });
    }
  }
};
