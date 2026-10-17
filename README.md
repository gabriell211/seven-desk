<!-- Seven Desk | Documentação oficial de visão e requisitos -->
<div align="center">

# Seven Desk

### Acesso remoto seguro, rápido e sob seu controle.

**Plataforma multiplataforma de assistência remota, administração de dispositivos e colaboração em tempo real.**

[Visão geral](#visão-geral) · [Funcionalidades](#funcionalidades) · [Arquitetura](#arquitetura-proposta) · [Segurança](#segurança-e-privacidade) · [Roadmap](#roadmap) · [Contribuição](#desenvolvimento-e-contribuição)

</div>

> [!IMPORTANT]
> **Status: planejamento inicial.** Este repositório ainda não contém uma implementação funcional. Os recursos e tecnologias descritos representam **requisitos e decisões propostas**, não funcionalidades já entregues. O progresso deve ser acompanhado pelas [issues](https://github.com/gabriell211/seven-desk/issues) e por versões publicadas, quando existirem.

## Visão geral

**Seven Desk** é um projeto de software de acesso e suporte remoto, concebido para permitir que pessoas e equipes visualizem e operem computadores autorizados de forma segura, com boa experiência em redes locais e pela internet.

O produto prioriza **baixa latência**, **privacidade**, **controle explícito de permissões**, **confiabilidade**, **acessibilidade** e **operação em múltiplas plataformas**. O modo de atendimento assistido deve funcionar mediante aceitação da pessoa no equipamento remoto; o acesso não assistido exige habilitação prévia e controles adicionais.

### Princípios

- **Segurança por padrão:** sem sessões invisíveis, permissões implícitas ou acesso persistente habilitado automaticamente.
- **Consentimento e transparência:** solicitação identificável, indicação persistente de sessão ativa e encerramento pelo dispositivo remoto.
- **Performance adaptativa:** ajustar captura, compressão, resolução e taxa de quadros às condições da rede e da máquina.
- **Portabilidade:** componentes nativos por sistema operacional, com experiências coerentes.
- **Resiliência:** reconexão controlada, recuperação de rede, estados explícitos e observabilidade sem capturar dados privados por padrão.
- **Extensibilidade:** protocolos versionados e módulos desacoplados para evolução e distribuição corporativa.
- **Documentação verificável:** todo recurso anunciado como disponível deve ter implementação, testes e evidências de validação.

## Públicos e casos de uso

| Perfil | Necessidade |
| --- | --- |
| Usuários pessoais | Acessar seus próprios computadores a distância, com autorização e proteção da conta |
| Suporte técnico | Atender solicitações, compartilhar tela, transferir arquivos e resolver incidentes |
| Equipes de TI | Organizar parque de dispositivos, políticas, acessos, histórico e auditoria |
| Empresas | Administrar endpoints autorizados com controle por função, rastreabilidade e políticas de retenção |
| Equipes distribuídas | Colaborar em sessões, orientar usuários e trabalhar com múltiplos monitores |

## Funcionalidades

A tabela a seguir é o **escopo-alvo**, não uma lista de funcionalidades prontas.

| Área | Recursos previstos |
| --- | --- |
| Conexões | ID público do dispositivo, apelido, convite de sessão, favoritos, histórico, acesso LAN e remoto |
| Sessão assistida | Solicitar conexão, aceitar/rejeitar, escolher permissões, suspender controle e encerrar |
| Sessão não assistida | Habilitação explícita pelo proprietário/administrador, credenciais fortes, MFA, restrições por dispositivo e revogação |
| Visualização | Compartilhamento de tela, seleção de monitor, escalonamento, tela cheia, ajuste dinâmico de qualidade e FPS |
| Controle | Mouse, teclado, atalhos, mapeamento de teclado, teclas especiais, suporte a layouts e permissões granularizadas |
| Transferência | Arquivos bidirecionais, arrastar e soltar quando suportado, progresso, integridade e cancelamento |
| Área de transferência | Texto e, em fases posteriores, imagens/arquivos, sempre sujeitos a permissão |
| Comunicação | Chat de sessão, convites e mensagens de estado |
| Colaboração | Modo somente visualização, anotações temporárias, sessões com múltiplos participantes autorizados |
| Gerenciamento | Contatos, grupos, tags, pesquisa, dispositivos confiáveis, favoritos e políticas |
| Operação | Reinicialização remota autorizada, reconexão, Wake-on-LAN em ambientes compatíveis, execução com serviço instalado |
| Avançado | Impressão remota, áudio remoto, gravação consentida, modo de privacidade e API de integração |
| Empresas | Organizações, RBAC, SSO/OIDC, auditoria, políticas, implantação administrada e opção de infraestrutura própria |
| Distribuição | Aplicativo instalável, modo portátil onde permitido, atualização assinada, instaladores e desinstalação limpa |

### Jornada principal de suporte

1. A pessoa que receberá suporte abre o Seven Desk e compartilha seu **ID de dispositivo** ou convite temporário.
2. O técnico informa o ID, visualiza a identidade declarada do destino e solicita conexão.
3. O dispositivo remoto apresenta a origem da solicitação e as **permissões solicitadas**.
4. A pessoa aceita ou rejeita; ao aceitar, vê indicação persistente de sessão ativa e pode alterar/revogar permissões.
5. A sessão começa somente após autenticação, autorização e negociação de canal seguro.
6. O técnico executa apenas as ações concedidas; arquivos, clipboard, gravação e elevação exigem políticas específicas.
7. Qualquer participante autorizado pode encerrar a sessão; o host conserva um registro mínimo auditável.

### Sessão não assistida

**Desabilitada por padrão.** O proprietário ou administrador autorizado deve realizar a configuração no dispositivo, confirmar o vínculo, definir contas e permissões elegíveis, habilitar autenticação multifator conforme política e receber meios de **revogar o acesso a qualquer momento**. O funcionamento após reinicialização requer instalação e serviço apropriado. Não são previstos mecanismos para disfarçar a execução ou contornar avisos do sistema operacional.

## Plataformas e restrições

| Plataforma | Cliente controlador | Host controlado | Observações |
| --- | --- | --- | --- |
| Windows 10/11 | Planejado | Planejado | Serviço para cenários que exigem execução antes do login, com UAC e privilégios tratados de forma segura |
| Windows Server | Planejado | Planejado | Suporte condicionado a testes por versão/edição e políticas corporativas |
| Linux | Planejado | Planejado | X11 e Wayland exigem caminhos de captura/entrada distintos; priorizar portais oficiais |
| macOS | Planejado | Planejado | Requer autorizações do sistema para gravação de tela, acessibilidade e entrada |
| Android | Planejado | Visualização/controle apenas quando permitido pela plataforma | Hospedagem e injeção de entrada dependem de APIs/permissões suportadas |
| iOS/iPadOS | Planejado | Compartilhamento compatível com APIs oficiais | Controle remoto irrestrito de terceiros não deve ser prometido |
| Navegador | Futuro | Não previsto inicialmente | Cliente web requer avaliação de segurança e restrições de dispositivos |

**Nenhuma plataforma está homologada no estado inicial.** Cada combinação cliente/host terá matriz de compatibilidade, testes e limitações documentadas.

## Experiência de usuário (diretrizes)

- Tela inicial com **Seu ID**, botão de cópia, estado de disponibilidade, endereço de destino e conexão rápida.
- Áreas de **Recentes**, **Favoritos**, **Dispositivos**, **Convites** e **Configurações**.
- Janela de consentimento clara e não enganosa, exibindo solicitante, dispositivo, prazo e permissões.
- Barra de sessão com monitores, resolução, escala, FPS, qualidade, entrada, clipboard, arquivos, chat e encerrar.
- Estados claros para offline, conectando, aguardando aceite, autenticando, conectado, reconectando, recusado e erro.
- Tema claro/escuro, PT-BR como idioma inicial, internacionalização, navegação por teclado e leitores de tela.
- Mensagens acionáveis para firewall, proxy, permissões do sistema, codec incompatível e indisponibilidade de relay.

## Arquitetura proposta

```text
┌─────────────────────┐         ┌─────────────────────────┐
│ Seven Desk Desktop  │         │ Seven Desk Desktop      │
│ Operador / Viewer   │         │ Dispositivo / Host      │
│ React + TypeScript  │         │ React + TypeScript      │
│ Tauri + Rust        │         │ Tauri + Rust nativo     │
└──────────┬──────────┘         └──────────┬──────────────┘
           │                              │
           └───── Identidade / Sinalização ┘
                         │
               ┌─────────▼──────────┐
               │ Control Plane      │
               │ Rust + Axum        │
               │ Auth, pares, ACL,  │
               │ sessões e auditoria│
               └─────────┬──────────┘
                         │
        Canal de mídia/dados ponto a ponto quando viável
        ou relay autenticado quando necessário
```

| Camada | Proposta | Responsabilidade |
| --- | --- | --- |
| Desktop UI | React + TypeScript + Tauri | Interface, estado de sessão, acessibilidade e interação |
| Núcleo nativo | Rust | Captura, entrada, codec, transporte, políticas locais e IPC tipado |
| Serviço do host | Rust | Disponibilidade, identidade do dispositivo, permissões, reconexão e integração com SO |
| Control plane | Rust + Axum | Registro, presença, sinalização, autorização, diretório e eventos |
| Transporte | WebRTC/ICE com STUN/TURN **ou** QUIC com relay próprio, após prova de conceito | Fluxo de mídia, entrada e dados com criptografia e negociação de capacidades |
| Persistência central | PostgreSQL | Contas, organizações, dispositivos, políticas e registros de auditoria |
| Estado efêmero | Redis (quando justificado) | Presença, rate limit, filas efêmeras e coordenação |
| Observabilidade | Logs estruturados, métricas e tracing | Diagnóstico com minimização de dados sensíveis |

**Decisões ainda abertas:** transporte definitivo, codec por plataforma, modelo criptográfico E2EE auditável, topologia de relay, empacotamento do serviço e infraestrutura gerenciada/self-hosted. Devem ser decididas por ADRs, protótipos e testes reproduzíveis — não por suposição.

### Componentes sugeridos do monorepo

```text
apps/
  desktop/                 # Interface e empacotamento Tauri
  server/                  # API de identidade, sinalização e gestão
crates/
  protocol/                # Tipos compartilhados e versionamento
  capture/                 # Captura por plataforma
  input/                   # Entrada por plataforma
  media/                   # Codec, bitrate e métricas
  transport/               # Comunicação segura e relay
  host-service/            # Serviço do endpoint
  security/                # Criptografia, autenticação e políticas
packages/
  ui/                      # Design system
  contracts/               # Contratos TypeScript gerados
docs/
  adr/                     # Decisões arquiteturais
  security/                # Modelo de ameaças e hardening
tests/
  e2e/                     # Fluxos ponta a ponta
  interoperability/        # Matriz sistema operacional / rede
```

> Esta árvore é **proposta de organização** e não representa diretórios já existentes no repositório.

## Segurança e privacidade

**Diretriz: nenhuma conexão deve conceder acesso apenas por conhecer o ID do dispositivo.**

- **Autenticação:** dispositivos com chaves individuais; autenticação de contas; MFA para acesso não assistido e operações sensíveis.
- **Autorização:** verificação no servidor **e no host**, menor privilégio, papéis explícitos, políticas por dispositivo e por sessão.
- **Criptografia:** protocolos modernos validados; autenticar pares e negociar canais de mídia/dados protegidos; não inventar criptografia própria.
- **Segredos:** usar armazenamento seguro do SO; não registrar tokens, senhas, conteúdo de tela, clipboard ou arquivos em logs.
- **Consentimento:** autorização humana em sessões assistidas, sem elevação silenciosa; sinalização persistente da sessão.
- **Resiliência a abuso:** expiração de convites, rate limit, bloqueio progressivo, revogação imediata, proteção contra replay e verificação de identidade.
- **Auditoria:** início/fim, participantes, eventos de autorização e permissões; evitar registrar o conteúdo da sessão por padrão.
- **Gravação:** desabilitada por padrão, aviso para todas as pessoas envolvidas, política de retenção e controles de acesso.
- **Atualizações:** distribuição assinada, verificação de integridade, canal de release e rollback seguro.
- **LGPD:** finalidade definida, minimização, retenção configurável, direitos do titular, bases legais e documentação de tratamento.

**Aviso de transparência:** um indicador de acesso ativo e o mecanismo de encerramento pelo host são requisitos de segurança obrigatórios. Acesso legítimo não equivale a operação oculta.

Para relatar vulnerabilidades, não publique credenciais, exploit funcional ou dados de terceiros em issues públicas. Utilize um canal privado de divulgação responsável a ser definido antes do beta público.

## Requisitos não funcionais e metas de validação

As metas abaixo são **propostas de aceitação**, não medições obtidas.

| Dimensão | Critério de validação |
| --- | --- |
| Latência | Medir entrada→resposta visual em LAN e WAN, incluindo p50/p95 e jitter |
| Vídeo | Perfil adaptativo com métricas de FPS efetivo, resolução, bitrate, perdas e uso de CPU |
| Redes | Validar NAT residencial, CGNAT, proxy, firewall, reconexão, TURN/relay e dual-stack |
| Consumo | Perfis de CPU, memória, GPU, disco e rede em equipamentos modestos |
| Segurança | Testes de autenticação, autorização por operação, replay, revogação e isolamento de organizações |
| Confiabilidade | Testes de perda temporária de conexão, troca de rede, encerramento abrupto e atualização |
| Acessibilidade | Teclado, foco, rótulos acessíveis, contraste e leitores de tela |
| Distribuição | Instalador/desinstalador, upgrade, assinatura e permissões verificadas por SO |
| Qualidade | Unitários, integração, E2E, interoperabilidade, fuzzing de protocolo e auditoria de dependências |

## Roadmap

Planejamento organizado por entrega incremental, sem datas ou versões fictícias.

| Fase | Entregáveis | Condição de saída |
| --- | --- | --- |
| **P0 — Fundação** | Monorepo, contratos, ADRs, CI, design system, identidade de dispositivo e protótipo de transporte | Build reproduzível, testes básicos e threat model |
| **P1 — MVP assistido** | Windows↔Windows, conexão por ID, aceite, vídeo, teclado/mouse, desconexão e indicadores | E2E em duas máquinas reais, permissão negada e logs auditáveis |
| **P2 — Beta multiplataforma** | Linux/macOS, relay, transferência, clipboard, contatos, reconexão e instaladores | Matriz de SO/rede, testes de segurança e beta controlado |
| **P3 — Operação profissional** | Não assistido, MFA, políticas, serviço, chat, múltiplos monitores e gestão de dispositivos | Fluxos negativos testados e auditoria de acessos |
| **P4 — Enterprise / 1.0** | Organizações/RBAC, SSO, relatórios, atualização assinada, observabilidade e documentação operacional | Release checklist completo, requisitos críticos fechados e rollback testado |
| **P5 — Expansão** | Viewer móvel, impressão remota, Wake-on-LAN, áudio, gravação consentida e infraestrutura própria | Homologação individual por sistema e funcionalidade |

O roadmap detalhado vive nas [issues do projeto](https://github.com/gabriell211/seven-desk/issues). **Uma versão 1.0 só deve ser publicada após cumprir seus critérios de homologação.**

## Desenvolvimento e contribuição

### Pré-requisitos previstos

- Rust stable e Cargo (toolchain fixada em arquivo do projeto quando o código existir).
- Node.js LTS e pnpm com versões fixadas em `package.json`/`packageManager`.
- Toolchains nativas exigidas por Tauri e pelos módulos de captura de cada sistema.
- PostgreSQL para serviços centrais; Redis apenas se necessário para o caso de uso.

**Ainda não existem comandos de build, instalação ou servidor verificáveis.** Eles serão publicados com o código inicial, sem instruções fictícias.

### Fluxo de contribuição

1. Verifique [issues abertas](https://github.com/gabriell211/seven-desk/issues) e dependências.
2. Discuta mudanças de protocolo, segurança ou arquitetura antes de implementá-las.
3. Crie branch curta e pull request com contexto, riscos, mudanças e evidência de testes.
4. Exija testes unitários, integração e E2E proporcionais ao comportamento alterado.
5. Não realize merge de mudanças críticas sem revisão de segurança e CI verde.
6. Nunca inclua arquivos de terceiros, marcas, chaves privadas ou amostras de sessões reais sem autorização.

### Convenções sugeridas

- Commits convencionais: `feat:`, `fix:`, `docs:`, `test:`, `refactor:`, `security:`.
- Rust: `cargo fmt`, `cargo clippy`, `cargo test`, análise de dependências.
- TypeScript: `strict: true`, lint, typecheck, testes de componentes e fluxos E2E.
- Segurança: revisão de ameaças para endpoints, autenticação, IPC e capacidades remotas.
- API: contratos versionados, validação estrita, paginação, limites e documentação.
- CI: pipelines específicos por Windows/macOS/Linux e artefatos assinados em releases.

## Definição de pronto (DoD)

Uma funcionalidade só pode ser marcada como concluída quando possuir:
- [ ] Implementação real sem caminhos mockados em produção.
- [ ] Testes de sucesso, falha, autorização negada e entradas inválidas.
- [ ] Verificação de segurança, privacidade e acessibilidade aplicáveis.
- [ ] Logs e erros compreensíveis sem vazamento de informações sensíveis.
- [ ] Compatibilidade com sistemas/plataformas declarados.
- [ ] Documentação atualizada, revisão aprovada e CI verde.
- [ ] Demonstração ou evidência reproduzível do resultado.

## Licenciamento e uso

**Licença ainda não definida.** A disponibilidade pública do código não concede automaticamente permissão para redistribuição, modificação ou uso além do permitido pela legislação aplicável. A licença deverá ser escolhida e adicionada ao repositório antes de divulgar releases.

O Seven Desk destina-se exclusivamente a **acesso autorizado a equipamentos**. Operações em dispositivos de terceiros exigem consentimento ou base legítima e devem seguir as políticas aplicáveis.

---

<div align="center">

**Seven Desk** · Acesso remoto projetado para ser simples de usar, sólido de operar e seguro por padrão.

**Repositório:** https://github.com/gabriell211/seven-desk

</div>
