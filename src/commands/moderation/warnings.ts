import { SlashCommandBuilder, PermissionFlagsBits, EmbedBuilder, ChatInputCommandInteraction } from 'discord.js';
import Warning from '../../models/Warning.js';

export default {
  data: new SlashCommandBuilder()
    .setName('warnings')
    .setDescription('View warnings for a member')
    .setDefaultMemberPermissions(PermissionFlagsBits.ModerateMembers)
    .addUserOption(option =>
      option.setName('target').setDescription('Member to check').setRequired(true)
    )
    .addIntegerOption(option =>
      option.setName('page').setDescription('Page number').setMinValue(1).setRequired(false)
    ),
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    if (!interaction.guild) {
      await interaction.reply({ content: 'This command can only be used in a server.', ephemeral: true });
      return;
    }

    const user = interaction.options.getUser('target', true);
    const page = (interaction.options.getInteger('page') || 1) - 1;
    const perPage = 5;

    let warnings;
    try {
      warnings = await Warning.find({ guildId: interaction.guild.id, userId: user.id })
        .sort({ createdAt: -1 })
        .skip(page * perPage)
        .limit(perPage)
        .lean();
    } catch (_) {
      await interaction.reply({ content: 'Database is unavailable. Warnings cannot be retrieved.', ephemeral: true });
      return;
    }

    const total = await Warning.countDocuments({ guildId: interaction.guild.id, userId: user.id }).catch(() => 0);

    if (total === 0) {
      await interaction.reply({ content: `${user.tag} has no warnings in this server.`, ephemeral: true });
      return;
    }

    const embed = new EmbedBuilder()
      .setTitle(`⚠️ Warnings for ${user.tag}`)
      .setColor('#f59e0b')
      .setThumbnail(user.displayAvatarURL({ size: 128 }))
      .setFooter({ text: `Total: ${total} warning${total !== 1 ? 's' : ''} | Page ${page + 1} of ${Math.ceil(total / perPage)}` });

    if (warnings.length === 0) {
      embed.setDescription('No warnings on this page.');
    } else {
      warnings.forEach((w, i) => {
        embed.addFields({
          name: `#${page * perPage + i + 1} — <t:${Math.floor(new Date(w.createdAt).getTime() / 1000)}:R>`,
          value: `**Reason:** ${w.reason}\n**Moderator:** <@${w.moderatorId}>`
        });
      });
    }

    await interaction.reply({ embeds: [embed], ephemeral: true });
  }
};
