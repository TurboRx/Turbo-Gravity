import mongoose from 'mongoose';

const configSchema = new mongoose.Schema(
  {
    autoStart: { type: Boolean, default: true },
    presenceText: { type: String, default: 'Ready to serve' },
    presenceType: { type: Number, default: 0 },
    commandScope: { type: String, enum: ['global', 'guild'], default: 'guild' },
    guildId: { type: String, default: '' },
    invitePermissions: { type: String, default: '8' },
    botToken: { type: String },
    clientId: { type: String },
    clientSecret: { type: String },
    callbackUrl: { type: String }
  },
  { timestamps: true }
);

export default mongoose.model('Config', configSchema);
