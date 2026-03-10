import { SlashCommandBuilder, EmbedBuilder, ChatInputCommandInteraction, Collection } from 'discord.js';
import { BotCommand } from '../../BotManager.js';

export default {
  data: new SlashCommandBuilder()
    .setName('help')
    .setDescription('List available commands'),
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    const clientWithCommands = interaction.client as typeof interaction.client & {
      commands?: Collection<string, BotCommand>;
    };
    const commands = Array.from((clientWithCommands.commands?.values() ?? []))
      .sort((a, b) => {
        const aName = typeof a.data === 'object' && 'name' in a.data ? (a.data as { name: string }).name : '';
        const bName = typeof b.data === 'object' && 'name' in b.data ? (b.data as { name: string }).name : '';
        return aName.localeCompare(bName);
      });

    const fields = commands.map(cmd => {
      const name = typeof cmd.data === 'object' && 'name' in cmd.data ? (cmd.data as { name: string; description: string }).name : '';
      const description = typeof cmd.data === 'object' && 'description' in cmd.data ? (cmd.data as { name: string; description: string }).description : 'No description provided';
      return { name: `/${name}`, value: description, inline: true };
    });

    const embed = new EmbedBuilder()
      .setTitle('Available Commands')
      .setColor('#5865f2')
      .addFields(fields)
      .setFooter({ text: 'Use slash commands directly in your server.' });

    await interaction.reply({ embeds: [embed], ephemeral: true });
  }
};
