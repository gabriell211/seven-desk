import { useEffect, useMemo, useState, type FormEvent } from "react";
import { checkDevicePresence } from "./bridge";

const MAX_CONTACTS = 50;
const KEY_PREFIX = "seven-desk-public-directory-v1:";
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;

interface Contact {
  id: string;
  name: string;
  group: string;
  favorite: boolean;
}

type Presence = "online" | "offline" | "unavailable";

function loadContacts(key: string, ownDeviceId: string): Contact[] {
  try {
    const raw: unknown = JSON.parse(window.localStorage.getItem(key) ?? "[]");
    if (!Array.isArray(raw)) return [];
    const seen = new Set<string>();
    const contacts: Contact[] = [];
    for (const item of raw) {
      if (!item || typeof item !== "object") continue;
      const data = item as Partial<Contact>;
      if (typeof data.id !== "string" || !UUID.test(data.id)) continue;
      const id = data.id.toLowerCase();
      if (id === ownDeviceId.toLowerCase() || seen.has(id)) continue;
      seen.add(id);
      contacts.push({
        id,
        name: typeof data.name === "string" ? data.name.trim().slice(0, 64) : "",
        group: typeof data.group === "string" ? data.group.trim().slice(0, 40) : "",
        favorite: data.favorite === true,
      });
      if (contacts.length === MAX_CONTACTS) break;
    }
    return contacts;
  } catch {
    return [];
  }
}

export function DeviceDirectory({
  ownDeviceId,
  presenceActive,
  onConnect,
}: {
  ownDeviceId: string;
  presenceActive: boolean;
  onConnect: (id: string) => void;
}) {
  const key = KEY_PREFIX + ownDeviceId.toLowerCase();
  const [loadedFor, setLoadedFor] = useState("");
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [newId, setNewId] = useState("");
  const [newName, setNewName] = useState("");
  const [newGroup, setNewGroup] = useState("");
  const [search, setSearch] = useState("");
  const [error, setError] = useState("");
  const [statuses, setStatuses] = useState<Record<string, Presence>>({});

  useEffect(() => {
    setContacts(ownDeviceId ? loadContacts(key, ownDeviceId) : []);
    setLoadedFor(key);
    setStatuses({});
  }, [key, ownDeviceId]);

  useEffect(() => {
    if (loadedFor !== key || !ownDeviceId) return;
    try {
      window.localStorage.setItem(key, JSON.stringify(contacts));
    } catch {
      setError("Não foi possível salvar contatos neste dispositivo.");
    }
  }, [contacts, loadedFor, key, ownDeviceId]);

  useEffect(() => {
    if (!presenceActive || loadedFor !== key) {
      setStatuses({});
      return;
    }
    let cancelled = false;
    let checking = false;
    async function refresh() {
      if (checking) return;
      checking = true;
      for (const contact of contacts) {
        if (cancelled) break;
        let state: Presence = "unavailable";
        try {
          state = await checkDevicePresence(contact.id) ? "online" : "offline";
        } catch {
          // Network/auth problems are NOT evidence that a device is offline.
        }
        if (!cancelled) {
          setStatuses((current) => ({ ...current, [contact.id]: state }));
        }
      }
      checking = false;
    }
    void refresh();
    const timer = window.setInterval(() => void refresh(), 45_000);
    return () => { cancelled = true; window.clearInterval(timer); };
  }, [contacts, key, loadedFor, presenceActive]);

  const visible = useMemo(() => {
    const term = search.trim().toLocaleLowerCase("pt-BR");
    return contacts.filter((item) =>
      !term || [item.id, item.name, item.group].some((value) => value.toLocaleLowerCase("pt-BR").includes(term))
    ).sort((a, b) =>
      Number(b.favorite) - Number(a.favorite) ||
      a.group.localeCompare(b.group, "pt-BR") ||
      (a.name || a.id).localeCompare(b.name || b.id, "pt-BR")
    );
  }, [contacts, search]);

  function add(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const id = newId.trim().toLowerCase();
    if (!UUID.test(id)) { setError("Digite um ID de dispositivo válido (UUID completo)."); return; }
    if (id === ownDeviceId.toLowerCase()) { setError("Seu próprio dispositivo não pode ser adicionado."); return; }
    if (contacts.some((item) => item.id === id)) { setError("Esse dispositivo já está na lista."); return; }
    if (contacts.length >= MAX_CONTACTS) { setError("Limite de 50 contatos locais atingido."); return; }
    setContacts((current) => [...current, { id, name: newName.trim().slice(0, 64), group: newGroup.trim().slice(0, 40), favorite: false }]);
    setNewId("");
    setNewName("");
    setNewGroup("");
    setError("");
  }

  if (!ownDeviceId) {
    return <section className="panel"><p>Ative sua identidade de dispositivo para salvar contatos locais.</p></section>;
  }

  return (
    <section className="panel directory-panel" aria-labelledby="directory-heading">
      <div className="section-title">
        <div className="card-icon blue" aria-hidden="true">▣</div>
        <div>
          <h2 id="directory-heading">Contatos e favoritos ({contacts.length}/{MAX_CONTACTS})</h2>
          <p>IDs públicos salvos somente neste computador, separados pela sua identidade.</p>
        </div>
      </div>
      <form onSubmit={add} className="directory-add">
        <label>
          <span className="field-label">ID de dispositivo</span>
          <input value={newId} onChange={(event) => setNewId(event.target.value.slice(0, 64))} placeholder="UUID do dispositivo" required autoComplete="off" />
        </label>
        <label>
          <span className="field-label">Nome amigável</span>
          <input value={newName} maxLength={64} onChange={(event) => setNewName(event.target.value)} placeholder="Ex.: Computador da recepção" />
        </label>
        <label>
          <span className="field-label">Grupo</span>
          <input value={newGroup} maxLength={40} onChange={(event) => setNewGroup(event.target.value)} placeholder="Ex.: Escritório" />
        </label>
        <button className="primary" type="submit" disabled={loadedFor !== key}>Adicionar</button>
      </form>
      {error && <p role="alert" className="directory-message">{error}</p>}
      <label>
        <span className="field-label">Buscar contatos</span>
        <input type="search" value={search} onChange={(event) => setSearch(event.target.value)} placeholder="Busque por nome, grupo ou ID" />
      </label>
      <p className="directory-message" role="status">
        {presenceActive
          ? "Presença consultada com assinatura do dispositivo e prazo definido no servidor; atualiza aproximadamente a cada 45 segundos."
          : "Registre sua presença na tela Início para consultar online/offline. Contatos não concedem acesso remoto."}
      </p>
      {visible.length === 0 ? (
        <div className="empty-state">Nenhum contato corresponde à busca ou à lista atual.</div>
      ) : (
        <div className="directory-list">
          {visible.map((contact) => (
            <div className="directory-entry" key={contact.id}>
              <div>
                <strong>{contact.name || "Dispositivo sem apelido"}</strong>
                <small>{contact.group || "Sem grupo"} · {contact.id}</small>
                <span className="directory-presence" role="status">
                  {!presenceActive ? "Presença não consultada" :
                    statuses[contact.id] === "online" ? "● Online (recente)" :
                    statuses[contact.id] === "offline" ? "○ Offline (sem heartbeat recente)" :
                    "Presença indisponível"}
                </span>
              </div>
              <div className="directory-actions">
                <button className="secondary" type="button"
                  aria-label={contact.favorite ? `Remover ${contact.name || contact.id} dos favoritos` : `Favoritar ${contact.name || contact.id}`}
                  onClick={() => setContacts((current) => current.map((item) =>
                    item.id === contact.id ? { ...item, favorite: !item.favorite } : item
                  ))}>{contact.favorite ? "★ Favorito" : "☆ Favoritar"}</button>
                <button className="secondary" type="button" onClick={() => onConnect(contact.id)}>Usar ID no convite</button>
                <button className="secondary" type="button" aria-label={`Remover ${contact.name || contact.id}`}
                  onClick={() => {
                    setContacts((current) => current.filter((item) => item.id !== contact.id));
                    setStatuses((current) => {
                      const next = { ...current }; delete next[contact.id]; return next;
                    });
                  }}>Excluir</button>
              </div>
            </div>
          ))}
        </div>
      )}
      <p className="directory-message">Salvar um ID ou marcá-lo como favorito nunca autoriza visualização, controle ou chat. O host continua precisando aceitar cada convite.</p>
    </section>
  );
}
