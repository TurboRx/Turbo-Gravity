import { SlashCommandBuilder, EmbedBuilder } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('userinfo')
    .setDescription('View info about a user')
    .addUserOption(option =>
      option.setName('target').setDescription('User to lookup').setRequired(false)
    ),
  async execute(interaction) {
    const user = interaction.options.getUser('target') || interaction.user;
    const member = await interaction.guild.members.fetch(user.id).catch(() => null);

    const embed = new EmbedBuilder()
      .setAuthor({ name: user.tag, iconURL: user.displayAvatarURL({ size: 256 }) })
      .setThumbnail(user.displayAvatarURL({ size: 256 }))
      .addFields(
        { name: 'User ID', value: user.id, inline: true },
        { name: 'Bot', value: user.bot ? 'Yes' : 'No', inline: true }
      )
      .setColor(member?.displayHexColor || '#5865f2');

    if (member) {
      embed.addFields(
        { name: 'Joined Server', value: `<t:${Math.floor(member.joinedTimestamp / 1000)}:R>`, inline: true },
        { name: 'Joined Discord', value: `<t:${Math.floor(user.createdTimestamp / 1000)}:R>`, inline: true },
        { name: 'Roles', value: member.roles.cache.filter(r => r.id !== interaction.guild.roles.everyone.id).map(r => r.toString()).join(', ') || 'None' }
      );
    } else {
      embed.addFields({ name: 'Joined Discord', value: `<t:${Math.floor(user.createdTimestamp / 1000)}:R>` });
    }

    return interaction.reply({ embeds: [embed], ephemeral: true });
  }
};
