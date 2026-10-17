import { useEffect, useRef, useState, type PointerEvent as ReactPointerEvent } from "react";
import {
  getBackendStatus,
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
  getAuthorizedFrame,
  setScreenDelivery,
  sendRemoteMouse,
  configureHostMouse,
  pollHostMouse,
  type RemoteMouseEvent,
  type LocalStatus,
  type SessionReceipt,
  type PendingInvitation,
  type ActiveAuthorization,
  type ActiveOperatorSession,
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
  const [target, setTarget] = useState("");
  const [requestMouse, setRequestMouse] = useState(false);
  const [viewerMouseActive, setViewerMouseActive] = useState(false);
  const [hostMouseSession, setHostMouseSession] = useState<string | null>(null);
  const [mouseBusy, setMouseBusy] = useState(false);
  const lastMouseMovement = useRef(0);
  const inputSendBusy = useRef(false);
  const lastMouseWireAt = useRef(0);
  const mouseSendChain = useRef<Promise<void>>(Promise.resolve());
  const viewerMouseEnabledRef = useRef(false);
  const [feedback, setFeedback] = useState("");
  const [busy, setBusy] = useState(true);
  const [enrolling, setEnrolling] = useState(false);
  const [registering, setRegistering] = useState(false);
  const [presenceActive, setPresenceActive] = useState(false);
  const [requesting, setRequesting] = useState(false);
  const [outboundSession, setOutboundSession] = useState<SessionReceipt | null>(null);
  const [activeSessions, setActiveSessions] = useState<ActiveAuthorization[]>([]);
  const [operatorSessions, setOperatorSessions] = useState<ActiveOperatorSession[]>([]);
  const [inbox, setInbox] = useState<PendingInvitation[]>([]);
  const [decisionBusy, setDecisionBusy] = useState<string | null>(null);
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

  useEffect(() => {
    let mounted = true;
    const controller = new AbortController();

    async function check() {
      try {
        const local = await getLocalStatus();
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
    return () => { mounted = false; controller.abort(); };
  }, []);

  async function activateIdentity(): Promise<void> {
    setEnrolling(true);
    try {
      await initializeIdentity();
      const updated = await getLocalStatus();
      setStatus(updated);
      setFeedback("Identidade criada e protegida pelo armazenamento seguro do sistema.");
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Não foi possível ativar este dispositivo.");
    } finally {
      setEnrolling(false);
    }
  }

  // Presence is opt-in for this running UI session. An interrupted heartbeat
  // expires server-side after the configured TTL; it never enables remote input.
  useEffect(() => {
    if (!presenceActive) return;
    const timer = window.setInterval(() => {
      void heartbeatLocalDevice().catch(() => {
        setPresenceActive(false);
        setFeedback("Presença local interrompida. Registre este dispositivo novamente.");
      });
    }, 30_000);
    return () => window.clearInterval(timer);
  }, [presenceActive]);

  async function registerWithLocalServer(): Promise<void> {
    if (!status?.device_id || !backendReady || registering) return;
    setRegistering(true);
    try {
      const registeredId = await registerLocalDevice();
      if (registeredId !== status.device_id) {
        throw new Error("A identidade registrada não corresponde a este dispositivo.");
      }
      setPresenceActive(true);
      setFeedback("Dispositivo registrado no servidor configurado de desenvolvimento.");
    } catch (error: unknown) {
      setPresenceActive(false);
      setFeedback(error instanceof Error ? error.message : "Falha ao registrar dispositivo.");
    } finally {
      setRegistering(false);
    }
  }

  // Only the authenticated device can retrieve its own outstanding requests.
  // Requests are refreshed while the user has explicitly enabled presence.
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

  // Explicit host-initiated low-rate frame publishing. No background service
  // or automatic capture exists: closing the app stops the timer.
  useEffect(() => {
    if (!streamingSession || !presenceActive || !backendReady) return;
    const id = streamingSession;
    let cancelled = false;
    let busyFrame = false;
    async function capture(): Promise<void> {
      if (cancelled || busyFrame) return;
      busyFrame = true;
      try {
        await publishManualFrame(id);
      } catch (error: unknown) {
        if (!cancelled) {
          void stopSharing(id);
          setFeedback(error instanceof Error ? error.message : "Compartilhamento interrompido.");
        }
      } finally {
        busyFrame = false;
      }
    }
    void capture();
    const timer = window.setInterval(() => void capture(), 1_000);
    const stopIfHidden = () => { if (document.hidden) void stopSharing(id); };
    document.addEventListener("visibilitychange", stopIfHidden);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
      document.removeEventListener("visibilitychange", stopIfHidden);
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
    const timer = window.setInterval(() => void refreshFrame(), 1_000);
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
    }
  }, [outboundSession?.status]);

  useEffect(() => {
    if (streamingSession && !activeSessions.some((session) => session.session_id === streamingSession)) {
      void stopSharing(streamingSession);
    }
  }, [activeSessions, streamingSession]);

  // Mouse events are only applied while the host explicitly enabled them,
  // screen sharing remains active, and this frontend is not hidden.
  useEffect(() => {
    if (!hostMouseSession) return;
    const id = hostMouseSession;
    if (!presenceActive || !backendReady || streamingSession !== id) {
      setHostMouseSession(null);
      void configureHostMouse(id, false);
      return;
    }
    let stopped = false;
    let busy = false;
    async function pump(): Promise<void> {
      if (stopped || busy) return;
      busy = true;
      try {
        await pollHostMouse(id);
      } catch (error: unknown) {
        if (!stopped) {
          setHostMouseSession(null);
          void configureHostMouse(id, false);
          setFeedback(error instanceof Error ? error.message : "Mouse remoto interrompido.");
        }
      } finally {
        busy = false;
      }
    }
    const timer = window.setInterval(() => void pump(), 180);
    const stopWhenHidden = () => {
      if (document.hidden) {
        setHostMouseSession(null);
        void configureHostMouse(id, false);
      }
    };
    document.addEventListener("visibilitychange", stopWhenHidden);
    return () => {
      stopped = true;
      window.clearInterval(timer);
      document.removeEventListener("visibilitychange", stopWhenHidden);
      void configureHostMouse(id, false);
    };
  }, [hostMouseSession, streamingSession, presenceActive, backendReady]);

  useEffect(() => {
    viewerMouseEnabledRef.current = viewerMouseActive;
  }, [viewerMouseActive]);

  useEffect(() => {
    if (!outboundSession || outboundSession.status !== "active" ||
        !operatorSessions.some((item) =>
          item.session_id === outboundSession.session_id && item.granted.includes("input")
        )) {
      setViewerMouseActive(false);
    }
  }, [outboundSession?.session_id, outboundSession?.status, operatorSessions]);

  async function sendInvitation(): Promise<void> {
    if (!presenceActive || requesting) return;
    setRequesting(true);
    try {
      const result = await requestSupport(target.trim(), requestMouse);
      setOutboundSession(result);
      setFeedback("Solicitação de negociação enviada. Aguarde a autorização no host.");
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Falha ao solicitar sessão.");
    } finally {
      setRequesting(false);
    }
  }

  async function respondToInvitation(sessionId: string, approve: boolean, grantMouse = false): Promise<void> {
    if (decisionBusy) return;
    setDecisionBusy(sessionId);
    try {
      await decideInvitation(sessionId, approve, grantMouse);
      setInbox((previous) => previous.filter((item) => item.session_id !== sessionId));
      // The signed authorization listing refreshes automatically after approval.
      setFeedback(approve
        ? "Negociação autorizada. O host pode compartilhar quadros da tela explicitamente."
        : "Solicitação recusada com sucesso.");
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Não foi possível responder ao convite.");
    } finally {
      setDecisionBusy(null);
    }
  }

  async function closeSession(sessionId: string): Promise<void> {
    if (ending) return;
    setEnding(true);
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
      setActiveSessions((current) => current.filter((item) => item.session_id !== sessionId));
      setOperatorSessions((current) => current.filter((item) => item.session_id !== sessionId));
      setFeedback("Negociação encerrada e permissões revogadas.");
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Não foi possível encerrar a sessão.");
    } finally {
      setEnding(false);
    }
  }

  async function stopSharing(sessionId: string): Promise<void> {
    // Stop local capture immediately, even if the server is offline. The
    // backend independently deletes cached pixels when the signed stop arrives.
    setStreamingSession(null);
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
      setFeedback("Compartilhamento iniciado com autorização do host e botão de parada visível.");
    } catch (error: unknown) {
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

  function imagePointer(event: ReactPointerEvent<HTMLImageElement>): { x: number; y: number } | null {
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

  async function shareManualFrame(sessionId: string): Promise<void> {
    if (frameBusy) return;
    setFrameBusy(true);
    try {
      // The host may have paused the previous stream; a fresh explicit click
      // is required before a new single-frame publication.
      await setScreenDelivery(sessionId, true);
      await publishManualFrame(sessionId);
      setFeedback("Quadro da tela principal enviado após sua autorização. Nenhum streaming contínuo foi iniciado.");
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Não foi possível capturar e compartilhar o quadro.");
    } finally {
      setFrameBusy(false);
    }
  }

  async function viewManualFrame(sessionId: string): Promise<void> {
    if (frameBusy) return;
    setFrameBusy(true);
    try {
      setLastFrame(await getAuthorizedFrame(sessionId));
      setFeedback("Quadro autorizado recebido. A imagem não se atualiza automaticamente.");
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Nenhum quadro recente disponível.");
    } finally {
      setFrameBusy(false);
    }
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
  const mayConnect = hasIdentity && backendReady && presenceActive && target.trim().length > 0
    && !requesting && (!outboundSession || outboundSession.status === "rejected" || outboundSession.status === "ended");

  return (
    <div className="app-shell">
      <aside className="sidebar">
        <div className="brand" aria-label="Seven Desk">
          <div className="brand-mark" aria-hidden="true">7</div>
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
          <span className="version">{status ? `v${status.app_version}` : "Pré-alfa"}</span>
        </header>
        {streamingSession && (
          <div className="panel" role="status" style={{ margin: 18, padding: 15, borderColor: "var(--accent)" }}>
            <strong>● Compartilhamento ativo — até 1 quadro por segundo</strong>
            <p>A tela principal está sendo capturada enquanto esta janela permanecer aberta. {hostMouseSession === streamingSession ? "Controle de mouse REMOTO ATIVO." : "Mouse remoto desativado."} Sem teclado ou áudio remoto.</p>
            {hostMouseSession === streamingSession && (
              <button className="secondary" type="button"
                onClick={() => { if (hostMouseSession) void toggleHostMouse(hostMouseSession); }}>
                Parar mouse remoto imediatamente
              </button>
            )}
            <button className="secondary" type="button" onClick={() => { if (streamingSession) void stopSharing(streamingSession); }}>Parar compartilhamento agora</button>
            <button className="secondary" type="button" disabled={ending}
              onClick={() => { if (streamingSession) void closeSession(streamingSession); }}>Encerrar autorização de tela</button>
          </div>
        )}

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
                  {hasIdentity ? (
                    <button className="secondary" type="button" onClick={() => void copyId()}>Copiar</button>
                  ) : (
                    <button className="secondary" type="button" disabled={busy || enrolling} onClick={() => void activateIdentity()}>
                      {enrolling ? "Ativando..." : "Ativar"}
                    </button>
                  )}
                </div>
                <div className="foot-note" id="device-note"><span className="dot amber" />
                  {hasIdentity
                    ? "Identidade local protegida. Serviço remoto ainda não configurado."
                    : "Ative uma identidade local antes de conectar ou compartilhar seu ID."}
                </div>
                {hasIdentity && (
                  <div>
                    <button
                      className="secondary"
                      type="button"
                      disabled={!backendReady || registering || presenceActive}
                      onClick={() => void registerWithLocalServer()}
                    >
                      {registering ? "Registrando..." : presenceActive ? "Presença local ativa" : "Registrar no servidor"}
                    </button>
                    <p className="foot-note" role="status">
                      {presenceActive
                        ? "Presença temporária ativa. Não habilita captura nem acesso remoto."
                        : "Registro experimental e voluntário, disponível apenas com o servidor de desenvolvimento acessível."}
                    </p>
                  </div>
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
                <button className="primary connect" type="button" disabled={!mayConnect}
                  onClick={() => void sendInvitation()}>
                  <span aria-hidden="true">↗</span> {requesting ? "Solicitando..." : "Solicitar autorização"}
                </button>
                <div className="foot-note">Quadros PNG até ~1 FPS. O controle de mouse no Windows exige solicitação e aceite separados. Teclado e áudio remotos ainda não estão disponíveis.</div>
                {outboundSession && (
                  <div className="foot-note" role="status">
                    <strong>Estado:</strong> {
                      outboundSession.status === "awaiting_consent" ? "Aguardando aprovação" :
                      outboundSession.status === "active" ? "Aprovado (visualização consentida disponível)" :
                      outboundSession.status === "rejected" ? "Recusado pelo host" : "Encerrado"
                    }
                    {outboundSession.status === "active" && (
                      <>
                        <button className="secondary" type="button" disabled={frameBusy || viewingSession !== null}
                          onClick={() => void viewManualFrame(outboundSession.session_id)}>
                          {frameBusy ? "Carregando quadro..." : "Ver último quadro autorizado"}
                        </button>
                        <button className="secondary" type="button"
                          onClick={() => {
                            setViewerMouseActive(false);
                            setViewingSession((current) => current === outboundSession.session_id ? null : outboundSession.session_id);
                            setLastFrame(null);
                          }}>
                          {viewingSession === outboundSession.session_id ? "Parar atualização" : "Atualizar automaticamente (~1 FPS)"}
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
                {lastFrame && outboundSession?.status === "active" && (
                  <div className="foot-note">
                    <p>{viewingSession ? "Atualização autorizada de até 1 quadro por segundo." : "Quadro estático autorizado."} {viewerMouseActive ? "Mouse remoto habilitado pelo operador; o host precisa manter a entrada ativa." : "Mouse desativado."} Sem teclado remoto.</p>
                    {operatorSessions.some((item) =>
                      item.session_id === outboundSession.session_id && item.granted.includes("input")
                    ) && (
                      <button className="secondary" type="button"
                        onClick={() => setViewerMouseActive((current) => !current)}>
                        {viewerMouseActive ? "Parar controle do mouse" : "Controlar mouse na imagem"}
                      </button>
                    )}
                    <img src={lastFrame} alt="Imagem do desktop do host, com controle de mouse apenas quando autorizado"
                      onPointerMove={(event) => forwardViewerPointer(event, "move")}
                      onPointerUp={(event) => {
                        if (event.button === 0) forwardViewerPointer(event, "click", "left");
                        if (event.button === 1) forwardViewerPointer(event, "click", "middle");
                        if (event.button === 2) forwardViewerPointer(event, "click", "right");
                      }}
                      onContextMenu={(event) => { if (viewerMouseActive) event.preventDefault(); }}
                      style={{ width: "100%", maxHeight: 480, objectFit: "contain", borderRadius: 8,
                        cursor: viewerMouseActive ? "crosshair" : "default" }} />
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
                {inbox.length === 0 ? <p className="foot-note">Nenhuma solicitação aguardando autorização.</p> :
                  inbox.map((invitation) => (
                    <div className="setting-row" key={invitation.session_id}>
                      <div>
                        <strong>Operador: {invitation.operator}</strong>
                        <p>Permissões solicitadas: {invitation.requested.join(", ")}. O aceite não inicia transmissão de tela.</p>
                      </div>
                      <div className="input-group">
                        <button className="secondary" type="button" disabled={decisionBusy !== null}
                          onClick={() => void respondToInvitation(invitation.session_id, false)}>Recusar</button>
                        <button className="primary" type="button" disabled={decisionBusy !== null}
                          onClick={() => void respondToInvitation(invitation.session_id, true)}>Autorizar só tela</button>
                        {invitation.requested.includes("input") && status?.operating_system === "windows" && (
                          <button className="primary" type="button" disabled={decisionBusy !== null}
                            onClick={() => void respondToInvitation(invitation.session_id, true, true)}>
                            Autorizar tela + mouse
                          </button>
                        )}
                      </div>
                    </div>
                  ))
                }
                <div className="section-title"><div className="card-icon muted">●</div>
                  <div><h2>Autorizações ativas</h2>
                    <p>Visíveis mesmo depois de reabrir a interface, enquanto o servidor mantiver as sessões.</p></div>
                </div>
                {activeSessions.length === 0 ? <p className="foot-note">Nenhuma autorização ativa.</p> :
                  activeSessions.map((item) => (
                    <div className="setting-row" key={item.session_id}>
                      <div>
                        <strong>Operador: {item.operator}</strong>
                        <p>Capacidades: {item.granted.join(", ")}. Compartilhamento opcional e sempre interrompível.</p>
                      </div>
                      <div className="input-group">
                        <button className="secondary" type="button"
                          disabled={frameBusy || streamingSession !== null || status?.operating_system !== "windows"}
                          onClick={() => void shareManualFrame(item.session_id)}>
                          {frameBusy ? "Capturando..." : "Compartilhar um quadro"}
                        </button>
                        <button className="secondary" type="button"
                          disabled={sharingBusy || status?.operating_system !== "windows"}
                          onClick={() => void toggleSharing(item.session_id)}>
                          {streamingSession === item.session_id ? "Parar quadros" : "Iniciar quadros (~1 FPS)"}
                        </button>
                        {item.granted.includes("input") && status?.operating_system === "windows" && (
                          <button className="secondary" type="button"
                            disabled={mouseBusy || (streamingSession !== item.session_id && hostMouseSession !== item.session_id)}
                            onClick={() => void toggleHostMouse(item.session_id)}>
                            {hostMouseSession === item.session_id ? "Parar mouse remoto" : "Permitir mouse agora"}
                          </button>
                        )}
                        <button className="secondary" type="button" disabled={ending}
                          onClick={() => void closeSession(item.session_id)}>Encerrar autorização</button>
                      </div>
                    </div>
                  ))
                }
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
          <div className="content"><div className="welcome compact"><div className="eyebrow">SEUS EQUIPAMENTOS</div>
            <h1>Dispositivos</h1><p>Gerencie futuramente favoritos e computadores autorizados.</p></div>
            <section className="panel empty-page"><div className="empty-state"><span className="empty-illustration">▣</span>
              <strong>Nenhum dispositivo cadastrado</strong><span>O diretório será habilitado após concluir os serviços de identidade e autenticação.</span>
            </div></section>
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
              <div className="setting-row"><div><strong>Serviço remoto</strong><p>É necessário para captura, entrada e acesso autorizado.</p></div>
                <span className="setting-value">{hostAvailable ? "Disponível" : "Não configurado"}</span></div>
            </section>
          </div>
        )}
        {feedback && <div className="toast" role="status"><span>{feedback}</span><button type="button" aria-label="Fechar aviso" onClick={() => setFeedback("")}>×</button></div>}
      </main>
    </div>
  );
}
