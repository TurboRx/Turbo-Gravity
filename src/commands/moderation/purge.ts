import {
  SlashCommandBuilder,
  PermissionFlagsBits,
  ChatInputCommandInteraction,
  TextChannel
} from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('purge')
    .setDescription('Bulk delete messages')
    .setDefaultMemberPermissions(PermissionFlagsBits.ManageMessages)
    .addIntegerOption(option =>
      option.setName('amount').setDescription('Number of messages to delete').setRequired(true).setMinValue(1).setMaxValue(100)
    ),
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    if (!interaction.guild) return;
    const amount = interaction.options.getInteger('amount', true);
    const channel = interaction.channel as TextChannel;

    if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.ManageMessages)) {
      await interaction.reply({ content: 'I need message management permissions to do that.', ephemeral: true });
      return;
    }

    try {
      const fetched = await channel.messages.fetch({ limit: amount });
      const deleted = await channel.bulkDelete(fetched, true);

      await interaction.reply({
        content: `Deleted ${deleted.size} messages.${deleted.size < fetched.size ? ' (Messages older than 14 days were skipped)' : ''}`,
        ephemeral: true
      });
    } catch (err) {
      await interaction.reply({ content: `Failed to delete messages: ${(err as Error).message}`, ephemeral: true });
    }
  }
};
