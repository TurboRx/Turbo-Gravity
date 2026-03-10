import { SlashCommandBuilder, PermissionFlagsBits, ChatInputCommandInteraction, GuildMember } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('timeout')
    .setDescription('Timeout a member')
    .setDefaultMemberPermissions(PermissionFlagsBits.ModerateMembers)
    .addUserOption(option =>
      option.setName('target').setDescription('Member to timeout').setRequired(true)
    )
    .addIntegerOption(option =>
      option.setName('duration').setDescription('Duration in minutes').setRequired(true).setMinValue(1).setMaxValue(40320)
    )
    .addStringOption(option =>
      option.setName('reason').setDescription('Reason for timeout').setRequired(false)
    ),
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    if (!interaction.guild) return;
    const target = interaction.options.getMember('target') as GuildMember | null;
    const durationMin = interaction.options.getInteger('duration', true);
    const duration = durationMin * 60 * 1000;
    const reason = interaction.options.getString('reason') || 'No reason provided';

    if (!target) {
      await interaction.reply({ content: 'Unable to find that member.', ephemeral: true });
      return;
    }

    if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.ModerateMembers)) {
      await interaction.reply({ content: 'I need moderate members permission to do that.', ephemeral: true });
      return;
    }

    if (target.id === interaction.user.id) {
      await interaction.reply({ content: 'You cannot timeout yourself.', ephemeral: true });
      return;
    }

    if (target.id === interaction.client.user?.id) {
      await interaction.reply({ content: 'I cannot timeout myself.', ephemeral: true });
      return;
    }

    if (!target.moderatable) {
      await interaction.reply({ content: 'I cannot timeout that member.', ephemeral: true });
      return;
    }

    const issuer = interaction.member as GuildMember;
    if (issuer.roles.highest && target.roles.highest?.comparePositionTo(issuer.roles.highest) >= 0) {
      await interaction.reply({ content: 'You cannot timeout someone with an equal or higher role.', ephemeral: true });
      return;
    }

    try {
      await target.timeout(duration, reason);
      await interaction.reply({
        content: `Timed out ${target.user.tag} for ${durationMin} minutes | Reason: ${reason}`,
        ephemeral: true
      });
    } catch (err) {
      await interaction.reply({ content: `Failed to timeout: ${(err as Error).message}`, ephemeral: true });
    }
  }
};
