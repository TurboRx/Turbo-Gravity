import { SlashCommandBuilder } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('avatar')
    .setDescription('Get the avatar of a user')
    .addUserOption(option =>
      option.setName('target').setDescription('User to view').setRequired(false)
    ),
  async execute(interaction) {
    const user = interaction.options.getUser('target') || interaction.user;
    const avatarUrl = user.displayAvatarURL({ size: 1024, extension: 'png' });
    return interaction.reply({ content: `${user.tag}'s avatar: ${avatarUrl}`, ephemeral: true });
  }
};
