import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface LocalStatus {
  app_name: string;
  app_version: string;
  operating_system: string;
  architecture: string;
  device_id: string | null;
  host_service_available: boolean;
  native_input_available: boolean;
}

export interface BackendStatus {
  status: "ok";
  release: string;
  protocol: number;
  production_ready: boolean;
}

export interface RelayStatus {
  status: "ok";
  transport: "signed-http-relay";
  production_ready: boolean;
}

export async function getRelayStatus(): Promise<RelayStatus | null> {
  if (!isTauri()) return null;
  return invoke<RelayStatus>("relay_status");
}

export interface ServerConfiguration {
  url: string;
  environment_override: boolean;
}

export interface ProxyConfiguration {
  mode: "direct" | "system" | "manual";
  url: string;
  environment_override: boolean;
}

export async function getProxyConfiguration(): Promise<ProxyConfiguration | null> {
  if (!isTauri()) return null;
  return invoke<ProxyConfiguration>("proxy_configuration");
}

export async function saveProxyConfiguration(mode: ProxyConfiguration["mode"], url: string): Promise<ProxyConfiguration> {
  if (!isTauri()) throw new Error("Configuração disponível apenas no aplicativo instalado.");
  return invoke<ProxyConfiguration>("save_proxy_configuration", { mode, url });
}

export async function validateProxyConfiguration(mode: ProxyConfiguration["mode"], url: string): Promise<ProxyConfiguration> {
  if (!isTauri()) throw new Error("Configuração disponível apenas no aplicativo instalado.");
  return invoke<ProxyConfiguration>("validate_proxy_configuration", { mode, url });
}

export async function getAutostartStatus(): Promise<boolean> {
  if (!isTauri()) return false;
  return invoke<boolean>("autostart_status");
}

export async function setAutostart(enabled: boolean): Promise<boolean> {
  if (!isTauri()) throw new Error("Inicialização automática disponível apenas no aplicativo instalado.");
  return invoke<boolean>("set_autostart", { enabled });
}

export async function getServerConfiguration(): Promise<ServerConfiguration | null> {
  if (!isTauri()) return null;
  return invoke<ServerConfiguration>("server_configuration");
}

export async function saveServerConfiguration(url: string): Promise<ServerConfiguration> {
  if (!isTauri()) throw new Error("Configuração disponível apenas no aplicativo instalado.");
  return invoke<ServerConfiguration>("save_server_configuration", { url });
}

export async function validateServerConfiguration(url: string): Promise<string> {
  if (!isTauri()) throw new Error("Configuração disponível apenas no aplicativo instalado.");
  return invoke<string>("validate_server_configuration", { url });
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

export async function getBackendStatus(_signal?: AbortSignal): Promise<BackendStatus | null> {
  if (!isTauri()) return null;
  // Use native Rust HTTPS certificate validation. No browser-side CORS bypass.
  return invoke<BackendStatus>("backend_status");
}

/** Presence is authenticated by the native key; a contact is not an authorization. */
export async function checkDevicePresence(deviceId: string): Promise<boolean> {
  if (!isTauri()) throw new Error("Consulta disponível apenas no Seven Desk instalado.");
  return invoke<boolean>("check_device_presence", { deviceId });
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

export interface ActiveOperatorSession {
  session_id: string;
  host: string;
  granted: string[];
  expires_at_ms: number;
}

export interface ChatMessage {
  sequence: number;
  sender: string;
  text: string;
  sent_at_ms: number;
}

export async function sendSessionChat(sessionId: string, text: string): Promise<ChatMessage> {
  if (!isTauri()) throw new Error("Chat disponível apenas no aplicativo instalado.");
  return invoke<ChatMessage>("send_session_chat", { sessionId, text });
}

export async function revokeSessionChat(sessionId: string): Promise<void> {
  if (!isTauri()) throw new Error("Revogação disponível apenas no aplicativo instalado.");
  return invoke<void>("revoke_session_chat", { sessionId });
}

export async function listSessionChat(sessionId: string): Promise<ChatMessage[]> {
  if (!isTauri()) return [];
  return invoke<ChatMessage[]>("list_session_chat", { sessionId });
}

export interface HostMonitor {
  id: number;
  name: string;
  width: number;
  height: number;
  is_primary: boolean;
}

export async function listHostMonitors(): Promise<HostMonitor[]> {
  if (!isTauri()) return [];
  return invoke<HostMonitor[]>("list_host_monitors");
}

export async function selectHostMonitor(monitorId: number): Promise<HostMonitor> {
  if (!isTauri()) throw new Error("Seleção de monitores disponível apenas no aplicativo instalado.");
  return invoke<HostMonitor>("select_host_monitor", { monitorId });
}

export type SessionState = "awaiting_consent" | "active" | "rejected" | "ended";

/** Restricted to the native desktop; no private keys or raw signed URLs enter JS. */
export async function requestSupport(targetId: string, requestMouse = false, requestKeyboard = false, requestChat = false): Promise<SessionReceipt> {
  if (!isTauri()) throw new Error("Disponível somente no aplicativo Seven Desk.");
  return invoke<SessionReceipt>("create_session_request", { targetId, requestMouse, requestKeyboard, requestChat });
}

export async function pendingInvitations(): Promise<PendingInvitation[]> {
  if (!isTauri()) return [];
  return invoke<PendingInvitation[]>("list_pending_invitations");
}

export async function activeAuthorizations(): Promise<ActiveAuthorization[]> {
  if (!isTauri()) return [];
  return invoke<ActiveAuthorization[]>("list_active_authorizations");
}

export async function activeOperatorSessions(): Promise<ActiveOperatorSession[]> {
  if (!isTauri()) return [];
  return invoke<ActiveOperatorSession[]>("list_active_operator_sessions");
}

export async function decideInvitation(
  sessionId: string,
  approve: boolean,
  grantMouse = false,
  grantKeyboard = false,
  grantChat = false,
): Promise<SessionReceipt> {
  if (!isTauri()) throw new Error("É necessário usar o aplicativo instalado.");
  return invoke<SessionReceipt>("decide_session", { sessionId, approve, grantMouse, grantKeyboard, grantChat });
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

/** Explicit host control: server stops accepting and clears screen pixels. */
export async function setScreenDelivery(sessionId: string, enabled: boolean): Promise<void> {
  if (!isTauri()) throw new Error("Compartilhamento disponível apenas no aplicativo instalado.");
  return invoke<void>("set_screen_delivery", { sessionId, enabled });
}

/** Windows host: capture only after explicit user click and host consent. */
export async function publishManualFrame(sessionId: string): Promise<void> {
  if (!isTauri()) throw new Error("Captura disponível apenas no aplicativo instalado.");
  return invoke<void>("publish_manual_screen_frame", { sessionId });
}

export async function startHostStreamCapture(sessionId: string): Promise<void> {
  if (!isTauri()) throw new Error("Captura disponível apenas no aplicativo instalado.");
  return invoke<void>("start_host_stream_capture", { sessionId });
}

export async function stopHostStreamCapture(sessionId: string): Promise<void> {
  if (!isTauri()) return;
  return invoke<void>("stop_host_stream_capture", { sessionId });
}

export async function onHostStreamStopped(
  callback: (event: { session_id: string; message: string }) => void
): Promise<() => void> {
  if (!isTauri()) return () => {};
  return listen<{ session_id: string; message: string }>("seven:stream-stopped", (event) => {
    callback(event.payload);
  });
}

export async function onHostInputStopped(
  callback: (event: { session_id: string; kind: "mouse" | "keyboard"; message: string }) => void
): Promise<() => void> {
  if (!isTauri()) return () => {};
  return listen<{ session_id: string; kind: "mouse" | "keyboard"; message: string }>(
    "seven:host-input-stopped", (event) => callback(event.payload)
  );
}

/** Signed, authorized retrieval. Pixels never come from an unauthenticated URL. */
export async function getAuthorizedFrame(sessionId: string): Promise<string> {
  if (!isTauri()) throw new Error("Visualização disponível apenas no aplicativo instalado.");
  return invoke<string>("fetch_authorized_screen_frame", { sessionId });
}

/** Mouse events are bounded to the main display. */
export type RemoteMouseEvent =
  | { type: "move"; x: number; y: number }
  | { type: "click"; x: number; y: number; button: "left" | "right" | "middle" }
  | { type: "scroll"; x: number; y: number; steps: number };

export async function sendRemoteMouse(sessionId: string, event: RemoteMouseEvent): Promise<void> {
  if (!isTauri()) throw new Error("Controle de mouse disponível apenas no aplicativo instalado.");
  return invoke<void>("send_remote_mouse", { sessionId, event });
}

export async function configureHostMouse(sessionId: string, enabled: boolean): Promise<void> {
  if (!isTauri()) throw new Error("Controle de mouse disponível apenas no aplicativo instalado.");
  return invoke<void>("configure_host_mouse", { sessionId, enabled });
}

export async function pollHostMouse(sessionId: string): Promise<number> {
  if (!isTauri()) throw new Error("Controle de mouse disponível apenas no aplicativo instalado.");
  return invoke<number>("poll_host_mouse", { sessionId });
}

/** Printable characters and a small allowlist of unmodified editing keys. */
export type RemoteKeyboardEvent =
  | { type: "character"; value: string }
  | { type: "special"; key: "backspace" | "delete" | "enter" | "tab" | "escape" |
      "arrow_left" | "arrow_right" | "arrow_up" | "arrow_down" };

export async function sendRemoteKeyboard(sessionId: string, event: RemoteKeyboardEvent): Promise<void> {
  if (!isTauri()) throw new Error("Controle de teclado disponível apenas no aplicativo instalado.");
  return invoke<void>("send_remote_keyboard", { sessionId, event });
}

export async function configureHostKeyboard(sessionId: string, enabled: boolean): Promise<void> {
  if (!isTauri()) throw new Error("Controle de teclado disponível apenas no aplicativo instalado.");
  return invoke<void>("configure_host_keyboard", { sessionId, enabled });
}

export async function pollHostKeyboard(sessionId: string): Promise<number> {
  if (!isTauri()) throw new Error("Controle de teclado disponível apenas no aplicativo instalado.");
  return invoke<number>("poll_host_keyboard", { sessionId });
}
