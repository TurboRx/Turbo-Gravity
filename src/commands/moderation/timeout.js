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
      option.setName('duration').setDescription('Duration in minutes').setRequired(true)
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

    await target.timeout(duration, reason);
    return interaction.reply({
      content: `Timed out ${target.user.tag} for ${interaction.options.getNumber('duration')} minutes | Reason: ${reason}`,
      ephemeral: true
    });
  }
};
