import { useEffect, useRef, useState, type KeyboardEvent as ReactKeyboardEvent, type PointerEvent as ReactPointerEvent, type WheelEvent as ReactWheelEvent } from "react";
import { DeviceDirectory } from "./DeviceDirectory";
import { isTauri } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { emitTo, listen } from "@tauri-apps/api/event";
import { INVITATION_ACTION, INVITATION_READY, INVITATION_STATE, INVITATION_WINDOW, type InvitationAction, type InvitationState } from "./invitationEvents";
import {
  getBackendStatus,
  getRelayStatus,
  getServerConfiguration,
  getProxyConfiguration,
  saveProxyConfiguration,
  validateProxyConfiguration,
  getAutostartStatus,
  setAutostart,
  validateServerConfiguration,
  saveServerConfiguration,
  getLocalStatus,
  heartbeatLocalDevice,
  initializeIdentity,
  registerLocalDevice,
  requestSupport,
  pendingInvitations,
  activeAuthorizations,
  activeOperatorSessions,
  decideInvitation,
  readSessionState,
  terminateSession,
  publishManualFrame,
  startHostStreamCapture,
  stopHostStreamCapture,
  onHostStreamStopped,
  onHostInputStopped,
  getAuthorizedFrame,
  setScreenDelivery,
  sendRemoteMouse,
  configureHostMouse,
  sendRemoteKeyboard,
  configureHostKeyboard,
  sendSessionChat,
  listSessionChat,
  revokeSessionChat,
  listHostMonitors,
  selectHostMonitor,
  type RemoteMouseEvent,
  type RemoteKeyboardEvent,
  type LocalStatus,
  type SessionReceipt,
  type PendingInvitation,
  type ActiveAuthorization,
  type ActiveOperatorSession,
  type ChatMessage,
  type HostMonitor,
  type ServerConfiguration,
  type ProxyConfiguration,
} from "./bridge";

type Page = "inicio" | "dispositivos" | "configuracoes";
type Theme = "dark" | "light";

const navigation: Array<{ id: Page; label: string; icon: string }> = [
  { id: "inicio", label: "Início", icon: "⌂" },
  { id: "dispositivos", label: "Dispositivos", icon: "▣" },
  { id: "configuracoes", label: "Configurações", icon: "⚙" },
];


export function App() {
  const [page, setPage] = useState<Page>("inicio");
  const [theme, setTheme] = useState<Theme>(() =>
    window.localStorage.getItem("seven-desk-theme") === "light" ? "light" : "dark"
  );
  const [status, setStatus] = useState<LocalStatus | null>(null);
  const [backendReady, setBackendReady] = useState(false);
  const [relayReady, setRelayReady] = useState(false);
  const [target, setTarget] = useState("");
  const [requestMouse, setRequestMouse] = useState(true);
  const [requestKeyboard, setRequestKeyboard] = useState(true);
  const [requestChat, setRequestChat] = useState(false);
  const [chatSessionId, setChatSessionId] = useState<string | null>(null);
  const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
  const [chatDraft, setChatDraft] = useState("");
  const [chatBusy, setChatBusy] = useState(false);
  const [revokingChat, setRevokingChat] = useState<string | null>(null);
  const [monitors, setMonitors] = useState<HostMonitor[]>([]);
  const [monitorError, setMonitorError] = useState("");
  const [selectedMonitorId, setSelectedMonitorId] = useState<number | null>(null);
  const [monitorBusy, setMonitorBusy] = useState(false);
  const [serverConfiguration, setServerConfiguration] = useState<ServerConfiguration | null>(null);
  const [serverUrlDraft, setServerUrlDraft] = useState("http://127.0.0.1:43821");
  const [serverSaving, setServerSaving] = useState(false);
  const [proxyConfiguration, setProxyConfiguration] = useState<ProxyConfiguration | null>(null);
  const [proxyModeDraft, setProxyModeDraft] = useState<ProxyConfiguration["mode"]>("direct");
  const [proxyUrlDraft, setProxyUrlDraft] = useState("");
  const [proxySaving, setProxySaving] = useState(false);
  const [networkMessage, setNetworkMessage] = useState("");
  const [autostartEnabled, setAutostartEnabled] = useState(false);
  const [autostartBusy, setAutostartBusy] = useState(false);
  const [autostartMessage, setAutostartMessage] = useState("");
  const [viewerMouseActive, setViewerMouseActive] = useState(false);
  const [viewerKeyboardActive, setViewerKeyboardActive] = useState(false);
  const [hostMouseSession, setHostMouseSession] = useState<string | null>(null);
  const [hostKeyboardSession, setHostKeyboardSession] = useState<string | null>(null);
  const [mouseBusy, setMouseBusy] = useState(false);
  const [keyboardBusy, setKeyboardBusy] = useState(false);
  const lastMouseMovement = useRef(0);
  const lastMouseWheel = useRef(0);
  const inputSendBusy = useRef(false);
  const lastMouseWireAt = useRef(0);
  const mouseSendChain = useRef<Promise<void>>(Promise.resolve());
  const viewerMouseEnabledRef = useRef(false);
  const viewerKeyboardEnabledRef = useRef(false);
  const keyboardSendChain = useRef<Promise<void>>(Promise.resolve());
  const lastKeyboardWireAt = useRef(0);
  const pendingKeyboardEvents = useRef(0);
  const viewerImageRef = useRef<HTMLImageElement | null>(null);
  const minimizeAfterCapture = useRef<{ sessionId: string; captureReady: boolean; inputReady: boolean } | null>(null);
  const autoViewingSession = useRef<string | null>(null);
  const autoInputSession = useRef<string | null>(null);
  const [, setFeedback] = useState("");
  const [invitationError, setInvitationError] = useState("");
  const [busy, setBusy] = useState(true);
  const [registering, setRegistering] = useState(false);
  const [presenceActive, setPresenceActive] = useState(false);
  const [requesting, setRequesting] = useState(false);
  const [outboundSession, setOutboundSession] = useState<SessionReceipt | null>(null);
  const [activeSessions, setActiveSessions] = useState<ActiveAuthorization[]>([]);
  const [operatorSessions, setOperatorSessions] = useState<ActiveOperatorSession[]>([]);
  const [inbox, setInbox] = useState<PendingInvitation[]>([]);
  const [decisionBusy, setDecisionBusy] = useState<string | null>(null);
  const [hostDialogSession, setHostDialogSession] = useState<string | null>(null);
  const [pendingAutoShareSession, setPendingAutoShareSession] = useState<string | null>(null);
  const invitationActionRef = useRef<(action: InvitationAction) => void>(() => {});
  const [ending, setEnding] = useState(false);
  const [frameBusy, setFrameBusy] = useState(false);
  const [sharingBusy, setSharingBusy] = useState(false);
  const [lastFrame, setLastFrame] = useState<string | null>(null);
  const [streamingSession, setStreamingSession] = useState<string | null>(null);
  const [viewingSession, setViewingSession] = useState<string | null>(null);

  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    window.localStorage.setItem("seven-desk-theme", theme);
  }, [theme]);

  function markReadyToMinimize(sessionId: string, part: "captureReady" | "inputReady"): void {
    const pending = minimizeAfterCapture.current;
    if (!pending || pending.sessionId !== sessionId) return;
    pending[part] = true;
    if (!pending.captureReady || !pending.inputReady) return;
    minimizeAfterCapture.current = null;
    if (isTauri()) {
      void WebviewWindow.getByLabel(INVITATION_WINDOW).then((window) => window?.minimize()).catch(() => {
        setFeedback("Conexão ativa. Minimize a janela de atendimento manualmente se desejar.");
      });
    }
  }

  useEffect(() => {
    let mounted = true;
    const controller = new AbortController();

    async function check() {
      try {
        let local = await getLocalStatus();
        if (local && !local.device_id) {
          await initializeIdentity();
          local = await getLocalStatus();
        }
        if (mounted) setStatus(local);
      } catch {
        if (mounted) setFeedback("Não foi possível ler as informações do aplicativo.");
      } finally {
        if (mounted) setBusy(false);
      }
      try {
        const api = await getBackendStatus(controller.signal);
        if (mounted) setBackendReady(api !== null);
      } catch {
        if (mounted) setBackendReady(false);
      }
    }
    void check();
    const timer = window.setInterval(() => {
      void getBackendStatus(controller.signal).then((api) => {
        if (mounted) setBackendReady(api !== null);
      }).catch(() => { if (mounted) setBackendReady(false); });
    }, 10_000);
    return () => { mounted = false; controller.abort(); window.clearInterval(timer); };
  }, []);

  useEffect(() => {
    let mounted = true;
    const check = () => { void getRelayStatus().then((value) => { if (mounted) setRelayReady(value !== null); })
      .catch(() => { if (mounted) setRelayReady(false); }); };
    check();
    const timer = window.setInterval(check, 10_000);
    return () => { mounted = false; window.clearInterval(timer); };
  }, [serverConfiguration?.url, proxyConfiguration?.mode, proxyConfiguration?.url]);

  useEffect(() => {
    if (!status || !["windows", "macos", "linux"].includes(status.operating_system)) return;
    let mounted = true;
    void listHostMonitors().then((found) => {
      if (mounted) {
        setMonitors(found);
        setMonitorError(found.length === 0 ? "Nenhum monitor disponível para captura nesta sessão." : "");
        setSelectedMonitorId(found.find((item) => item.is_primary)?.id ?? found[0]?.id ?? null);
      }
    }).catch((error) => { if (mounted) { setMonitors([]); setMonitorError(String(error)); } });
    return () => { mounted = false; };
  }, [status?.operating_system]);

  useEffect(() => {
    let mounted = true;
    void getServerConfiguration().then((configuration) => {
      if (mounted && configuration) {
        setServerConfiguration(configuration);
        setServerUrlDraft(configuration.url);
      }
    }).catch(() => {});
    void getProxyConfiguration().then((configuration) => {
      if (mounted && configuration) {
        setProxyConfiguration(configuration);
        setProxyModeDraft(configuration.mode);
        setProxyUrlDraft(configuration.url);
      }
    }).catch(() => {});
    void getAutostartStatus().then((enabled) => { if (mounted) setAutostartEnabled(enabled); }).catch(() => {});
    return () => { mounted = false; };
  }, []);

  // Register the running app automatically; an interrupted heartbeat expires
  // server-side after the configured TTL and never enables remote input.
  useEffect(() => {
    if (!presenceActive) return;
    const timer = window.setInterval(() => {
      void heartbeatLocalDevice().catch(() => {
        setPresenceActive(false);
        setFeedback("Conexão com o servidor interrompida. Tentando reconectar automaticamente.");
      });
    }, 30_000);
    return () => window.clearInterval(timer);
  }, [presenceActive]);

  useEffect(() => {
    if (!status?.device_id || !backendReady || presenceActive || serverSaving || proxySaving) return;
    let stopped = false;
    let inFlight = false;
    async function register(): Promise<void> {
      if (stopped || inFlight) return;
      inFlight = true;
      setRegistering(true);
      try {
        const registeredId = await registerLocalDevice();
        if (registeredId !== status?.device_id) throw new Error("A identidade registrada não corresponde a este dispositivo.");
        if (!stopped) setPresenceActive(true);
      } catch {
        // The next retry handles a temporarily unavailable server.
      } finally {
        inFlight = false;
        if (!stopped) setRegistering(false);
      }
    }
    void register();
    const timer = window.setInterval(() => void register(), 10_000);
    return () => { stopped = true; window.clearInterval(timer); };
  }, [status?.device_id, backendReady, presenceActive, serverSaving, proxySaving, serverConfiguration?.url, proxyConfiguration?.mode, proxyConfiguration?.url]);

  // Only the authenticated device can retrieve its own outstanding requests.
  // Requests are refreshed while this device is registered and online.
  useEffect(() => {
    if (!presenceActive || !backendReady) {
      setInbox([]);
      setActiveSessions([]);
      setOperatorSessions([]);
      return;
    }
    let stopped = false;
    let fetching = false;
    async function refresh(): Promise<void> {
      if (fetching) return;
      fetching = true;
      try {
        const [received, authorizations, operatorGrants] = await Promise.all([
          pendingInvitations(),
          activeAuthorizations(),
          activeOperatorSessions(),
        ]);
        if (!stopped) {
          setInbox(received);
          setActiveSessions(authorizations);
          setOperatorSessions(operatorGrants);
        }
      } catch {
        if (!stopped) {
          setInbox([]);
          setActiveSessions([]);
          setOperatorSessions([]);
        }
      } finally {
        fetching = false;
      }
    }
    void refresh();
    const timer = window.setInterval(() => void refresh(), 4_000);
    return () => { stopped = true; window.clearInterval(timer); };
  }, [presenceActive, backendReady]);

  useEffect(() => {
    if (!inbox[0] || !isTauri()) return;
    void (async () => {
      const window = await WebviewWindow.getByLabel(INVITATION_WINDOW);
      if (!window) throw new Error("Janela de atendimento indisponível");
      await window.show();
      await window.unminimize();
      await window.setFocus();
    })().catch(() => {
      setFeedback("Há um pedido de conexão. Abra a janela de atendimento do Seven Desk.");
    });
  }, [inbox[0]?.session_id]);

  useEffect(() => {
    if (!chatSessionId || !presenceActive) return;
    let stopped = false;
    let polling = false;
    async function poll(): Promise<void> {
      if (polling || !chatSessionId) return;
      polling = true;
      try {
        const history = await listSessionChat(chatSessionId);
        if (!stopped) setChatMessages(history);
      } catch {
        if (!stopped) {
          setChatSessionId(null);
          setChatMessages([]);
          setFeedback("Chat interrompido: sessão ou permissão indisponível.");
        }
      } finally {
        polling = false;
      }
    }
    void poll();
    const timer = window.setInterval(() => void poll(), 2_000);
    return () => { stopped = true; window.clearInterval(timer); };
  }, [chatSessionId, presenceActive]);

  useEffect(() => {
    if (!outboundSession || outboundSession.status === "rejected" || outboundSession.status === "ended") return;
    const sessionId = outboundSession.session_id;
    let stopped = false;
    async function refresh(): Promise<void> {
      try {
        const state = await readSessionState(sessionId);
        if (!stopped) {
          setOutboundSession((previous) =>
            previous?.session_id === sessionId ? { ...previous, status: state } : previous
          );
        }
      } catch {
        // The server may have restarted: never show an inferred approval.
      }
    }
    const timer = window.setInterval(() => void refresh(), 3_000);
    return () => { stopped = true; window.clearInterval(timer); };
  }, [outboundSession?.session_id, outboundSession?.status]);

  // Explicit host-authorized capture continues after minimizing the window.
  // Server revocation and the visible Finalizar action stop it.
  useEffect(() => {
    if (!streamingSession || !presenceActive || !backendReady) return;
    const id = streamingSession;
    let cancelled = false;
    let unlisten: (() => void) | null = null;
    async function start(): Promise<void> {
      try {
        unlisten = await onHostStreamStopped((event) => {
          if (!cancelled && event.session_id === id) {
            void stopSharing(id).finally(() => setFeedback(event.message));
          }
        });
        if (cancelled) return;
        await startHostStreamCapture(id);
        if (cancelled) await stopHostStreamCapture(id);
        else markReadyToMinimize(id, "captureReady");
      } catch (error: unknown) {
        if (!cancelled) {
          const message = error instanceof Error ? error.message : "Compartilhamento interrompido.";
          void stopSharing(id).finally(() => setFeedback(message));
        }
      }
    }
    void start();
    return () => {
      cancelled = true;
      unlisten?.();
      void stopHostStreamCapture(id);
    };
  }, [streamingSession, presenceActive, backendReady]);

  // The operator opts in separately. An expired/missing frame clears the image,
  // while revocation and network errors stop the viewer completely.
  useEffect(() => {
    if (!viewingSession || outboundSession?.session_id !== viewingSession || outboundSession.status !== "active") return;
    const id = viewingSession;
    let cancelled = false;
    let busyFrame = false;
    async function refreshFrame(): Promise<void> {
      if (cancelled || busyFrame) return;
      busyFrame = true;
      try {
        const next = await getAuthorizedFrame(id);
        if (!cancelled) setLastFrame(next);
      } catch (error: unknown) {
        if (!cancelled) {
          setLastFrame(null);
          const message = error instanceof Error ? error.message : "Falha ao consultar a tela.";
          // No frame has been published recently: remain ready for a new frame.
          if (!message.includes("HTTP 404")) {
            setViewingSession(null);
            setFeedback(message);
          }
        }
      } finally {
        busyFrame = false;
      }
    }
    void refreshFrame();
    const timer = window.setInterval(() => void refreshFrame(), 67);
    return () => { cancelled = true; window.clearInterval(timer); };
  }, [viewingSession, outboundSession?.session_id, outboundSession?.status]);

  useEffect(() => {
    if (!presenceActive || !backendReady) {
      if (streamingSession) void stopSharing(streamingSession);
      setViewingSession(null);
      setLastFrame(null);
    }
  }, [presenceActive, backendReady]);

  useEffect(() => {
    if (outboundSession?.status !== "active") {
      setViewingSession(null);
      setLastFrame(null);
      autoViewingSession.current = null;
      autoInputSession.current = null;
    }
  }, [outboundSession?.status]);

  useEffect(() => {
    if (!outboundSession || outboundSession.status !== "active") return;
    if (!operatorSessions.some((item) => item.session_id === outboundSession.session_id)) return;
    if (autoViewingSession.current === outboundSession.session_id) return;
    autoViewingSession.current = outboundSession.session_id;
    setViewingSession(outboundSession.session_id);
  }, [outboundSession?.session_id, outboundSession?.status, operatorSessions]);

  useEffect(() => {
    if (!lastFrame || !viewingSession || outboundSession?.session_id !== viewingSession || autoInputSession.current === viewingSession) return;
    const grant = operatorSessions.find((item) => item.session_id === viewingSession);
    if (!grant) return;
    autoInputSession.current = viewingSession;
    setViewerMouseActive(grant.granted.includes("input"));
    setViewerKeyboardActive(grant.granted.includes("keyboard"));
    if (grant.granted.includes("keyboard")) window.requestAnimationFrame(() => viewerImageRef.current?.focus());
  }, [lastFrame, viewingSession, outboundSession?.session_id, operatorSessions]);

  useEffect(() => {
    if (!lastFrame || !viewingSession || outboundSession?.session_id !== viewingSession) {
      setViewerKeyboardActive(false);
    }
  }, [lastFrame, viewingSession, outboundSession?.session_id]);

  useEffect(() => {
    if (streamingSession && !activeSessions.some((session) => session.session_id === streamingSession)) {
      void stopSharing(streamingSession);
    }
  }, [activeSessions, streamingSession]);

  useEffect(() => {
    if (!pendingAutoShareSession || !activeSessions.some((item) => item.session_id === pendingAutoShareSession)) return;
    const sessionId = pendingAutoShareSession;
    setPendingAutoShareSession(null);
    minimizeAfterCapture.current = { sessionId, captureReady: false, inputReady: false };
    void toggleSharing(sessionId);
  }, [pendingAutoShareSession, activeSessions]);

  // Native input receivers keep working while the consent window is
  // minimized. The local gates still close on stop or authorization loss.
  useEffect(() => {
    if (!hostMouseSession) return;
    const id = hostMouseSession;
    if (!presenceActive || !backendReady || streamingSession !== id) {
      setHostMouseSession(null);
      void configureHostMouse(id, false);
    } else {
      return () => { void configureHostMouse(id, false); };
    }
  }, [hostMouseSession, streamingSession, presenceActive, backendReady]);

  useEffect(() => {
    if (!hostKeyboardSession) return;
    const id = hostKeyboardSession;
    if (!presenceActive || !backendReady || streamingSession !== id) {
      setHostKeyboardSession(null);
      void configureHostKeyboard(id, false);
    } else {
      return () => { void configureHostKeyboard(id, false); };
    }
  }, [hostKeyboardSession, streamingSession, presenceActive, backendReady]);

  useEffect(() => {
    let stopped = false;
    let unlisten: (() => void) | null = null;
    void onHostInputStopped((event) => {
      if (stopped) return;
      if (event.kind === "mouse") setHostMouseSession((current) => current === event.session_id ? null : current);
      if (event.kind === "keyboard") setHostKeyboardSession((current) => current === event.session_id ? null : current);
      setFeedback(`${event.kind === "mouse" ? "Mouse" : "Teclado"} remoto interrompido: ${event.message}`);
    }).then((cleanup) => { if (stopped) cleanup(); else unlisten = cleanup; }).catch(() => {});
    return () => { stopped = true; unlisten?.(); };
  }, []);

  useEffect(() => {
    viewerMouseEnabledRef.current = viewerMouseActive;
  }, [viewerMouseActive]);

  useEffect(() => {
    viewerKeyboardEnabledRef.current = viewerKeyboardActive;
  }, [viewerKeyboardActive]);

  useEffect(() => {
    if (!outboundSession || outboundSession.status !== "active" ||
        !operatorSessions.some((item) =>
          item.session_id === outboundSession.session_id && item.granted.includes("input")
        )) {
      setViewerMouseActive(false);
    }
  }, [outboundSession?.session_id, outboundSession?.status, operatorSessions]);

  useEffect(() => {
    if (!outboundSession || outboundSession.status !== "active" ||
        !operatorSessions.some((item) =>
          item.session_id === outboundSession.session_id && item.granted.includes("keyboard")
        )) {
      setViewerKeyboardActive(false);
    }
  }, [outboundSession?.session_id, outboundSession?.status, operatorSessions]);

  async function sendInvitation(): Promise<void> {
    if (!presenceActive || requesting) return;
    setRequesting(true);
    try {
      const result = await requestSupport(target.trim(), requestMouse, requestKeyboard, requestChat);
      setOutboundSession(result);
      setFeedback("Solicitação de negociação enviada. Aguarde a autorização no host.");
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Falha ao solicitar sessão.");
    } finally {
      setRequesting(false);
    }
  }

  async function respondToInvitation(sessionId: string, approve: boolean, selected?: { mouse: boolean; keyboard: boolean; chat: boolean }): Promise<void> {
    if (decisionBusy) return;
    setDecisionBusy(sessionId);
    setInvitationError("");
    try {
      const invitation = inbox.find((item) => item.session_id === sessionId);
      if (!invitation) throw new Error("Este pedido não está mais disponível.");
      const grantMouse = approve && invitation.requested.includes("input") && (selected?.mouse ?? true) && Boolean(status?.native_input_available);
      const grantKeyboard = approve && invitation.requested.includes("keyboard") && (selected?.keyboard ?? true) && Boolean(status?.native_input_available);
      const grantChat = approve && invitation.requested.includes("chat") && (selected?.chat ?? true);
      await decideInvitation(sessionId, approve, grantMouse, grantKeyboard, grantChat);
      setInbox((previous) => previous.filter((item) => item.session_id !== sessionId));
      if (approve) {
        try {
          setActiveSessions(await activeAuthorizations());
        } catch {
          // The normal presence refresh can recover the signed grant.
        }
        setHostDialogSession(sessionId);
        if (grantChat) openChat(sessionId);
        setPendingAutoShareSession(sessionId);
      }
      setFeedback(approve
        ? "Conexão aceita. Iniciando o compartilhamento com as permissões escolhidas."
        : "Solicitação recusada com sucesso.");
    } catch (error: unknown) {
      setInvitationError(error instanceof Error ? error.message : "Não foi possível responder ao convite.");
    } finally {
      setDecisionBusy(null);
    }
  }

  async function stopSessionChat(sessionId: string): Promise<void> {
    if (revokingChat) return;
    setRevokingChat(sessionId);
    try {
      await revokeSessionChat(sessionId);
      setActiveSessions((current) => current.map((item) =>
        item.session_id === sessionId ? { ...item, granted: item.granted.filter((grant) => grant !== "chat") } : item
      ));
      setFeedback("Chat revogado. Novas mensagens e leituras foram bloqueadas.");
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Não foi possível revogar o chat.");
    } finally { setRevokingChat(null); }
  }

  async function closeSession(sessionId: string): Promise<void> {
    if (ending) return;
    setEnding(true);
    setInvitationError("");
    try {
      const result = await terminateSession(sessionId);
      if (outboundSession?.session_id === sessionId) {
        setOutboundSession(result);
        setViewingSession(null);
        setLastFrame(null);
      }
      if (streamingSession === sessionId) setStreamingSession(null);
      if (hostMouseSession === sessionId) {
        setHostMouseSession(null);
        void configureHostMouse(sessionId, false);
      }
      if (hostKeyboardSession === sessionId) {
        setHostKeyboardSession(null);
        void configureHostKeyboard(sessionId, false);
      }
      setActiveSessions((current) => current.filter((item) => item.session_id !== sessionId));
      if (hostDialogSession === sessionId) setHostDialogSession(null);
      setOperatorSessions((current) => current.filter((item) => item.session_id !== sessionId));
      if (chatSessionId === sessionId) {
        setChatSessionId(null);
        setChatMessages([]);
      }
      setFeedback("Negociação encerrada e permissões revogadas.");
    } catch (error: unknown) {
      setInvitationError(error instanceof Error ? error.message : "Não foi possível encerrar a sessão.");
    } finally {
      setEnding(false);
    }
  }

  async function stopSharing(sessionId: string): Promise<void> {
    if (minimizeAfterCapture.current?.sessionId === sessionId) minimizeAfterCapture.current = null;
    // Stop local capture immediately, even if the server is offline. The
    // backend independently deletes cached pixels when the signed stop arrives.
    setStreamingSession(null);
    try {
      await stopHostStreamCapture(sessionId);
    } catch {
      // Still disable delivery on the server if native IPC is interrupted.
    }
    if (hostMouseSession === sessionId) {
      setHostMouseSession(null);
      void configureHostMouse(sessionId, false);
    }
    if (hostKeyboardSession === sessionId) {
      setHostKeyboardSession(null);
      void configureHostKeyboard(sessionId, false);
    }
    try {
      await setScreenDelivery(sessionId, false);
      setFeedback("Compartilhamento parado e último quadro apagado do servidor.");
    } catch {
      setFeedback("Captura local interrompida. Não foi possível confirmar a limpeza no servidor; os quadros remotos expiram automaticamente.");
    }
  }

  async function toggleSharing(sessionId: string): Promise<void> {
    if (sharingBusy) return;
    if (streamingSession === sessionId) {
      await stopSharing(sessionId);
      return;
    }
    setSharingBusy(true);
    try {
      await setScreenDelivery(sessionId, true);
      setStreamingSession(sessionId);
      const grant = activeSessions.find((item) => item.session_id === sessionId);
      const inputErrors: string[] = [];
      if (grant?.granted.includes("input") && status?.native_input_available &&
          monitors.find((monitor) => monitor.id === selectedMonitorId)?.is_primary) {
        try {
          await configureHostMouse(sessionId, true);
          setHostMouseSession(sessionId);
        } catch (error: unknown) {
          inputErrors.push(`mouse: ${String(error)}`);
        }
      }
      if (grant?.granted.includes("keyboard") && status?.native_input_available) {
        try {
          await configureHostKeyboard(sessionId, true);
          setHostKeyboardSession(sessionId);
        } catch (error: unknown) {
          inputErrors.push(`teclado: ${String(error)}`);
        }
      }
      setFeedback(inputErrors.length
        ? `Tela ativa. Controle indisponível (${inputErrors.join("; ")}).`
        : "Compartilhamento e controles autorizados ativos. Use Parar para interromper imediatamente.");
      markReadyToMinimize(sessionId, "inputReady");
    } catch (error: unknown) {
      if (minimizeAfterCapture.current?.sessionId === sessionId) minimizeAfterCapture.current = null;
      setFeedback(error instanceof Error ? error.message : "Não foi possível ativar o compartilhamento.");
    } finally {
      setSharingBusy(false);
    }
  }

  async function toggleHostMouse(sessionId: string): Promise<void> {
    if (mouseBusy) return;
    setMouseBusy(true);
    try {
      if (hostMouseSession === sessionId) {
        setHostMouseSession(null);
        await configureHostMouse(sessionId, false);
        setFeedback("Controle de mouse desativado neste computador.");
      } else {
        if (streamingSession !== sessionId) {
          throw new Error("Inicie o compartilhamento de tela antes de habilitar o mouse.");
        }
        await configureHostMouse(sessionId, true);
        setHostMouseSession(sessionId);
        setFeedback("Controle de mouse remoto ativo. Use Parar mouse para bloquear imediatamente.");
      }
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Falha ao configurar mouse remoto.");
    } finally {
      setMouseBusy(false);
    }
  }

  async function toggleHostKeyboard(sessionId: string): Promise<void> {
    if (keyboardBusy) return;
    setKeyboardBusy(true);
    try {
      if (hostKeyboardSession === sessionId) {
        setHostKeyboardSession(null);
        await configureHostKeyboard(sessionId, false);
        setFeedback("Controle de teclado desativado neste computador.");
      } else {
        if (streamingSession !== sessionId) {
          throw new Error("Inicie o compartilhamento de tela antes de habilitar o teclado.");
        }
        await configureHostKeyboard(sessionId, true);
        setHostKeyboardSession(sessionId);
        setFeedback("Teclado remoto ativo. Use Parar teclado para bloquear imediatamente.");
      }
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Falha ao configurar teclado remoto.");
    } finally {
      setKeyboardBusy(false);
    }
  }

  function forwardViewerKey(event: ReactKeyboardEvent<HTMLImageElement>): void {
    if (!viewerKeyboardEnabledRef.current || !outboundSession || outboundSession.status !== "active" ||
        viewingSession !== outboundSession.session_id || !lastFrame ||
        event.ctrlKey || event.altKey || event.metaKey || event.nativeEvent.isComposing) return;
    const special: Record<string, RemoteKeyboardEvent> = {
      Backspace: { type: "special", key: "backspace" },
      Delete: { type: "special", key: "delete" },
      Enter: { type: "special", key: "enter" },
      Tab: { type: "special", key: "tab" },
      Escape: { type: "special", key: "escape" },
      ArrowLeft: { type: "special", key: "arrow_left" },
      ArrowRight: { type: "special", key: "arrow_right" },
      ArrowUp: { type: "special", key: "arrow_up" },
      ArrowDown: { type: "special", key: "arrow_down" },
    };
    const remote = special[event.key] ??
      (Array.from(event.key).length === 1 &&
        (/[\p{L}\p{N}]/u.test(event.key) || /^[\x20-\x7e]$/.test(event.key))
        ? { type: "character" as const, value: event.key } : null);
    if (!remote) return;
    event.preventDefault();
    if (pendingKeyboardEvents.current >= 32) return;
    pendingKeyboardEvents.current += 1;
    const sessionId = outboundSession.session_id;
    const send = keyboardSendChain.current.then(async () => {
      try {
        const remaining = Math.max(0, 35 - (Date.now() - lastKeyboardWireAt.current));
        if (remaining > 0) await new Promise<void>((resolve) => window.setTimeout(resolve, remaining));
        if (!viewerKeyboardEnabledRef.current) return;
        await sendRemoteKeyboard(sessionId, remote);
        lastKeyboardWireAt.current = Date.now();
      } finally {
        pendingKeyboardEvents.current -= 1;
      }
    });
    keyboardSendChain.current = send.catch(() => {});
    void send.catch((error: unknown) => {
      setViewerKeyboardActive(false);
      setFeedback(error instanceof Error ? error.message : "Controle do teclado interrompido.");
    });
  }

  async function submitViewerMouse(event: RemoteMouseEvent, motion: boolean): Promise<void> {
    if (!viewerMouseEnabledRef.current || !outboundSession || outboundSession.status !== "active") return;
    if (inputSendBusy.current && motion) return;
    if (motion) inputSendBusy.current = true;
    const sessionId = outboundSession.session_id;
    // Serialize calls so a click immediately following mouse movement cannot
    // trip the server's 20 ms rate limit or be reordered by the network.
    const send = mouseSendChain.current.then(async () => {
      const remaining = Math.max(0, 35 - (Date.now() - lastMouseWireAt.current));
      if (remaining > 0) {
        await new Promise<void>((resolve) => window.setTimeout(resolve, remaining));
      }
      if (!viewerMouseEnabledRef.current) return;
      await sendRemoteMouse(sessionId, event);
      lastMouseWireAt.current = Date.now();
    });
    mouseSendChain.current = send.catch(() => {});
    try {
      await send;
    } catch (error: unknown) {
      setViewerMouseActive(false);
      setFeedback(error instanceof Error ? error.message : "Controle do mouse interrompido.");
    } finally {
      if (motion) inputSendBusy.current = false;
    }
  }

  function imagePointer(event: ReactPointerEvent<HTMLImageElement> | ReactWheelEvent<HTMLImageElement>): { x: number; y: number } | null {
    const image = event.currentTarget;
    const rect = image.getBoundingClientRect();
    if (!image.naturalWidth || !image.naturalHeight || rect.width <= 0 || rect.height <= 0) return null;
    // object-fit: contain can letterbox; only the actual image content is interactive.
    const scale = Math.min(rect.width / image.naturalWidth, rect.height / image.naturalHeight);
    const shownWidth = image.naturalWidth * scale;
    const shownHeight = image.naturalHeight * scale;
    const left = rect.left + (rect.width - shownWidth) / 2;
    const top = rect.top + (rect.height - shownHeight) / 2;
    const relX = (event.clientX - left) / shownWidth;
    const relY = (event.clientY - top) / shownHeight;
    if (relX < 0 || relY < 0 || relX > 1 || relY > 1) return null;
    return { x: Math.round(relX * 10_000), y: Math.round(relY * 10_000) };
  }

  function forwardViewerPointer(
    event: ReactPointerEvent<HTMLImageElement>,
    kind: "move" | "click",
    button: "left" | "right" | "middle" = "left",
  ): void {
    if (!viewerMouseActive || (event.pointerType !== "mouse" && event.pointerType !== "pen")) return;
    const mapped = imagePointer(event);
    if (!mapped) return;
    if (kind === "move") {
      const time = Date.now();
      if (time - lastMouseMovement.current < 90) return;
      lastMouseMovement.current = time;
      void submitViewerMouse({ type: "move", ...mapped }, true);
    } else {
      void submitViewerMouse({ type: "click", ...mapped, button }, false);
    }
  }

  function forwardViewerWheel(event: ReactWheelEvent<HTMLImageElement>): void {
    if (!viewerMouseActive || event.deltaY === 0) return;
    const mapped = imagePointer(event);
    if (!mapped) return;
    event.preventDefault();
    const time = Date.now();
    if (time - lastMouseWheel.current < 90) return;
    lastMouseWheel.current = time;
    void submitViewerMouse({ type: "scroll", ...mapped, steps: Math.sign(event.deltaY) }, false);
  }

  async function shareManualFrame(sessionId: string): Promise<void> {
    if (frameBusy) return;
    setFrameBusy(true);
    try {
      // The host may have paused the previous stream; a fresh explicit click
      // is required before a new single-frame publication.
      await setScreenDelivery(sessionId, true);
      await publishManualFrame(sessionId);
      setFeedback("Quadro do monitor escolhido enviado após sua autorização. Nenhum streaming contínuo foi iniciado.");
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Não foi possível capturar e compartilhar o quadro.");
    } finally {
      setFrameBusy(false);
    }
  }

  async function submitChat(): Promise<void> {
    if (!chatSessionId || chatBusy) return;
    const text = chatDraft.trim();
    if (!text || new TextEncoder().encode(text).length > 2_048 || [...text].length > 500) {
      setFeedback("A mensagem deve ter até 500 caracteres e 2048 bytes.");
      return;
    }
    setChatBusy(true);
    try {
      const sent = await sendSessionChat(chatSessionId, text);
      setChatMessages((current) => current.some((message) => message.sequence === sent.sequence) ? current : [...current, sent].slice(-24));
      setChatDraft("");
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Falha ao enviar mensagem.");
    } finally {
      setChatBusy(false);
    }
  }

  async function chooseMonitor(monitorId: number): Promise<void> {
    if (monitorBusy) return;
    if (hostMouseSession) {
      setHostMouseSession(null);
      void configureHostMouse(hostMouseSession, false);
    }
    setMonitorBusy(true);
    try {
      if (streamingSession) await stopSharing(streamingSession);
      const selected = await selectHostMonitor(monitorId);
      setSelectedMonitorId(selected.id);
      setFeedback(selected.is_primary
        ? "Monitor principal selecionado. Reinicie o compartilhamento para aplicar a mudança."
        : "Monitor secundário selecionado. O mouse remoto fica bloqueado neste monitor.");
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Não foi possível selecionar o monitor.");
    } finally {
      setMonitorBusy(false);
    }
  }

  async function disconnectForNetworkChange(): Promise<boolean> {
      if (streamingSession) await stopSharing(streamingSession);
      if (hostMouseSession) {
        setHostMouseSession(null);
        await configureHostMouse(hostMouseSession, false);
      }
      if (hostKeyboardSession) {
        setHostKeyboardSession(null);
        await configureHostKeyboard(hostKeyboardSession, false);
      }
      setPresenceActive(false);
      const sessionsToEnd = new Set([
        ...activeSessions.map((item) => item.session_id),
        ...operatorSessions.map((item) => item.session_id),
        ...(outboundSession ? [outboundSession.session_id] : []),
      ]);
      const revocations = await Promise.allSettled(
        [...sessionsToEnd].map((sessionId) => terminateSession(sessionId))
      );
      const revocationFailed = revocations.some((result) => result.status === "rejected");
      setViewingSession(null);
      setLastFrame(null);
      setOutboundSession(null);
      setActiveSessions([]);
      setOperatorSessions([]);
      setInbox([]);
      setChatSessionId(null);
      setChatMessages([]);
      return revocationFailed;
  }

  async function saveServer(): Promise<void> {
    if (serverSaving || !serverConfiguration || serverConfiguration.environment_override) return;
    setServerSaving(true);
    setNetworkMessage("");
    try {
      const validatedUrl = await validateServerConfiguration(serverUrlDraft.trim());
      if (validatedUrl === serverConfiguration.url) return;
      const revocationFailed = await disconnectForNetworkChange();
      const saved = await saveServerConfiguration(validatedUrl);
      setServerConfiguration(saved);
      setServerUrlDraft(saved.url);
      try {
        const status = await getBackendStatus();
        setBackendReady(status !== null);
        setFeedback(status
          ? "Servidor salvo e acessível. Conectando este dispositivo automaticamente."
          : "Servidor salvo, mas ainda indisponível. Confira endereço, VPN e certificado TLS.");
        setNetworkMessage(status ? "Servidor conectado." : "Servidor salvo, mas indisponível. Confira endereço, VPN e certificado TLS.");
      } catch {
        setBackendReady(false);
        setFeedback("Servidor salvo, mas ainda indisponível. Confira endereço, VPN e certificado TLS.");
        setNetworkMessage("Servidor salvo, mas indisponível. Confira endereço, VPN e certificado TLS.");
      }
      if (revocationFailed) {
        setFeedback("Servidor alterado; não foi possível confirmar todas as revogações no servidor anterior. As sessões expiram automaticamente.");
        setNetworkMessage("Servidor alterado. Algumas sessões anteriores expirarão automaticamente.");
      }
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Não foi possível salvar o servidor.");
      setNetworkMessage(error instanceof Error ? error.message : "Não foi possível salvar o servidor.");
    } finally {
      setServerSaving(false);
    }
  }

  async function saveProxy(): Promise<void> {
    if (proxySaving || !proxyConfiguration || proxyConfiguration.environment_override) return;
    setProxySaving(true);
    setNetworkMessage("");
    try {
      if (proxyModeDraft === "manual" && !proxyUrlDraft.trim()) throw new Error("Informe o endereço do proxy.");
      const changed = proxyModeDraft !== proxyConfiguration.mode ||
        (proxyModeDraft === "manual" && proxyUrlDraft.trim() !== proxyConfiguration.url);
      if (!changed) return;
      const validated = await validateProxyConfiguration(proxyModeDraft, proxyUrlDraft.trim());
      const revocationFailed = await disconnectForNetworkChange();
      const saved = await saveProxyConfiguration(validated.mode, validated.url);
      setProxyConfiguration(saved);
      setProxyModeDraft(saved.mode);
      setProxyUrlDraft(saved.url);
      try {
        const health = await getBackendStatus();
        setBackendReady(health !== null);
        setNetworkMessage(health ? "Proxy salvo. Servidor conectado." : "Proxy salvo. Não foi possível alcançar o servidor; confira endereço e autenticação da rede.");
      } catch {
        setBackendReady(false);
        setNetworkMessage("Proxy salvo. Não foi possível alcançar o servidor; confira endereço e autenticação da rede.");
      }
      if (revocationFailed) setNetworkMessage("Proxy salvo. Algumas sessões anteriores expirarão automaticamente.");
    } catch (error: unknown) {
      setNetworkMessage(error instanceof Error ? error.message : "Não foi possível salvar o proxy.");
    } finally {
      setProxySaving(false);
    }
  }

  async function toggleAutostart(): Promise<void> {
    if (autostartBusy) return;
    setAutostartBusy(true);
    setAutostartMessage("");
    try {
      const enabled = await setAutostart(!autostartEnabled);
      setAutostartEnabled(enabled);
      setAutostartMessage(enabled ? "O Seven Desk iniciará ao entrar neste usuário." : "Inicialização automática desativada.");
    } catch (error: unknown) {
      setAutostartMessage(error instanceof Error ? error.message : "Não foi possível alterar a inicialização automática.");
    } finally {
      setAutostartBusy(false);
    }
  }

  function openChat(sessionId: string): void {
    setChatMessages([]);
    setChatDraft("");
    setChatSessionId(sessionId);
  }

  async function copyId(): Promise<void> {
    if (!status?.device_id) return;
    try {
      await navigator.clipboard.writeText(status.device_id);
      setFeedback("Identificação copiada.");
    } catch {
      setFeedback("Não foi possível copiar. Verifique as permissões da área de transferência.");
    }
  }

  const hasIdentity = Boolean(status?.device_id);
  const hostAvailable = Boolean(status?.host_service_available);
  const currentInvitation = inbox[0] ?? null;
  const activeHostDialog = hostDialogSession
    ? activeSessions.find((item) => item.session_id === hostDialogSession) ?? null
    : null;
  const invitationState: InvitationState = {
    pending: currentInvitation,
    active: activeHostDialog,
    nativeInputAvailable: Boolean(status?.native_input_available),
    monitorAvailable: monitors.length > 0,
    deviceId: status?.device_id ?? null,
    decisionBusy: decisionBusy !== null,
    ending,
    chatBusy,
    chatMessages: activeHostDialog && chatSessionId === activeHostDialog.session_id ? chatMessages : [],
    feedback: invitationError,
  };
  const invitationStateRef = useRef(invitationState);
  invitationStateRef.current = invitationState;
  invitationActionRef.current = (action) => {
    if (action.type === "decide") {
      void respondToInvitation(action.sessionId, action.approve, action);
    } else if (action.type === "finalize") {
      void closeSession(action.sessionId);
    } else if (action.type === "chat" && activeHostDialog?.session_id === action.sessionId && !chatBusy) {
      const message = action.text.trim();
      if (!message || new TextEncoder().encode(message).length > 2_048 || [...message].length > 500) return;
      setChatBusy(true);
      void sendSessionChat(action.sessionId, message).then((sent) => {
        setChatMessages((current) => current.some((message) => message.sequence === sent.sequence) ? current : [...current, sent].slice(-24));
      }).catch((error: unknown) => {
        setFeedback(error instanceof Error ? error.message : "Falha ao enviar mensagem.");
      }).finally(() => setChatBusy(false));
    }
  };
  useEffect(() => {
    if (!isTauri()) return;
    let disposed = false;
    let stopActions: (() => void) | undefined;
    let stopReady: (() => void) | undefined;
    void listen<InvitationAction>(INVITATION_ACTION, (event) => invitationActionRef.current(event.payload)).then((stop) => { if (disposed) stop(); else stopActions = stop; });
    void listen(INVITATION_READY, () => { void emitTo(INVITATION_WINDOW, INVITATION_STATE, invitationStateRef.current); }).then((stop) => { if (disposed) stop(); else stopReady = stop; });
    return () => { disposed = true; stopActions?.(); stopReady?.(); };
  }, []);
  useEffect(() => {
    if (!isTauri()) return;
    void emitTo(INVITATION_WINDOW, INVITATION_STATE, invitationState).catch(() => {});
  }, [currentInvitation, activeHostDialog, status, monitors, decisionBusy, ending, chatBusy, chatMessages, chatSessionId, invitationError]);
  useEffect(() => {
    if (!isTauri()) return;
    const timer = window.setInterval(() => { void emitTo(INVITATION_WINDOW, INVITATION_STATE, invitationStateRef.current).catch(() => {}); }, 1_000);
    return () => window.clearInterval(timer);
  }, []);
  useEffect(() => {
    if (!isTauri() || currentInvitation || activeHostDialog || decisionBusy) return;
    void WebviewWindow.getByLabel(INVITATION_WINDOW).then((window) => window?.hide()).catch(() => {});
  }, [currentInvitation?.session_id, activeHostDialog?.session_id, decisionBusy]);
  const mayConnect = hasIdentity && backendReady && presenceActive && target.trim().length > 0
    && !requesting && (!outboundSession || outboundSession.status === "rejected" || outboundSession.status === "ended");

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand" aria-label="Seven Desk">
          <img className="brand-mark" src="/brand-mark.svg" alt="" aria-hidden="true" />
          <div><strong>Seven Desk</strong><span>Acesso remoto</span></div>
        </div>
        <nav aria-label="Navegação principal" className="navigation">
          {navigation.map((item) => (
            <button
              type="button"
              key={item.id}
              className={page === item.id ? "nav-link active" : "nav-link"}
              aria-current={page === item.id ? "page" : undefined}
              onClick={() => { setPage(item.id); setFeedback(""); }}
            >
              <span className="nav-icon" aria-hidden="true">{item.icon}</span>
              {item.label}
            </button>
          ))}
        </nav>
        <div className="sidebar-footer">
          <span className="indicator off" aria-hidden="true" />
          <div><strong>{hostAvailable ? "Serviço disponível" : "Serviço indisponível"}</strong>
            <small>{backendReady ? "Servidor configurado online" : "Servidor configurado offline"}</small></div>
        </div>
      </aside>

      <main className="main-content">
        <header className="topbar">
          <div className="breadcrumb">Seven Desk <span>/</span> {navigation.find((item) => item.id === page)?.label}</div>
          <div className="topbar-actions">
            {streamingSession && (
              <div className="sharing-indicator" role="status">
                <span className="dot" aria-hidden="true" />
                <span>Compartilhando</span>
                <button type="button" onClick={() => void stopSharing(streamingSession)}>Parar</button>
              </div>
            )}
            <span className="version">{status ? `v${status.app_version}` : "Pré-alfa"}</span>
          </div>
        </header>

        {page === "inicio" && (
          <div className="content">
            <div className="welcome">
              <div className="eyebrow">CONTROLE COM SEGURANÇA</div>
              <h1>Acesse e ofereça suporte de qualquer lugar.</h1>
              <p>Controle de dispositivos autorizados, com permissões claras e proteção em cada conexão.</p>
            </div>
            <div className="cards">
              <section className="panel identity-panel" aria-labelledby="identity-heading">
                <div className="section-title"><div className="card-icon">⌘</div>
                  <div><h2 id="identity-heading">Este dispositivo</h2>
                    <p>Identificação usada em conexões autorizadas</p></div></div>
                <label className="field-label" htmlFor="own-id">Seu ID</label>
                <div className="input-group">
                  <input id="own-id" value={status?.device_id ?? ""} placeholder={busy ? "Carregando..." : "Aguardando configuração"} readOnly aria-describedby="device-note" />
                  {hasIdentity && <button className="secondary" type="button" onClick={() => void copyId()}>Copiar</button>}
                </div>
                <div className="foot-note" id="device-note"><span className="dot amber" />
                  {hasIdentity
                    ? presenceActive ? "Disponível para receber pedidos. Cada conexão exige sua autorização." : "Identidade pronta. Aguardando conexão com o servidor."
                    : busy ? "Preparando identidade deste dispositivo..." : "Não foi possível preparar a identidade deste dispositivo."}
                </div>
                {hasIdentity && (
                  <p className="foot-note" role="status">
                    {presenceActive ? "Pronto para receber solicitações." : registering ? "Conectando ao servidor..." : "Tentando conectar automaticamente..."}
                  </p>
                )}
              </section>
              <section className="panel connection-panel" aria-labelledby="connect-heading">
                <div className="section-title"><div className="card-icon blue">↗</div>
                  <div><h2 id="connect-heading">Conectar-se</h2>
                    <p>Solicite acesso ao dispositivo remoto</p></div></div>
                <label className="field-label" htmlFor="target">ID do dispositivo remoto</label>
                <input id="target" placeholder="Digite o ID do equipamento" autoComplete="off"
                  value={target} onChange={(e) => setTarget(e.target.value.slice(0, 128))} />
                <label className="foot-note" style={{ display: "flex", alignItems: "center", gap: 8 }}>
                  <input type="checkbox" checked={requestMouse}
                    onChange={(event) => setRequestMouse(event.target.checked)}
                    aria-label="Solicitar controle de mouse além da visualização" />
                  Solicitar também controle de mouse (depende de aceite separado do host)
                </label>
                <label className="foot-note" style={{ display: "flex", alignItems: "center", gap: 8 }}>
                  <input type="checkbox" checked={requestKeyboard}
                    onChange={(event) => setRequestKeyboard(event.target.checked)}
                    aria-label="Solicitar controle de teclado além da visualização" />
                  Solicitar também controle de teclado (depende de aceite separado do host)
                </label>
                <label className="foot-note" style={{ display: "flex", alignItems: "center", gap: 8 }}>
                  <input type="checkbox" checked={requestChat}
                    onChange={(event) => setRequestChat(event.target.checked)}
                    aria-label="Solicitar chat de texto durante a sessão" />
                  Solicitar chat de texto (autorização independente do host)
                </label>
                <button className="primary connect" type="button" disabled={!mayConnect}
                  onClick={() => void sendInvitation()}>
                  <span aria-hidden="true">↗</span> {requesting ? "Solicitando..." : "Solicitar autorização"}
                </button>
                <div className="foot-note">Prévia JPEG até 15 FPS, conforme máquina e rede. O host escolhe as permissões antes de aceitar. Teclado aceita caracteres, edição básica e setas; sem atalhos com modificadores. Áudio remoto indisponível.</div>
                {outboundSession && (
                  <div className="foot-note" role="status">
                    <strong>Estado:</strong> {
                      outboundSession.status === "awaiting_consent" ? "Aguardando aprovação" :
                      outboundSession.status === "active" ? "Aprovado (visualização consentida disponível)" :
                      outboundSession.status === "rejected" ? "Recusado pelo host" : "Encerrado"
                    }
                    {outboundSession.status === "active" && (
                      <>
                        <button className="secondary" type="button"
                          onClick={() => {
                            setViewerMouseActive(false);
                            setViewerKeyboardActive(false);
                            setViewingSession((current) => current === outboundSession.session_id ? null : outboundSession.session_id);
                            setLastFrame(null);
                          }}>
                          {viewingSession === outboundSession.session_id ? "Parar visualização" : "Retomar visualização"}
                        </button>
                      </>
                    )}
                    {(outboundSession.status === "awaiting_consent" || outboundSession.status === "active") && (
                      <button className="secondary" type="button" disabled={ending}
                        onClick={() => void closeSession(outboundSession.session_id)}>Cancelar / encerrar</button>
                    )}
                  </div>
                )}
                {operatorSessions.length > 0 && (
                  <div className="foot-note">
                    <strong>Sessões autorizadas do operador</strong>
                    <p>Estas autorizações podem ser recuperadas após reabrir o programa, enquanto ainda estiverem ativas no servidor.</p>
                    {operatorSessions.map((item) => (
                      <div className="setting-row" key={item.session_id}>
                        <div>
                          <strong>Host: {item.host}</strong>
                          <p>Permissões: {item.granted.join(", ")}</p>
                        </div>
                        <div className="input-group">
                          {item.granted.includes("chat") && (
                            <button className="secondary" type="button"
                              onClick={() => openChat(item.session_id)}>Abrir chat</button>
                          )}
                          <button className="secondary" type="button"
                            disabled={outboundSession?.session_id === item.session_id && outboundSession.status === "active"}
                            onClick={() => {
                              setViewingSession(null);
                              setLastFrame(null);
                              setOutboundSession({ session_id: item.session_id, status: "active" });
                            }}>Retomar visualização</button>
                          <button className="secondary" type="button" disabled={ending}
                            onClick={() => void closeSession(item.session_id)}>Encerrar</button>
                        </div>
                      </div>
                    ))}
                  </div>
                )}
              </section>
            </div>
            {presenceActive && (
              <section className="panel recent-panel" aria-labelledby="inbox-heading">
                <div className="section-title"><div className="card-icon muted">✓</div>
                  <div><h2 id="inbox-heading">Solicitações recebidas</h2>
                  <p>Apenas pedidos autenticados destinados a este dispositivo aparecem aqui.</p></div>
                </div>
                {inbox.length === 0
                  ? <p className="foot-note">Nenhuma solicitação aguardando autorização.</p>
                  : <p className="foot-note" role="status">{inbox.length} solicitação{inbox.length > 1 ? "ões" : ""} aguardando sua decisão. O pedido aparece em destaque nesta janela.</p>}
                <div className="section-title"><div className="card-icon muted">●</div>
                  <div><h2>Autorizações ativas</h2>
                    <p>Visíveis mesmo depois de reabrir a interface, enquanto o servidor mantiver as sessões.</p></div>
                </div>
                {status?.operating_system === "linux" && !status.native_input_available && (
                  <p className="foot-note">Nesta sessão Linux, a tela depende do portal do sistema. Mouse e teclado remotos estão disponíveis apenas em X11 nesta versão.</p>
                )}
                {status?.operating_system === "macos" && (
                  <p className="foot-note">O macOS pode pedir permissão de Gravação de Tela e Acessibilidade antes de compartilhar e controlar.</p>
                )}
                {monitorError && <p className="foot-note">Captura indisponível: {monitorError}</p>}
                {status && monitors.length > 0 && (
                  <div className="setting-row">
                    <div><strong>Monitor compartilhado</strong>
                      <p>O host escolhe qual tela mostrar. Mouse remoto funciona apenas no monitor principal.</p></div>
                    <select aria-label="Monitor compartilhado" disabled={monitorBusy}
                      value={selectedMonitorId ?? ""}
                      onChange={(event) => void chooseMonitor(Number(event.target.value))}>
                      {monitors.map((monitor) => (
                        <option key={monitor.id} value={monitor.id}>
                          {monitor.name || `Monitor ${monitor.id}`} · {monitor.width}×{monitor.height}{monitor.is_primary ? " (principal)" : ""}
                        </option>
                      ))}
                    </select>
                  </div>
                )}
                {activeSessions.length === 0 ? <p className="foot-note">Nenhuma autorização ativa.</p> :
                  activeSessions.map((item) => (
                    <div className="setting-row" key={item.session_id}>
                      <div>
                        <strong>Operador: {item.operator}</strong>
                        <p>Capacidades: {item.granted.join(", ")}. Compartilhamento opcional e sempre interrompível.</p>
                      </div>
                      <div className="input-group">
                        {item.granted.includes("chat") && (
                          <button className="secondary" type="button"
                            onClick={() => openChat(item.session_id)}>Abrir chat</button>
                        )}
                        <button className="secondary" type="button"
                          disabled={frameBusy || streamingSession !== null || monitors.length === 0}
                          onClick={() => void shareManualFrame(item.session_id)}>
                          {frameBusy ? "Capturando..." : "Compartilhar um quadro"}
                        </button>
                        <button className="secondary" type="button"
                          disabled={sharingBusy || monitors.length === 0}
                          onClick={() => void toggleSharing(item.session_id)}>
                          {streamingSession === item.session_id ? "Parar quadros" : "Iniciar prévia (até 15 FPS)"}
                        </button>
                        {hostMouseSession === item.session_id && (
                          <button className="secondary" type="button" disabled={mouseBusy}
                            onClick={() => void toggleHostMouse(item.session_id)}>Parar mouse remoto</button>
                        )}
                        {hostKeyboardSession === item.session_id && (
                          <button className="secondary" type="button" disabled={keyboardBusy}
                            onClick={() => void toggleHostKeyboard(item.session_id)}>Parar teclado remoto</button>
                        )}
                        {item.granted.includes("chat") &&
                          <button className="secondary" type="button"
                            disabled={revokingChat !== null}
                            onClick={() => void stopSessionChat(item.session_id)}>
                            {revokingChat === item.session_id ? "Revogando chat..." : "Revogar somente chat"}
                          </button>}
                        <button className="secondary" type="button" disabled={ending}
                          onClick={() => void closeSession(item.session_id)}>Encerrar autorização</button>
                      </div>
                    </div>
                  ))
                }
              </section>
            )}
            {chatSessionId && (
              <section className="panel recent-panel" aria-label="Chat da sessão">
                <div className="section-title"><div><h2>Chat da sessão</h2>
                  <p>Mensagens temporárias neste servidor de laboratório; conteúdo ainda não tem criptografia ponta a ponta.</p></div></div>
                <div role="log" aria-live="polite" style={{ maxHeight: 220, overflowY: "auto" }}>
                  {chatMessages.length === 0 ? <p className="foot-note">Nenhuma mensagem nesta janela.</p> :
                    chatMessages.map((message) => (
                      <p key={message.sequence} className="foot-note">
                        <strong>{message.sender === status?.device_id ? "Você" : "Outro participante"}:</strong> {message.text}
                      </p>
                    ))}
                </div>
                <form onSubmit={(event) => { event.preventDefault(); void submitChat(); }}>
                  <label className="field-label" htmlFor="chat-draft">Mensagem</label>
                  <div className="input-group">
                    <input id="chat-draft" value={chatDraft} maxLength={500}
                      onChange={(event) => setChatDraft(event.target.value)} autoComplete="off" />
                    <button className="primary" type="submit" disabled={chatBusy || !chatDraft.trim()}>Enviar</button>
                    <button className="secondary" type="button" onClick={() => { setChatSessionId(null); setChatMessages([]); }}>Fechar chat</button>
                  </div>
                </form>
              </section>
            )}
            <section className="panel recent-panel">
              <div className="section-title"><div className="card-icon muted">▤</div>
                <div><h2>Conexões recentes</h2><p>Seus dispositivos aparecerão aqui quando houver sessões reais.</p></div></div>
              <div className="empty-state"><span className="empty-illustration" aria-hidden="true">▧</span>
                <strong>Nenhuma sessão registrada</strong><span>O histórico estará disponível após a primeira conexão autorizada.</span></div>
            </section>
          </div>
        )}

        {page === "dispositivos" && (
          <div className="content">
            <div className="welcome compact">
              <div className="eyebrow">SEUS EQUIPAMENTOS</div>
              <h1>Dispositivos</h1>
              <p>Organize contatos e favoritos locais. Adicionar um dispositivo não equivale a autorização remota.</p>
            </div>
            <DeviceDirectory
              ownDeviceId={status?.device_id ?? ""}
              presenceActive={presenceActive && backendReady}
              onConnect={(id) => {
                setTarget(id);
                setPage("inicio");
                setFeedback("ID preenchido no convite. Para se conectar, solicite autorização explícita ao host.");
              }}
            />
          </div>
        )}

        {page === "configuracoes" && (
          <div className="content"><div className="welcome compact"><div className="eyebrow">PREFERÊNCIAS</div>
            <h1>Configurações</h1><p>Personalize a interface e consulte as informações do aplicativo.</p></div>
            <section className="panel settings-panel">
              <div className="setting-row"><div><strong>Aparência</strong><p>Escolha o tema da interface Seven Desk.</p></div>
                <select aria-label="Tema da interface" value={theme} onChange={(e) => setTheme(e.target.value === "light" ? "light" : "dark")}>
                  <option value="dark">Escuro</option><option value="light">Claro</option>
                </select></div>
              <div className="setting-row"><div><strong>Sistema operacional</strong><p>Detectado pelo runtime nativo.</p></div>
                <span className="setting-value">{status ? `${status.operating_system} · ${status.architecture}` : "Navegador / indisponível"}</span></div>
              <div className="setting-row"><div><strong>Servidor de desenvolvimento</strong><p>Conexão nativa com HTTPS fora do loopback; não homologada para produção.</p></div>
                <span className="setting-value">{backendReady ? "Online" : "Offline"}</span></div>
              <div className="setting-row"><div><strong>Relay de conexão</strong><p>Encaminha tela e comandos das sessões autorizadas pelo mesmo servidor HTTPS.</p></div>
                <span className="setting-value">{relayReady ? "Online" : "Offline"}</span></div>
              <div className="setting-row"><div><strong>Endereço do servidor</strong>
                <p>Use HTTPS para outro computador. Salvar encerra as sessões locais e reconecta automaticamente.</p></div>
                <div className="input-group network-input-group">
                  <input aria-label="Endereço do servidor" type="url" value={serverUrlDraft}
                    disabled={!status || !serverConfiguration || serverSaving || serverConfiguration.environment_override}
                    onChange={(event) => setServerUrlDraft(event.target.value.slice(0, 2048))} />
                  <button className="secondary" type="button"
                    disabled={!status || !serverConfiguration || serverSaving || serverConfiguration.environment_override || !serverUrlDraft.trim() || serverUrlDraft.trim() === serverConfiguration.url}
                    onClick={() => void saveServer()}>{serverSaving ? "Salvando..." : "Salvar"}</button>
                </div>
              </div>
              {serverConfiguration?.environment_override && (
                <p className="foot-note">Endereço definido pela variável de ambiente desta execução. Feche o aplicativo e remova a variável para editar aqui.</p>
              )}
              <div className="setting-row"><div><strong>Proxy de rede</strong>
                <p>Use se esta rede exigir proxy para acessar o servidor HTTPS. Conexões locais continuam diretas.</p></div>
                <div className="proxy-controls">
                  <select aria-label="Modo de proxy" value={proxyModeDraft}
                    disabled={!proxyConfiguration || proxySaving || proxyConfiguration.environment_override}
                    onChange={(event) => setProxyModeDraft(event.target.value as ProxyConfiguration["mode"])}>
                    <option value="direct">Direto</option>
                    <option value="system">Detectar proxy do sistema</option>
                    <option value="manual">Manual</option>
                  </select>
                  {proxyModeDraft === "manual" && <input aria-label="Endereço do proxy" type="url"
                    placeholder="http://proxy.exemplo:8080" value={proxyUrlDraft}
                    disabled={!proxyConfiguration || proxySaving || proxyConfiguration.environment_override}
                    onChange={(event) => setProxyUrlDraft(event.target.value.slice(0, 2048))} />}
                  <button className="secondary" type="button"
                    disabled={!proxyConfiguration || proxySaving || proxyConfiguration.environment_override ||
                      (proxyModeDraft === proxyConfiguration.mode && (proxyModeDraft !== "manual" || proxyUrlDraft.trim() === proxyConfiguration.url))}
                    onClick={() => void saveProxy()}>{proxySaving ? "Salvando..." : "Salvar proxy"}</button>
                </div>
              </div>
              <p className="foot-note">A detecção usa as variáveis da rede e o proxy configurado no Windows ou macOS. No Linux, usa as variáveis da rede. PAC e autenticação NTLM do Windows ainda não estão disponíveis. O proxy manual aceita URL sem login e não guarda senhas.</p>
              {proxyConfiguration?.environment_override && <p className="foot-note">Proxy definido por SEVEN_DESK_PROXY_URL nesta execução.</p>}
              {networkMessage && <p className="foot-note" role="status">{networkMessage}</p>}
              <div className="setting-row"><div><strong>Serviço remoto deste usuário</strong><p>Continua disponível na área de notificação ao fechar a janela principal. Cada conexão ainda exige seu aceite.</p></div>
                <span className="setting-value">{hostAvailable ? "Ativo" : "Indisponível"}</span></div>
              <div className="setting-row"><div><strong>Iniciar ao entrar no sistema</strong><p>Abre o agente em segundo plano quando este usuário entrar no Windows, macOS ou Linux.</p></div>
                <button className="secondary" type="button" disabled={!status || autostartBusy} onClick={() => void toggleAutostart()}>
                  {autostartBusy ? "Aguarde..." : autostartEnabled ? "Desativar" : "Ativar"}
                </button></div>
              {autostartMessage && <p className="foot-note" role="status">{autostartMessage}</p>}
            </section>
          </div>
        )}
        {viewingSession && outboundSession?.status === "active" && outboundSession.session_id === viewingSession && (
          <section className="remote-viewer" aria-label="Tela remota">
            <div className="remote-viewer-toolbar">
              <div>
                <strong>Seven Desk · Sessão remota</strong>
                <span>Host: {operatorSessions.find((item) => item.session_id === viewingSession)?.host ?? "Conectado"}</span>
              </div>
              <div className="remote-viewer-actions">
                {viewerMouseActive && <button type="button" onClick={() => setViewerMouseActive(false)}>Parar mouse</button>}
                {viewerKeyboardActive && <button type="button" onClick={() => setViewerKeyboardActive(false)}>Parar teclado</button>}
                <button type="button" onClick={() => {
                  setViewerMouseActive(false);
                  setViewerKeyboardActive(false);
                  setViewingSession(null);
                  setLastFrame(null);
                }}>Voltar ao aplicativo</button>
                <button className="danger" type="button" disabled={ending} onClick={() => void closeSession(viewingSession)}>Encerrar conexão</button>
              </div>
            </div>
            <div className="remote-viewer-stage">
              {lastFrame ? (
                <img ref={viewerImageRef} className="remote-viewer-image" src={lastFrame}
                  alt="Tela do computador remoto" tabIndex={viewerKeyboardActive ? 0 : -1}
                  onKeyDown={forwardViewerKey}
                  onPointerDown={(event) => event.currentTarget.focus()}
                  onPointerMove={(event) => forwardViewerPointer(event, "move")}
                  onWheel={forwardViewerWheel}
                  onPointerUp={(event) => {
                    if (event.button === 0) forwardViewerPointer(event, "click", "left");
                    if (event.button === 1) forwardViewerPointer(event, "click", "middle");
                    if (event.button === 2) forwardViewerPointer(event, "click", "right");
                  }}
                  onContextMenu={(event) => { if (viewerMouseActive) event.preventDefault(); }}
                  style={{ cursor: viewerMouseActive ? "crosshair" : "default" }} />
              ) : <div className="remote-viewer-wait" role="status">Aguardando a imagem do computador remoto...</div>}
            </div>
          </section>
        )}
      </main>
    </div>
  );
}
