import { SlashCommandBuilder, EmbedBuilder, ChannelType } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('serverinfo')
    .setDescription('Show details about this server'),
  async execute(interaction) {
    const { guild } = interaction;
    const owner = await guild.fetchOwner().catch(() => null);
    const channels = guild.channels.cache;
    const textChannels = channels.filter(ch => ch.isTextBased() && ch.type !== ChannelType.GuildCategory).size;
    const voiceChannels = channels.filter(ch => ch.isVoiceBased()).size;

    const embed = new EmbedBuilder()
      .setTitle(guild.name)
      .setThumbnail(guild.iconURL({ size: 256 }))
      .addFields(
        { name: 'Server ID', value: guild.id, inline: true },
        { name: 'Owner', value: owner ? owner.user.tag : 'Unknown', inline: true },
        { name: 'Members', value: guild.memberCount.toString(), inline: true },
        { name: 'Channels', value: `${textChannels} text | ${voiceChannels} voice`, inline: true },
        { name: 'Created', value: `<t:${Math.floor(guild.createdTimestamp / 1000)}:F>`, inline: true }
      )
      .setColor('#00a884');

    return interaction.reply({ embeds: [embed], ephemeral: true });
  }
};
