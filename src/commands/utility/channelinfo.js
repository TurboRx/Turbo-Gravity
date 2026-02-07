import { SlashCommandBuilder, EmbedBuilder, ChannelType } from 'discord.js';

const channelTypeNames = {
  [ChannelType.GuildText]: 'Text',
  [ChannelType.GuildVoice]: 'Voice',
  [ChannelType.GuildCategory]: 'Category',
  [ChannelType.GuildAnnouncement]: 'Announcement',
  [ChannelType.GuildStageVoice]: 'Stage',
  [ChannelType.GuildForum]: 'Forum'
};

export default {
  data: new SlashCommandBuilder()
    .setName('channelinfo')
    .setDescription('View details about a channel')
    .addChannelOption(option =>
      option.setName('channel').setDescription('Channel to inspect').setRequired(false)
    ),
  async execute(interaction) {
    const channel = interaction.options.getChannel('channel') || interaction.channel;

    const embed = new EmbedBuilder()
      .setTitle(`#${channel.name}`)
      .setColor('#5865f2')
      .addFields(
        { name: 'ID', value: channel.id, inline: true },
        { name: 'Type', value: channelTypeNames[channel.type] || 'Unknown', inline: true },
        { name: 'Created', value: `<t:${Math.floor(channel.createdTimestamp / 1000)}:R>`, inline: true }
      );

    if (channel.topic) {
      embed.addFields({ name: 'Topic', value: channel.topic });
    }

    if (channel.rateLimitPerUser) {
      embed.addFields({ name: 'Slowmode', value: `${channel.rateLimitPerUser}s`, inline: true });
    }

    if (channel.nsfw !== undefined) {
      embed.addFields({ name: 'NSFW', value: channel.nsfw ? 'Yes' : 'No', inline: true });
    }

    if (channel.parentId) {
      embed.addFields({ name: 'Category', value: `<#${channel.parentId}>`, inline: true });
    }

    return interaction.reply({ embeds: [embed], ephemeral: true });
  }
};
