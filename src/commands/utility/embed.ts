import {
  SlashCommandBuilder,
  EmbedBuilder,
  PermissionFlagsBits,
  ChatInputCommandInteraction,
  TextChannel
} from 'discord.js';

const IMAGE_URL_PATTERN = /^https?:\/\/.+\.(png|jpg|jpeg|gif|webp)(\?.*)?$/i;

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
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    const title = interaction.options.getString('title', true);
    const description = interaction.options.getString('description', true);
    const color = interaction.options.getString('color');
    const footer = interaction.options.getString('footer');
    const image = interaction.options.getString('image');
    const thumbnail = interaction.options.getString('thumbnail');
    const channel = interaction.channel as TextChannel;

    const embed = new EmbedBuilder()
      .setTitle(title)
      .setDescription(description)
      .setColor(color && /^#[0-9a-fA-F]{6}$/.test(color) ? (color as `#${string}`) : '#5865f2')
      .setTimestamp(new Date());

    if (footer) embed.setFooter({ text: footer });

    if (image) {
      if (IMAGE_URL_PATTERN.test(image)) {
        embed.setImage(image);
      } else {
        await interaction.reply({ content: 'Invalid image URL. Must be a direct link ending in .png, .jpg, .gif, or .webp.', ephemeral: true });
        return;
      }
    }
    if (thumbnail) {
      if (IMAGE_URL_PATTERN.test(thumbnail)) {
        embed.setThumbnail(thumbnail);
      } else {
        await interaction.reply({ content: 'Invalid thumbnail URL. Must be a direct link ending in .png, .jpg, .gif, or .webp.', ephemeral: true });
        return;
      }
    }

    await channel.send({ embeds: [embed] });
    await interaction.reply({ content: 'Embed sent.', ephemeral: true });
  }
};
