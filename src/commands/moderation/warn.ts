import { SlashCommandBuilder, PermissionFlagsBits, EmbedBuilder, ChatInputCommandInteraction, GuildMember } from 'discord.js';
import Warning from '../../models/Warning.js';

export default {
  data: new SlashCommandBuilder()
    .setName('warn')
    .setDescription('Send a formal warning to a member')
    .setDefaultMemberPermissions(PermissionFlagsBits.ModerateMembers)
    .addUserOption(option =>
      option.setName('target').setDescription('Member to warn').setRequired(true)
    )
    .addStringOption(option =>
      option.setName('reason').setDescription('Reason for warning').setRequired(true)
    ),
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    if (!interaction.guild) return;
    const target = interaction.options.getMember('target') as GuildMember | null;
    const reason = interaction.options.getString('reason', true);

    if (!target) {
      await interaction.reply({ content: 'Unable to find that member.', ephemeral: true });
      return;
    }

    if (target.id === interaction.user.id) {
      await interaction.reply({ content: 'You cannot warn yourself.', ephemeral: true });
      return;
    }

    if (target.id === interaction.client.user?.id) {
      await interaction.reply({ content: 'I cannot warn myself.', ephemeral: true });
      return;
    }

    let warnCount: number | null = null;
    try {
      await Warning.create({
        guildId: interaction.guild.id,
        userId: target.id,
        moderatorId: interaction.user.id,
        reason
      });
      warnCount = await Warning.countDocuments({ guildId: interaction.guild.id, userId: target.id });
    } catch (_) {
      // Database unavailable — warning is still shown but not persisted
    }

    const embed = new EmbedBuilder()
      .setTitle('⚠️ Warning Issued')
      .setColor('#f59e0b')
      .addFields(
        { name: 'Member', value: `${target.user.tag} (${target.id})`, inline: true },
        { name: 'Moderator', value: interaction.user.tag, inline: true },
        { name: 'Reason', value: reason }
      )
      .setTimestamp(new Date());

    if (warnCount !== null) {
      embed.setFooter({ text: `Total warnings for this user: ${warnCount}` });
    }

    try {
      await target.user.send({
        embeds: [
          new EmbedBuilder()
            .setTitle(`⚠️ Warning from ${interaction.guild.name}`)
            .setColor('#f59e0b')
            .addFields({ name: 'Reason', value: reason })
            .setTimestamp(new Date())
        ]
      });
    } catch (_) {}

    await interaction.reply({ embeds: [embed], ephemeral: true });
  }
};
