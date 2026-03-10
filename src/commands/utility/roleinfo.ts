import { SlashCommandBuilder, EmbedBuilder, ChatInputCommandInteraction, Role } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('roleinfo')
    .setDescription('View details about a role')
    .addRoleOption(option =>
      option.setName('role').setDescription('Role to inspect').setRequired(true)
    ),
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    const role = interaction.options.getRole('role', true) as Role;

    const embed = new EmbedBuilder()
      .setTitle(role.name)
      .setColor(role.hexColor || '#5865f2')
      .addFields(
        { name: 'ID', value: role.id, inline: true },
        { name: 'Color', value: role.hexColor || 'None', inline: true },
        { name: 'Members', value: role.members.size.toString(), inline: true },
        { name: 'Mentionable', value: role.mentionable ? 'Yes' : 'No', inline: true },
        { name: 'Hoisted', value: role.hoist ? 'Yes' : 'No', inline: true },
        { name: 'Position', value: role.position.toString(), inline: true },
        { name: 'Created', value: `<t:${Math.floor(role.createdTimestamp / 1000)}:R>`, inline: true }
      );

    await interaction.reply({ embeds: [embed], ephemeral: true });
  }
};
