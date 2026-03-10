import { SlashCommandBuilder, EmbedBuilder, ChatInputCommandInteraction } from 'discord.js';
import User from '../../models/User.js';

export default {
  data: new SlashCommandBuilder()
    .setName('balance')
    .setDescription('Check your coin balance and XP')
    .addUserOption(option =>
      option.setName('target').setDescription('User to check (defaults to yourself)').setRequired(false)
    ),
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    const user = interaction.options.getUser('target') || interaction.user;

    let profile;
    try {
      profile = await User.findOne({ discordId: user.id }).lean();
    } catch (_) {
      await interaction.reply({ content: 'Database is unavailable.', ephemeral: true });
      return;
    }

    if (!profile) {
      const isSelf = user.id === interaction.user.id;
      await interaction.reply({
        content: isSelf
          ? 'You have no profile yet. Use `/daily` or `/work` to get started!'
          : `${user.tag} has no profile yet.`,
        ephemeral: true
      });
      return;
    }

    const embed = new EmbedBuilder()
      .setAuthor({ name: user.tag, iconURL: user.displayAvatarURL({ size: 64 }) })
      .setColor('#f59e0b')
      .addFields(
        { name: '💰 Coins', value: profile.balance.toLocaleString(), inline: true },
        { name: '⭐ XP', value: profile.xp.toLocaleString(), inline: true },
        { name: '🏆 Level', value: profile.level.toString(), inline: true }
      );

    await interaction.reply({ embeds: [embed], ephemeral: true });
  }
};
