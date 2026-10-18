# Seven Desk — Arquitetura de referência

**Status:** proposta para implementação, sujeita a ADRs e provas de conceito. Este documento não declara código existente nem tecnologias já homologadas.

## Objetivo e critérios

Entregar acesso remoto autorizado e de baixa latência, com UI responsiva, segurança por padrão, instaladores próprios, relay quando necessário e operação em desktop Windows, Linux e macOS. O primeiro fluxo homologável é **Windows↔Windows assistido**, em LAN e WAN controlada, com aceite e revogação explícitos. Recursos de operação avançada e móvel são incrementais.

## Estrutura dos processos

```text
┌──────────────────┐             ┌──────────────────────┐
│ Viewer / Operador│             │ Host / Dispositivo   │
│ Tauri v2 + React │             │ Tauri v2 + React     │
│ TS strict        │             │ TS strict            │
└────────┬─────────┘             └──────────┬───────────┘
         │ IPC tipado e escopado            │ IPC tipado e escopado
         ▼                                  ▼
┌──────────────────┐             ┌──────────────────────┐
│ Rust session core│             │ Rust session core    │
│ Mídia, input,    │             │ Captura, input,      │
│ codec, transporte│             │ codec, transporte    │
└────────┬─────────┘             └──────────┬───────────┘
         │                                  │
         │       Canal autenticado          │
         └──────────────┬───────────────────┘
                        │ P2P se possível
              ┌─────────▼──────────┐
              │ Relay autenticado  │
              │ quando necessário  │
              └─────────┬──────────┘
                        │ sinalização/presença
              ┌─────────▼───────────┐
              │ Control plane Rust  │
              │ Axum, auth, sessão, │
              │ permissões, auditoria│
              └─────────┬───────────┘
                        ▼
                 PostgreSQL
```

O **serviço do host** é um processo nativo separado da UI, instalado somente quando requerido e autorizado. Não confundir existência do serviço com habilitação de acesso não assistido. Um serviço privilegiado não deve executar browser/WebView nem aceitar comandos arbitrários vindos do frontend.

## Responsabilidades, invariantes e isolamento

| Módulo | O que faz | O que NÃO faz |
| --- | --- | --- |
| UI Tauri + React | Interface, onboarding, controles, estado visível e acessibilidade | Capturar/controlar via API web arbitrária, portar chaves privadas para JavaScript ou ignorar autorização |
| Rust core | Protocolo, sessão, mídia, controle, serialização, autorização defensiva | Conceder privilégio com base apenas em pedido do viewer |
| Serviço host | Integração de sistema, estado autorizado, ciclo de vida e ações locais privilegiadas | Abrir shell genérico, captura invisível ou acesso não assistido por padrão |
| Control plane | Registro, presence TTL, convite, políticas, autenticação e metadados auditáveis | Receber senha do host, decidir sozinho permissão local de uma ação, registrar vídeo por padrão |
| Transporte/relay | Canal autenticado e cifrado, P2P/fallback, cotas | Desativar autenticação, interpretar comando de controle sem política |
| PostgreSQL | Usuários, orgs, dispositivos, políticas, sessões e metadados | Guardar captura de tela, senha/clipboard ou chave privada do host em claro |

**Invariantes (obrigatórios):**
1. `device_public_id` identifica; **não autentica**. Chave privada é única por instalação e fica no armazenamento seguro do OS.
2. O host verifica identidade, política e autorização **para cada capacidade sensível** (teclado, mouse, arquivo, clipboard, túnel, terminal, gravação, reinício).
3. Sessão assistida só envia mídia/entrada após aceite explícito. Negação, timeout e revogação fecham canais.
4. Acesso não assistido nasce desabilitado, exige habilitação legítima, credenciais fortes, registro e revogação.
5. UI desconectada, queda de rede, falha de relay e processo interrompido não geram sessão permanente nem permissões extras.
6. Logs e telemetria minimizam dados; nunca registrar senhas, vídeo, conteúdo de arquivos e clipboard por padrão.
7. Instaladores legítimos não fazem bypass de prompts de segurança do sistema operacional.

## Contratos e dados

- Contratos versionados: `DeviceIdentity`, `SessionRequest`, `SessionGrant`, `PermissionScope`, `SessionEvent`, `TransportCapabilities`, `ProtocolError` e `TransferManifest`.
- Envelopes com `protocol_version`, `message_id`, `session_id`, `request_id`, `issued_at`, `expires_at`, `sender_identity` e nonce quando apropriado.
- Serialização limitada em tamanho e profundidade, com mensagens inválidas rejeitadas antes de alocar recursos caros. Não confiar em discriminadores vindos da UI.
- Backend multi-tenant com `tenant_id` em tabelas corporativas, constraints, índices e autorização em **todas** as consultas; separar cache e auditoria por organização.
- Redis **opcional**, nunca obrigatório para executar desenvolvimento local. Sem Docker como pré-requisito.

## Transporte e mídia — decisões a comprovar

**NÃO há escolha final validada.** PoC #9 deve comparar WebRTC/ICE/STUN/TURN e QUIC/relay no mesmo cenário, medindo p50/p95 de latência, custos de CPU/GPU, reconexão, integração de autenticação e capacidade do relay. PoC #51 deve comparar captura/encoder/decoder. Evitar codec/algoritmo próprio de criptografia e licenciamento sem auditoria. A conexão direta não é garantia em toda topologia NAT; fallback por relay é requisito para LAN/WAN.

### APIs nativas candidatas (validar por plataforma)

- Windows: Windows Graphics Capture ou DXGI Desktop Duplication; input e UAC devem respeitar isolamento de sessão e políticas do Windows.
- Linux X11/Wayland: portal de área de trabalho remota + ScreenCast/PipeWire conforme compositor. Portal apresenta diálogo de permissão; não contornar.
- macOS: ScreenCaptureKit para captura com consentimento do sistema; controles de entrada dependem de permissões oficiais de Acessibilidade.

Fontes públicas: [Windows DXGI](https://learn.microsoft.com/pt-br/windows-hardware/drivers/display/desktop-duplication-api) · [XDG RemoteDesktop](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.RemoteDesktop.html) · [Apple ScreenCaptureKit](https://developer.apple.com/documentation/ScreenCaptureKit) · [Tauri Security](https://v2.tauri.app/security/)

## Instalação oficial

| Windows | Linux | macOS |
| --- | --- | --- |
| `SevenDesk-Setup-<versão>.exe` via NSIS/Tauri | `.deb` e `.rpm` para distros homologadas | `.dmg` assinado/notarizado via Tauri, `.pkg` adicional se exigido |
| MSI opcional para empresa | systemd somente quando necessário | daemon/helper com autorização nativa |
| Serviço Windows separado, mediante opção e política | Entrada/captura via APIs autorizadas | Autorização TCC e Acessibilidade do SO |

A instalação não confere acesso permanente. **Assinatura, hashes, upgrade, rollback, desinstalador e identificação visual Seven Desk** fazem parte do gate de release.

## ADRs obrigatórias antes de implementação acoplada

| ID | Decisão | Critério de liberação | Issue |
| --- | --- | --- | --- |
| ADR-001 | Transporte e relay | PoC autenticada em LAN/WAN + custos e compatibilidade | #9 |
| ADR-002 | Codec/captura/renderização | Métricas e licenças por SO | #51 |
| ADR-003 | Identidade e chaves | Threat model + autenticação de pares + revogação | #50 |
| ADR-004 | Host service/IPC | Privilégio mínimo, auth IPC, sessão Windows, crash | #49 e #54 |
| ADR-005 | Banco/tenancy e migração | Isolamento, backup, índices e compatibilidade | #32 e #69 |
| ADR-006 | Licenciamento/ativos | Código original: AGPL-3.0-only (definido); inventário de ativos e licenças de terceiros ainda pendente | #67 |
| ADR-007 | Instalação/updates | Bundle assinado, serviço opcional, rollback | #15, #23 |
| ADR-008 | Escopo do produto 1.0 | Inclusões/exclusões e critérios de aceitação | #38 e docs/HOMOLOGACAO.md |

Cada ADR deve registrar contexto, alternativas, decisão, trade-offs, estado e evidências. **Não afirmar que uma decisão foi tomada sem testes.**
