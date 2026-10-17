import { useEffect, useState } from "react";
import {
  getBackendStatus,
  getLocalStatus,
  heartbeatLocalDevice,
  initializeIdentity,
  registerLocalDevice,
  requestSupport,
  pendingInvitations,
  activeAuthorizations,
  decideInvitation,
  readSessionState,
  terminateSession,
  publishManualFrame,
  getAuthorizedFrame,
  type LocalStatus,
  type SessionReceipt,
  type PendingInvitation,
  type ActiveAuthorization,
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
  const [feedback, setFeedback] = useState("");
  const [busy, setBusy] = useState(true);
  const [enrolling, setEnrolling] = useState(false);
  const [registering, setRegistering] = useState(false);
  const [presenceActive, setPresenceActive] = useState(false);
  const [requesting, setRequesting] = useState(false);
  const [outboundSession, setOutboundSession] = useState<SessionReceipt | null>(null);
  const [activeSessions, setActiveSessions] = useState<ActiveAuthorization[]>([]);
  const [inbox, setInbox] = useState<PendingInvitation[]>([]);
  const [decisionBusy, setDecisionBusy] = useState<string | null>(null);
  const [ending, setEnding] = useState(false);
  const [frameBusy, setFrameBusy] = useState(false);
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
      return;
    }
    let stopped = false;
    let fetching = false;
    async function refresh(): Promise<void> {
      if (fetching) return;
      fetching = true;
      try {
        const [received, authorizations] = await Promise.all([
          pendingInvitations(),
          activeAuthorizations(),
        ]);
        if (!stopped) {
          setInbox(received);
          setActiveSessions(authorizations);
        }
      } catch {
        if (!stopped) {
          setInbox([]);
          setActiveSessions([]);
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
          setStreamingSession(null);
          setFeedback(error instanceof Error ? error.message : "Compartilhamento interrompido.");
        }
      } finally {
        busyFrame = false;
      }
    }
    void capture();
    const timer = window.setInterval(() => void capture(), 1_000);
    const stopIfHidden = () => { if (document.hidden) setStreamingSession(null); };
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
      setStreamingSession(null);
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
      setStreamingSession(null);
    }
  }, [activeSessions, streamingSession]);

  async function sendInvitation(): Promise<void> {
    if (!presenceActive || requesting) return;
    setRequesting(true);
    try {
      const result = await requestSupport(target.trim());
      setOutboundSession(result);
      setFeedback("Solicitação de negociação enviada. Aguarde a autorização no host.");
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Falha ao solicitar sessão.");
    } finally {
      setRequesting(false);
    }
  }

  async function respondToInvitation(sessionId: string, approve: boolean): Promise<void> {
    if (decisionBusy) return;
    setDecisionBusy(sessionId);
    try {
      await decideInvitation(sessionId, approve);
      setInbox((previous) => previous.filter((item) => item.session_id !== sessionId));
      // The signed authorization listing refreshes automatically after approval.
      setFeedback(approve
        ? "Negociação autorizada. Compartilhamento de tela ainda não está implementado."
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
      setActiveSessions((current) => current.filter((item) => item.session_id !== sessionId));
      setFeedback("Negociação encerrada e permissões revogadas.");
    } catch (error: unknown) {
      setFeedback(error instanceof Error ? error.message : "Não foi possível encerrar a sessão.");
    } finally {
      setEnding(false);
    }
  }

  async function shareManualFrame(sessionId: string): Promise<void> {
    if (frameBusy) return;
    setFrameBusy(true);
    try {
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
            <p>A tela principal está sendo capturada enquanto esta janela permanecer aberta. Sem controle remoto ou áudio.</p>
            <button className="secondary" type="button" onClick={() => setStreamingSession(null)}>Parar compartilhamento agora</button>
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
                <button className="primary connect" type="button" disabled={!mayConnect}
                  onClick={() => void sendInvitation()}>
                  <span aria-hidden="true">↗</span> {requesting ? "Solicitando..." : "Solicitar autorização"}
                </button>
                <div className="foot-note">Quadros estáticos autorizados podem ser atualizados até uma vez por segundo. Sem áudio, mouse ou teclado remoto.</div>
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
                {lastFrame && outboundSession?.status === "active" && (
                  <div className="foot-note">
                    <p>{viewingSession ? "Atualização autorizada de até 1 quadro por segundo." : "Quadro estático autorizado."} Sem áudio nem controle remoto.</p>
                    <img src={lastFrame} alt="Último quadro da tela remota compartilhado manualmente pelo host"
                      style={{ width: "100%", maxHeight: 480, objectFit: "contain", borderRadius: 8 }} />
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
                          onClick={() => void respondToInvitation(invitation.session_id, true)}>Autorizar negociação</button>
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
                          disabled={status?.operating_system !== "windows"}
                          onClick={() => setStreamingSession((current) => current === item.session_id ? null : item.session_id)}>
                          {streamingSession === item.session_id ? "Parar quadros" : "Iniciar quadros (~1 FPS)"}
                        </button>
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
