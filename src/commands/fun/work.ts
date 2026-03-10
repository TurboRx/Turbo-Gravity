import { SlashCommandBuilder, EmbedBuilder, ChatInputCommandInteraction } from 'discord.js';
import User from '../../models/User.js';

const WORK_COOLDOWN_MS = 60 * 60 * 1000;
const MIN_COINS = 25;
const MAX_COINS = 75;

const WORK_MESSAGES: string[] = [
  'You delivered packages and earned',
  'You fixed some bugs and earned',
  'You washed dishes and earned',
  'You tutored a student and earned',
  'You walked dogs and earned',
  'You coded a feature and earned',
  'You designed a logo and earned',
  'You drove a taxi and earned',
  'You sold lemonade and earned'
];

export default {
  data: new SlashCommandBuilder()
    .setName('work')
    .setDescription('Work to earn coins (1-hour cooldown)'),
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    let profile;
    try {
      profile = await User.findOneAndUpdate(
        { discordId: interaction.user.id },
        {
          $setOnInsert: {
            username: interaction.user.username,
            discriminator: interaction.user.discriminator || '0',
            avatar: interaction.user.avatar
          }
        },
        { upsert: true, new: true, setDefaultsOnInsert: true }
      );
    } catch (_) {
      await interaction.reply({ content: 'Database is unavailable.', ephemeral: true });
      return;
    }

    if (!profile) {
      await interaction.reply({ content: 'Could not create or find your profile.', ephemeral: true });
      return;
    }

    const now = Date.now();
    const lastWork = profile.lastWork ? profile.lastWork.getTime() : 0;
    const elapsed = now - lastWork;

    if (elapsed < WORK_COOLDOWN_MS) {
      const remaining = WORK_COOLDOWN_MS - elapsed;
      const minutes = Math.floor(remaining / 60000);
      const seconds = Math.floor((remaining % 60000) / 1000);
      await interaction.reply({
        content: `⏰ You're tired! Rest for **${minutes}m ${seconds}s** before working again.`,
        ephemeral: true
      });
      return;
    }

    const earned = Math.floor(Math.random() * (MAX_COINS - MIN_COINS + 1)) + MIN_COINS;
    const xpGained = 5;
    const message = WORK_MESSAGES[Math.floor(Math.random() * WORK_MESSAGES.length)];

    profile.balance += earned;
    profile.xp += xpGained;
    profile.lastWork = new Date();

    while (profile.xp >= profile.level * 100) {
      profile.xp -= profile.level * 100;
      profile.level += 1;
    }

    await profile.save();

    const embed = new EmbedBuilder()
      .setTitle('💼 Work Complete!')
      .setDescription(`${message} **${earned} coins**!`)
      .setColor('#0ea5e9')
      .addFields(
        { name: 'Balance', value: profile.balance.toLocaleString(), inline: true },
        { name: 'XP', value: `${profile.xp} (Level ${profile.level})`, inline: true }
      )
      .setFooter({ text: 'Come back in 1 hour to work again.' });

    await interaction.reply({ embeds: [embed], ephemeral: true });
  }
};
