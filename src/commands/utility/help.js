import { SlashCommandBuilder, EmbedBuilder } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('help')
    .setDescription('List available commands'),
  async execute(interaction) {
    const commands = Array.from(interaction.client.commands.values())
      .sort((a, b) => a.data.name.localeCompare(b.data.name));
    const fields = commands.map(cmd => ({
      name: `/${cmd.data.name}`,
      value: cmd.data.description || 'No description provided',
      inline: true
    }));

    const embed = new EmbedBuilder()
      .setTitle('Available Commands')
      .setColor('#5865f2')
      .addFields(fields)
      .setFooter({ text: 'Use slash commands directly in your server.' });

    return interaction.reply({ embeds: [embed], ephemeral: true });
  }
};
