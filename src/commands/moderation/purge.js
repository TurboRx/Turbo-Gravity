import { SlashCommandBuilder, PermissionFlagsBits } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('purge')
    .setDescription('Bulk delete messages')
    .setDefaultMemberPermissions(PermissionFlagsBits.ManageMessages)
    .addNumberOption(option =>
      option.setName('amount').setDescription('Number of messages to delete').setRequired(true).setMinValue(1).setMaxValue(100)
    ),
  async execute(interaction) {
    const amount = interaction.options.getNumber('amount');
    if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.ManageMessages)) {
      return interaction.reply({ content: 'I need message management permissions to do that.', ephemeral: true });
    }

    const fetched = await interaction.channel.messages.fetch({ limit: amount });
    const deleted = await interaction.channel.bulkDelete(fetched, true);

    return interaction.reply({
      content: `Deleted ${deleted.size} messages.${deleted.size < fetched.size ? ' (Messages older than 14 days were skipped)' : ''}`,
      ephemeral: true
    });
  }
};
