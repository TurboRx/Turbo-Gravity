import { SlashCommandBuilder, PermissionFlagsBits, EmbedBuilder } from 'discord.js';

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
  async execute(interaction) {
    const target = interaction.options.getMember('target');
    const reason = interaction.options.getString('reason');

    if (!target) {
      return interaction.reply({ content: 'Unable to find that member.', ephemeral: true });
    }

    if (target.id === interaction.user.id) {
      return interaction.reply({ content: 'You cannot warn yourself.', ephemeral: true });
    }

    if (target.id === interaction.client.user.id) {
      return interaction.reply({ content: 'I cannot warn myself.', ephemeral: true });
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

    return interaction.reply({ embeds: [embed], ephemeral: true });
  }
};
