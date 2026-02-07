import { SlashCommandBuilder, EmbedBuilder, PermissionFlagsBits } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('embed')
    .setDescription('Send a custom embed message')
    .setDefaultMemberPermissions(PermissionFlagsBits.ManageMessages)
    .addStringOption(option =>
      option.setName('title').setDescription('Embed title').setRequired(true)
    )
    .addStringOption(option =>
      option.setName('description').setDescription('Embed description').setRequired(true)
    )
    .addStringOption(option =>
      option.setName('color').setDescription('Hex color (e.g. #ff0000)').setRequired(false)
    )
    .addStringOption(option =>
      option.setName('footer').setDescription('Footer text').setRequired(false)
    )
    .addStringOption(option =>
      option.setName('image').setDescription('Image URL').setRequired(false)
    )
    .addStringOption(option =>
      option.setName('thumbnail').setDescription('Thumbnail URL').setRequired(false)
    ),
  async execute(interaction) {
    const title = interaction.options.getString('title');
    const description = interaction.options.getString('description');
    const color = interaction.options.getString('color');
    const footer = interaction.options.getString('footer');
    const image = interaction.options.getString('image');
    const thumbnail = interaction.options.getString('thumbnail');

    const embed = new EmbedBuilder()
      .setTitle(title)
      .setDescription(description)
      .setColor(color && /^#[0-9a-fA-F]{6}$/.test(color) ? color : '#5865f2')
      .setTimestamp(new Date());

    if (footer) embed.setFooter({ text: footer });
    if (image) embed.setImage(image);
    if (thumbnail) embed.setThumbnail(thumbnail);

    await interaction.channel.send({ embeds: [embed] });
    return interaction.reply({ content: 'Embed sent.', ephemeral: true });
  }
};
