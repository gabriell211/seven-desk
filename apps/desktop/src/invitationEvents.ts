import type { ActiveAuthorization, ChatMessage, PendingInvitation } from "./bridge";

export const INVITATION_WINDOW = "invitation";
export const INVITATION_STATE = "seven:invitation-state";
export const INVITATION_ACTION = "seven:invitation-action";
export const INVITATION_READY = "seven:invitation-ready";

export interface InvitationState {
  pending: PendingInvitation | null;
  active: ActiveAuthorization | null;
  nativeInputAvailable: boolean;
  monitorAvailable: boolean;
  deviceId: string | null;
  decisionBusy: boolean;
  ending: boolean;
  chatBusy: boolean;
  chatMessages: ChatMessage[];
  feedback: string;
}

export type InvitationAction =
  | { type: "decide"; sessionId: string; approve: boolean; mouse: boolean; keyboard: boolean; chat: boolean }
  | { type: "finalize"; sessionId: string }
  | { type: "chat"; sessionId: string; text: string };
