import {
  SlashCommandBuilder,
  PermissionFlagsBits,
  ChannelType,
  EmbedBuilder,
  ChatInputCommandInteraction,
  GuildMember,
  TextChannel,
  CategoryChannel,
  Role,
  User
} from 'discord.js';

const TOPIC_PREFIX = 'Ticket owner:';

function extractOwnerId(channel: TextChannel): string | null {
  if (!channel.topic) return null;
  const match = channel.topic.match(/Ticket owner:\s*(\d{5,})/);
  return match ? match[1] : null;
}

function canManageTickets(member: GuildMember): boolean {
  return member.permissions.has(PermissionFlagsBits.ManageChannels);
}

async function handleCreate(interaction: ChatInputCommandInteraction): Promise<void> {
  if (!interaction.guild) return;
  const reason = interaction.options.getString('reason') || 'No reason provided';
  const category = interaction.options.getChannel('category') as CategoryChannel | null;
  const staffRole = interaction.options.getRole('staffrole') as Role | null;

  if (!interaction.guild.members.me?.permissions.has(PermissionFlagsBits.ManageChannels)) {
    await interaction.reply({ content: 'I need manage channels permission to create tickets.', ephemeral: true });
    return;
  }

  const existing = interaction.guild.channels.cache.find(
    ch => (ch as TextChannel).topic?.includes(`${TOPIC_PREFIX} ${interaction.user.id}`)
  );

  if (existing) {
    await interaction.reply({ content: `You already have an open ticket: ${existing}`, ephemeral: true });
    return;
  }

  const overwrites = [
    {
      id: interaction.guild.roles.everyone,
      deny: [PermissionFlagsBits.ViewChannel, PermissionFlagsBits.SendMessages]
    },
    {
      id: interaction.user.id,
      allow: [PermissionFlagsBits.ViewChannel, PermissionFlagsBits.SendMessages, PermissionFlagsBits.ReadMessageHistory]
    },
    {
      id: interaction.client.user!.id,
      allow: [PermissionFlagsBits.ViewChannel, PermissionFlagsBits.SendMessages, PermissionFlagsBits.ManageChannels]
    }
  ];

  if (staffRole) {
    overwrites.push({
      id: staffRole.id,
      allow: [PermissionFlagsBits.ViewChannel, PermissionFlagsBits.SendMessages, PermissionFlagsBits.ReadMessageHistory]
    });
  }

  const channelName = `ticket-${interaction.user.username.toLowerCase().replace(/[^a-z0-9]/g, '') || 'user'}-${interaction.user.discriminator || interaction.user.id.slice(-4)}`;

  let channel: TextChannel;
  try {
    channel = await interaction.guild.channels.create({
      name: channelName.slice(0, 90),
      type: ChannelType.GuildText,
      parent: category?.type === ChannelType.GuildCategory ? category.id : undefined,
      topic: `${TOPIC_PREFIX} ${interaction.user.id} | Reason: ${reason}`,
      permissionOverwrites: overwrites
    }) as TextChannel;
  } catch (err) {
    await interaction.reply({ content: `Failed to create ticket: ${(err as Error).message}`, ephemeral: true });
    return;
  }

  const embed = new EmbedBuilder()
    .setTitle('Support Ticket')
    .setDescription('Thank you for reaching out. A team member will assist you shortly.')
    .addFields(
      { name: 'Opened by', value: `<@${interaction.user.id}>`, inline: true },
      { name: 'Reason', value: reason, inline: true }
    )
    .setColor('#0ea5e9')
    .setTimestamp(new Date());

  await channel.send({ content: staffRole ? `<@&${staffRole.id}>` : undefined, embeds: [embed] });

  await interaction.reply({ content: `Ticket created: ${channel}`, ephemeral: true });
}

async function handleClose(interaction: ChatInputCommandInteraction): Promise<void> {
  if (!interaction.guild) return;
  const reason = interaction.options.getString('reason') || 'No reason provided';
  const deleteAfter = interaction.options.getInteger('delete_after');
  const channel = interaction.channel as TextChannel;
  const ownerId = extractOwnerId(channel);

  if (!ownerId) {
    await interaction.reply({ content: 'This does not appear to be a ticket channel.', ephemeral: true });
    return;
  }

  const isOwner = ownerId === interaction.user.id;
  if (!isOwner && !canManageTickets(interaction.member as GuildMember)) {
    await interaction.reply({ content: 'You cannot close this ticket.', ephemeral: true });
    return;
  }

  try {
    await channel.permissionOverwrites.edit(interaction.guild.roles.everyone, { SendMessages: false });

    await interaction.reply({
      content: `🔒 Ticket closed. Reason: ${reason}${deleteAfter ? ` | Deleting in ${deleteAfter} minute(s)` : ''}`
    });

    if (deleteAfter && deleteAfter > 0) {
      setTimeout(() => {
        void channel.delete(`Ticket closed: ${reason}`).catch(() => {});
      }, deleteAfter * 60 * 1000);
    }
  } catch (err) {
    await interaction.reply({ content: `Failed to close ticket: ${(err as Error).message}`, ephemeral: true });
  }
}

async function handleAdd(interaction: ChatInputCommandInteraction): Promise<void> {
  const user = interaction.options.getUser('user', true) as User;
  const channel = interaction.channel as TextChannel;
  const ownerId = extractOwnerId(channel);

  if (!ownerId) {
    await interaction.reply({ content: 'This does not appear to be a ticket channel.', ephemeral: true });
    return;
  }

  const isOwner = ownerId === interaction.user.id;
  if (!isOwner && !canManageTickets(interaction.member as GuildMember)) {
    await interaction.reply({ content: 'You cannot modify this ticket.', ephemeral: true });
    return;
  }

  if (user.id === ownerId) {
    await interaction.reply({ content: 'That user is already the ticket owner.', ephemeral: true });
    return;
  }

  try {
    await channel.permissionOverwrites.edit(user.id, {
      ViewChannel: true,
      SendMessages: true,
      ReadMessageHistory: true
    });

    await interaction.reply({ content: `✅ Added ${user.tag} to the ticket.`, ephemeral: true });
  } catch (err) {
    await interaction.reply({ content: `Failed to add user: ${(err as Error).message}`, ephemeral: true });
  }
}

async function handleRemove(interaction: ChatInputCommandInteraction): Promise<void> {
  const user = interaction.options.getUser('user', true) as User;
  const channel = interaction.channel as TextChannel;
  const ownerId = extractOwnerId(channel);

  if (!ownerId) {
    await interaction.reply({ content: 'This does not appear to be a ticket channel.', ephemeral: true });
    return;
  }

  const isOwner = ownerId === interaction.user.id;
  if (!isOwner && !canManageTickets(interaction.member as GuildMember)) {
    await interaction.reply({ content: 'You cannot modify this ticket.', ephemeral: true });
    return;
  }

  if (user.id === ownerId) {
    await interaction.reply({ content: 'Cannot remove the ticket owner.', ephemeral: true });
    return;
  }

  if (user.id === interaction.client.user?.id) {
    await interaction.reply({ content: 'Cannot remove the bot from the ticket.', ephemeral: true });
    return;
  }

  try {
    await channel.permissionOverwrites.delete(user.id);
    await interaction.reply({ content: `❌ Removed ${user.tag} from the ticket.`, ephemeral: true });
  } catch (err) {
    await interaction.reply({ content: `Failed to remove user: ${(err as Error).message}`, ephemeral: true });
  }
}

export default {
  data: new SlashCommandBuilder()
    .setName('ticket')
    .setDescription('Advanced ticket controls')
    .addSubcommand(sub =>
      sub
        .setName('create')
        .setDescription('Create a private support ticket')
        .addStringOption(option => option.setName('reason').setDescription('Reason for the ticket').setRequired(false))
        .addChannelOption(option =>
          option
            .setName('category')
            .setDescription('Category to place the ticket in')
            .addChannelTypes(ChannelType.GuildCategory)
            .setRequired(false)
        )
        .addRoleOption(option => option.setName('staffrole').setDescription('Role to notify and grant access').setRequired(false))
    )
    .addSubcommand(sub =>
      sub
        .setName('close')
        .setDescription('Close the current ticket')
        .addStringOption(option => option.setName('reason').setDescription('Close reason').setRequired(false))
        .addIntegerOption(option => option.setName('delete_after').setDescription('Delete after X minutes').setMinValue(1).setMaxValue(1440).setRequired(false))
    )
    .addSubcommand(sub =>
      sub
        .setName('add')
        .setDescription('Add a user to this ticket')
        .addUserOption(option => option.setName('user').setDescription('User to add').setRequired(true))
    )
    .addSubcommand(sub =>
      sub
        .setName('remove')
        .setDescription('Remove a user from this ticket')
        .addUserOption(option => option.setName('user').setDescription('User to remove').setRequired(true))
    ),
  async execute(interaction: ChatInputCommandInteraction): Promise<void> {
    const sub = interaction.options.getSubcommand();

    if (!interaction.guild) {
      await interaction.reply({ content: 'Tickets can only be used inside a server.', ephemeral: true });
      return;
    }

    if (sub === 'create') return handleCreate(interaction);
    if (sub === 'close') return handleClose(interaction);
    if (sub === 'add') return handleAdd(interaction);
    if (sub === 'remove') return handleRemove(interaction);

    await interaction.reply({ content: 'Unknown ticket action.', ephemeral: true });
  }
};
