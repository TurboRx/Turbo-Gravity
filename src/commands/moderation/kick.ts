import { SlashCommandBuilder, PermissionFlagsBits, ChatInputCommandInteraction, GuildMember } from 'discord.js';

export default {
  data: new SlashCommandBuilder()
    .setName('kick')
    .setDescription('Kick a member from the server')
    .setDefaultMemberPermissions(PermissionFlagsBits.KickMembers)
    .addUserOption(option =>
      option.setName('target').setDescription('Member to kick').setRequired(true)
    )
    .addStringOption(option =>
      option.setName('reason').setDescription('Reason for kick').setRequired(false)
    ),
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    if (!interaction.guild) return;
    const target = interaction.options.getMember('target') as GuildMember | null;
    const reason = interaction.options.getString('reason') || 'No reason provided';

    if (!target) {
      await interaction.reply({ content: 'Unable to find that member.', ephemeral: true });
      return;
    }

    if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.KickMembers)) {
      await interaction.reply({ content: 'I need kick permissions to do that.', ephemeral: true });
      return;
    }

    if (target.id === interaction.user.id) {
      await interaction.reply({ content: 'You cannot kick yourself.', ephemeral: true });
      return;
    }

    if (target.id === interaction.client.user?.id) {
      await interaction.reply({ content: 'I cannot kick myself.', ephemeral: true });
      return;
    }

    if (!target.kickable) {
      await interaction.reply({ content: 'I cannot kick that member.', ephemeral: true });
      return;
    }

    const issuer = interaction.member as GuildMember;
    if (issuer.roles.highest.position <= target.roles.highest.position) {
      await interaction.reply({ content: 'You cannot kick someone with an equal or higher role.', ephemeral: true });
      return;
    }

    try {
      try {
        await target.user.send(`You were kicked from **${interaction.guild.name}** | Reason: ${reason}`);
      } catch (_) {}
      await target.kick(reason);
      await interaction.reply({
        content: `Kicked ${target.user.tag} | Reason: ${reason}`,
        ephemeral: true
      });
    } catch (err) {
      await interaction.reply({ content: `Failed to kick: ${(err as Error).message}`, ephemeral: true });
    }
  }
};
