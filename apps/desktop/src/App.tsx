import { useEffect, useState } from "react";
import { getBackendStatus, getLocalStatus, type LocalStatus } from "./bridge";

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
  const mayConnect = hasIdentity && backendReady && hostAvailable && target.trim().length > 0;

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
            <small>{backendReady ? "Servidor local online" : "Servidor local offline"}</small></div>
        </div>
      </aside>

      <main className="main-content">
        <header className="topbar">
          <div className="breadcrumb">Seven Desk <span>/</span> {navigation.find((item) => item.id === page)?.label}</div>
          <span className="version">{status ? `v${status.app_version}` : "Pré-alfa"}</span>
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
                  <button className="secondary" type="button" disabled={!hasIdentity} onClick={() => void copyId()}>Copiar</button>
                </div>
                <div className="foot-note" id="device-note"><span className="dot amber" />
                  {hostAvailable ? "Disponível para configuração de acesso" : "O serviço remoto ainda não foi instalado/configurado."}
                </div>
              </section>
              <section className="panel connection-panel" aria-labelledby="connect-heading">
                <div className="section-title"><div className="card-icon blue">↗</div>
                  <div><h2 id="connect-heading">Conectar-se</h2>
                    <p>Solicite acesso ao dispositivo remoto</p></div></div>
                <label className="field-label" htmlFor="target">ID do dispositivo remoto</label>
                <input id="target" placeholder="Digite o ID do equipamento" autoComplete="off"
                  value={target} onChange={(e) => setTarget(e.target.value.slice(0, 128))} />
                <button className="primary connect" type="button" disabled={!mayConnect}
                  onClick={() => setFeedback("O fluxo de conexão criptografada ainda está em desenvolvimento.")}>
                  <span aria-hidden="true">↗</span> Conectar
                </button>
                <div className="foot-note">Conexões serão iniciadas somente após autorização do dispositivo.</div>
              </section>
            </div>
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
              <div className="setting-row"><div><strong>Backend local</strong><p>Servidor de desenvolvimento, sem exposição pública.</p></div>
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
