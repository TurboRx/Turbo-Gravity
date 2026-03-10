import mongoose, { Document, Model, Schema } from 'mongoose';

export interface IWarning extends Document {
  guildId: string;
  userId: string;
  moderatorId: string;
  reason: string;
  createdAt: Date;
  updatedAt: Date;
}

const warningSchema = new Schema<IWarning>(
  {
    guildId: { type: String, required: true, index: true },
    userId: { type: String, required: true, index: true },
    moderatorId: { type: String, required: true },
    reason: { type: String, required: true }
  },
  { timestamps: true }
);

warningSchema.index({ guildId: 1, userId: 1 });

const Warning: Model<IWarning> = mongoose.model<IWarning>('Warning', warningSchema);

export default Warning;
