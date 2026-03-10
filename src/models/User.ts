import mongoose, { Document, Model, Schema } from 'mongoose';

export interface IUser extends Document {
  discordId: string;
  username: string;
  discriminator?: string;
  avatar?: string;
  balance: number;
  inventory: string[];
  xp: number;
  level: number;
  lastDaily?: Date;
  lastWork?: Date;
  createdAt: Date;
  updatedAt: Date;
}

const userSchema = new Schema<IUser>(
  {
    discordId: { type: String, required: true, unique: true },
    username: { type: String, required: true },
    discriminator: { type: String },
    avatar: { type: String },
    balance: { type: Number, default: 0 },
    inventory: { type: [String], default: [] },
    xp: { type: Number, default: 0 },
    level: { type: Number, default: 1 },
    lastDaily: { type: Date },
    lastWork: { type: Date }
  },
  { timestamps: true }
);

const User: Model<IUser> = mongoose.model<IUser>('User', userSchema);

export default User;
