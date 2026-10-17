import { invoke, isTauri } from "@tauri-apps/api/core";

export interface LocalStatus {
  app_name: string;
  app_version: string;
  operating_system: string;
  architecture: string;
  device_id: string | null;
  host_service_available: boolean;
}

export interface BackendStatus {
  status: "ok";
  release: string;
  protocol: number;
  production_ready: boolean;
}

export async function initializeIdentity(): Promise<string> {
  if (!isTauri()) throw new Error("Ativação disponível apenas no aplicativo instalado.");
  return invoke<string>("initialize_identity");
}

/** Local development signaling only. Signing happens in the native process. */
export async function registerLocalDevice(): Promise<string> {
  if (!isTauri()) throw new Error("Registro disponível apenas no aplicativo instalado.");
  return invoke<string>("register_local_device");
}

export async function heartbeatLocalDevice(): Promise<string> {
  if (!isTauri()) throw new Error("Heartbeat disponível apenas no aplicativo instalado.");
  return invoke<string>("heartbeat_local_device");
}

export async function getLocalStatus(): Promise<LocalStatus | null> {
  if (!isTauri()) return null;
  return invoke<LocalStatus>("local_status");
}

export async function getBackendStatus(signal?: AbortSignal): Promise<BackendStatus | null> {
  // Developer control plane only; never expose an unauthenticated remote endpoint.
  const response = await fetch("http://127.0.0.1:43821/healthz", {
    method: "GET",
    signal,
  });
  if (!response.ok) throw new Error("Control plane indisponível");
  const payload: unknown = await response.json();
  if (!payload || typeof payload !== "object") return null;
  if (!("status" in payload) || payload.status !== "ok") return null;
  if (!("production_ready" in payload) || typeof payload.production_ready !== "boolean") return null;
  return payload as BackendStatus;
}

export interface SessionReceipt {
  session_id: string;
  status: string;
}

export interface PendingInvitation {
  session_id: string;
  operator: string;
  requested: string[];
  expires_at_ms: number;
}

export interface ActiveAuthorization {
  session_id: string;
  operator: string;
  granted: string[];
  expires_at_ms: number;
}

export type SessionState = "awaiting_consent" | "active" | "rejected" | "ended";

/** Restricted to the native desktop; no private keys or raw signed URLs enter JS. */
export async function requestSupport(targetId: string): Promise<SessionReceipt> {
  if (!isTauri()) throw new Error("Disponível somente no aplicativo Seven Desk.");
  return invoke<SessionReceipt>("create_session_request", { targetId });
}

export async function pendingInvitations(): Promise<PendingInvitation[]> {
  if (!isTauri()) return [];
  return invoke<PendingInvitation[]>("list_pending_invitations");
}

export async function activeAuthorizations(): Promise<ActiveAuthorization[]> {
  if (!isTauri()) return [];
  return invoke<ActiveAuthorization[]>("list_active_authorizations");
}

export async function decideInvitation(sessionId: string, approve: boolean): Promise<SessionReceipt> {
  if (!isTauri()) throw new Error("É necessário usar o aplicativo instalado.");
  return invoke<SessionReceipt>("decide_session", { sessionId, approve });
}

export async function readSessionState(sessionId: string): Promise<SessionState> {
  if (!isTauri()) throw new Error("É necessário usar o aplicativo instalado.");
  const status = await invoke<string>("session_status", { sessionId });
  if (status === "awaiting_consent" || status === "active" || status === "rejected" || status === "ended") {
    return status;
  }
  throw new Error("Resposta de sessão não reconhecida.");
}

export async function terminateSession(sessionId: string): Promise<SessionReceipt> {
  if (!isTauri()) throw new Error("É necessário usar o aplicativo instalado.");
  return invoke<SessionReceipt>("end_session", { sessionId });
}
