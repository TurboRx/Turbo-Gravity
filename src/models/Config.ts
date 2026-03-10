import mongoose, { Document, Model, Schema } from 'mongoose';

export interface IConfig extends Document {
  autoStart: boolean;
  presenceText: string;
  presenceType: number;
  commandScope: 'global' | 'guild';
  guildId: string;
  invitePermissions: string;
  botToken?: string;
  clientId?: string;
  clientSecret?: string;
  callbackUrl?: string;
  createdAt: Date;
  updatedAt: Date;
}

const configSchema = new Schema<IConfig>(
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

const Config: Model<IConfig> = mongoose.model<IConfig>('Config', configSchema);

export default Config;
