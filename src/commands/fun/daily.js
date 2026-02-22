import { SlashCommandBuilder, EmbedBuilder } from 'discord.js';
import User from '../../models/User.js';

const DAILY_COOLDOWN_MS = 24 * 60 * 60 * 1000;
const MIN_COINS = 100;
const MAX_COINS = 200;

export default {
  data: new SlashCommandBuilder()
    .setName('daily')
    .setDescription('Claim your daily coin reward'),
  async execute(interaction) {
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
      return interaction.reply({ content: 'Database is unavailable.', ephemeral: true });
    }

    const now = Date.now();
    const lastDaily = profile.lastDaily ? profile.lastDaily.getTime() : 0;
    const elapsed = now - lastDaily;

    if (elapsed < DAILY_COOLDOWN_MS) {
      const remaining = DAILY_COOLDOWN_MS - elapsed;
      const hours = Math.floor(remaining / 3600000);
      const minutes = Math.floor((remaining % 3600000) / 60000);
      return interaction.reply({
        content: `⏰ Daily already claimed! Come back in **${hours}h ${minutes}m**.`,
        ephemeral: true
      });
    }

    const earned = Math.floor(Math.random() * (MAX_COINS - MIN_COINS + 1)) + MIN_COINS;
    const xpGained = 10;

    profile.balance += earned;
    profile.xp += xpGained;
    profile.lastDaily = new Date();

    while (profile.xp >= profile.level * 100) {
      profile.level += 1;
    }

    await profile.save();

    const embed = new EmbedBuilder()
      .setTitle('💰 Daily Reward Claimed!')
      .setColor('#22c55e')
      .addFields(
        { name: 'Earned', value: `+${earned} coins`, inline: true },
        { name: 'Balance', value: profile.balance.toLocaleString(), inline: true },
        { name: 'XP', value: `${profile.xp} (Level ${profile.level})`, inline: true }
      )
      .setFooter({ text: 'Come back tomorrow for more!' });

    return interaction.reply({ embeds: [embed], ephemeral: true });
  }
};
