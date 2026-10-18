import { useEffect, useState } from "react";
import { emit, listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { InvitationAction, InvitationState } from "./invitationEvents";
import { INVITATION_ACTION, INVITATION_READY, INVITATION_STATE } from "./invitationEvents";

export function InvitationWindow() {
  const [state, setState] = useState<InvitationState | null>(null);
  const [permissions, setPermissions] = useState({ mouse: true, keyboard: true, chat: true });
  const [draft, setDraft] = useState("");

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    void listen<InvitationState>(INVITATION_STATE, (event) => {
      if (!disposed) setState(event.payload);
    }).then((stop) => {
      if (disposed) stop();
      else { unlisten = stop; void emit(INVITATION_READY); }
    });
    return () => { disposed = true; unlisten?.(); };
  }, []);

  useEffect(() => {
    let disposed = false;
    let unlisten: (() => void) | undefined;
    const current = getCurrentWindow();
    void current.onCloseRequested((event) => {
      event.preventDefault();
      void current.minimize();
    }).then((stop) => { if (disposed) stop(); else unlisten = stop; });
    return () => { disposed = true; unlisten?.(); };
  }, []);

  const session = state?.pending ?? state?.active;
  const active = !state?.pending && Boolean(state?.active);
  const capabilities = state?.pending?.requested ?? state?.active?.granted ?? [];

  function dispatch(action: InvitationAction): void {
    void emit(INVITATION_ACTION, action);
  }

  if (!session || !state) return <div className="invitation-window waiting">Aguardando pedido de conexão...</div>;

  return (
    <div className="invitation-window">
      <section className="invitation-dialog" role="dialog" aria-labelledby="invitation-title" aria-describedby="invitation-description">
        <div className="invitation-person">
          <div className="invitation-heading">
            <span className="invitation-avatar" aria-hidden="true">●</span>
            <div>
              <span className="eyebrow">{active ? "SESSÃO CONECTADA" : "PEDIDO DE CONEXÃO"}</span>
              <h2 id="invitation-title">{active ? "Acesso remoto em andamento" : "Alguém quer acessar seu computador"}</h2>
              <p id="invitation-description"><strong>{session.operator}</strong> {active ? "está conectado a este dispositivo." : "deseja se conectar a este dispositivo."}</p>
            </div>
          </div>
          <h3>{active ? "Permissões concedidas" : "Permissões solicitadas"}</h3>
          <p className="invitation-help">{active ? "Para interromper o acesso, clique em Finalizar." : "Confira o que será permitido nesta sessão. Você pode encerrar o acesso a qualquer momento."}</p>
          <div className="permission-list">
            <label className="permission-choice required"><input type="checkbox" checked disabled /><span><strong>Ver a tela</strong><small>{active ? "Permissão concedida nesta sessão." : "Compartilhamento começa quando você aceitar."}</small></span></label>
            {capabilities.includes("input") && <label className="permission-choice"><input type="checkbox" checked={active ? true : state.nativeInputAvailable && permissions.mouse} disabled={active || !state.nativeInputAvailable || state.decisionBusy} onChange={(event) => setPermissions((old) => ({ ...old, mouse: event.target.checked }))} /><span><strong>Controlar mouse</strong><small>{state.nativeInputAvailable ? "Mover e clicar na tela compartilhada." : "Indisponível nesta sessão do sistema."}</small></span></label>}
            {capabilities.includes("keyboard") && <label className="permission-choice"><input type="checkbox" checked={active ? true : state.nativeInputAvailable && permissions.keyboard} disabled={active || !state.nativeInputAvailable || state.decisionBusy} onChange={(event) => setPermissions((old) => ({ ...old, keyboard: event.target.checked }))} /><span><strong>Controlar teclado</strong><small>Digitar na sessão remota.</small></span></label>}
            {capabilities.includes("chat") && <label className="permission-choice"><input type="checkbox" checked={active ? true : permissions.chat} disabled={active || state.decisionBusy} onChange={(event) => setPermissions((old) => ({ ...old, chat: event.target.checked }))} /><span><strong>Bate-papo</strong><small>Trocar mensagens durante a sessão.</small></span></label>}
          </div>
          <div className="invitation-actions">
            <button className="accept" type="button" autoFocus={!active} disabled={active || state.decisionBusy || !state.monitorAvailable} onClick={() => dispatch({ type: "decide", sessionId: session.session_id, approve: true, ...permissions })}>{active ? "Conectado" : state.decisionBusy ? "Conectando..." : "Aceitar"}</button>
            <button className="reject" type="button" disabled={state.decisionBusy || state.ending} onClick={() => dispatch(active ? { type: "finalize", sessionId: session.session_id } : { type: "decide", sessionId: session.session_id, approve: false, ...permissions })}>{active ? state.ending ? "Finalizando..." : "Finalizar" : "Rejeitar"}</button>
          </div>
          {!active && !state.monitorAvailable && <p className="invitation-help">A captura de tela está indisponível neste computador. Você ainda pode rejeitar o pedido.</p>}
          {state.feedback && <p className="invitation-help" role="status">{state.feedback}</p>}
        </div>
        <div className="invitation-context">
          <div className="invitation-context-title">Bate-papo e detalhes</div>
          <div className="invitation-event"><span className="dot" /><span>{active ? "Sessão em andamento" : "Pedido de sessão recebido"}</span></div>
          {active && capabilities.includes("chat") ? <div className="invitation-chat"><div className="invitation-chat-messages" role="log" aria-live="polite">{state.chatMessages.length === 0 ? <p>Nenhuma mensagem ainda.</p> : state.chatMessages.map((message) => <p key={message.sequence}><strong>{message.sender === state.deviceId ? "Você" : "Operador"}:</strong> {message.text}</p>)}</div><form className="invitation-chat-form" onSubmit={(event) => { event.preventDefault(); if (draft.trim()) { dispatch({ type: "chat", sessionId: session.session_id, text: draft.trim() }); setDraft(""); } }}><input aria-label="Mensagem de bate-papo" value={draft} maxLength={500} onChange={(event) => setDraft(event.target.value)} placeholder="Mensagem" autoComplete="off" /><button className="primary" type="submit" disabled={state.chatBusy || !draft.trim()}>Enviar</button></form></div> : active ? <p>Compartilhamento autorizado. Restaure esta janela quando quiser finalizar a sessão.</p> : <p>O acesso só começa se você clicar em <strong>Aceitar</strong>. As permissões marcadas serão ativadas automaticamente.</p>}
        </div>
      </section>
    </div>
  );
}
