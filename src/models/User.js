import mongoose from 'mongoose';

const userSchema = new mongoose.Schema(
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

export default mongoose.model('User', userSchema);
