<!-- Seven Desk | Documentação oficial de visão e requisitos -->
<div align="center">

# Seven Desk

### Acesso remoto seguro, rápido e sob seu controle.

**Plataforma multiplataforma de assistência remota, administração de dispositivos e colaboração em tempo real.**

[Visão geral](#visão-geral) · [Funcionalidades](#funcionalidades) · [Arquitetura](#arquitetura-proposta) · [Infraestrutura](docs/INFRAESTRUTURA.md) · [Segurança](#segurança-e-privacidade) · [Roadmap](docs/ROADMAP.md) · [Contribuição](#desenvolvimento-e-contribuição)

</div>

> [!IMPORTANT]
> **Status: desenvolvimento pré-alfa.** Já existem contratos Rust, identidade assinada, sinalização de sessão, interface Tauri/React e **captura nativa experimental no Windows, macOS e Linux** com publicação manual PNG ou prévia JPEG de até 15 quadros/segundo. O aplicativo registra o dispositivo automaticamente quando o servidor está disponível; cada pedido de acesso abre uma janela separada de aceite com permissões. Ao aceitar, o host inicia o compartilhamento e os controles concedidos, depois minimiza apenas essa janela de atendimento. A janela principal permanece aberta. Ao restaurar a janela de atendimento, ela mostra **Finalizar**. O operador abre a tela remota automaticamente. Linux Wayland depende dos portais disponíveis na sessão gráfica e não oferece entrada remota nesta versão. O mouse remoto continua restrito ao monitor principal. O teclado aceita caracteres e algumas teclas de edição/navegação, sem atalhos. Há **chat textual temporário** mediante autorização do host, ainda legível pelo servidor de laboratório. **Ainda não há vídeo profissional, E2EE, serviço host completo nem plataforma homologada em E2E/produção.** Veja [o status técnico](docs/IMPLEMENTACAO.md) e as [issues](https://github.com/gabriell211/seven-desk/issues).

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

A tabela a seguir é o **escopo-alvo**, não uma lista de funcionalidades prontas. A [matriz de rastreabilidade](docs/MATRIZ_FUNCIONAL.md) associa **cada capacidade** às respectivas issues e fases.

| Área | Recursos previstos |
| --- | --- |
| Conexões | ID público do dispositivo, apelido, convite temporário, favoritos, histórico, acesso LAN/WAN, descoberta opt-in e proxy corporativo |
| Sessão assistida | Solicitar conexão, aceitar/rejeitar, escolher permissões, suspender controle e encerrar |
| Sessão não assistida | Habilitação explícita pelo proprietário/administrador, credenciais fortes, MFA, restrições por dispositivo e revogação |
| Visualização | Compartilhamento de tela, seleção de monitor, escalonamento, tela cheia, ajuste dinâmico de qualidade e FPS |
| Controle | Mouse, teclado, atalhos, mapeamento de teclado, teclas especiais, suporte a layouts e permissões granularizadas |
| Transferência | Arquivos bidirecionais, gerenciador de dois painéis sem streaming de tela, arrastar e soltar quando suportado, progresso, integridade e cancelamento |
| Área de transferência | Texto e, em fases posteriores, imagens/arquivos, sempre sujeitos a permissão |
| Comunicação | Chat de sessão, convites e mensagens de estado |
| Colaboração | Modo somente visualização, anotações temporárias, sessões com múltiplos participantes autorizados |
| Gerenciamento | Contatos, grupos, tags, pesquisa, dispositivos confiáveis, favoritos e políticas |
| Operação | Reinicialização remota autorizada, reconexão, serviço nativo independente, terminal opt-in e Wake-on-LAN futuro |
| Avançado | Tunelamento TCP controlado, VPN opcional futura, impressão remota, áudio remoto, gravação consentida, modo de privacidade e API de integração |
| Empresas | Organizações, RBAC, SSO/OIDC, namespaces/aliases, auditoria, políticas, provisionamento gerenciado e infraestrutura própria futura |
| Distribuição | Instalação oficial pelo **Instalador Seven Desk**, com identidade própria, atualização assinada e desinstalação limpa |

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
| Linux | Experimental | Experimental | Captura X11/Wayland via XCap; Wayland depende do portal e não oferece entrada remota nesta versão; mouse e teclado apenas em X11 |
| macOS | Experimental | Experimental | Captura via AVFoundation; Gravação de Tela e Acessibilidade exigem consentimento do sistema |
| Android | Planejado | Visualização/controle apenas quando permitido pela plataforma | Hospedagem e injeção de entrada dependem de APIs/permissões suportadas |
| iOS/iPadOS | Planejado | Compartilhamento compatível com APIs oficiais | Controle remoto irrestrito de terceiros não deve ser prometido |
| Navegador | Futuro | Não previsto inicialmente | Cliente web requer avaliação de segurança e restrições de dispositivos |

**Nenhuma plataforma está homologada no estado inicial.** Cada combinação cliente/host terá matriz de compatibilidade, testes e limitações documentadas.

> **Suporte de sistema operacional:** o suporte regular do Windows 10 terminou em 14/10/2025. Sua compatibilidade pode ser testada, mas o Seven Desk só poderá declarar suporte produtivo a versões/edições com cobertura de atualizações de segurança adequada (como ESU válida, onde aplicável). Windows 11 é a referência principal para o MVP. Veja [a matriz de homologação](docs/HOMOLOGACAO.md) e [o inventário de compatibilidade](docs/MATRIZ_FUNCIONAL.md).

## Instalação oficial — Instalador Seven Desk

A distribuição oficial do aplicativo desktop deverá ser feita pelo **Instalador Seven Desk**, identificado com o nome, ícone, editora e versão do próprio produto. **O instalador, e não uma pasta de executáveis soltos ou scripts manuais, será o caminho padrão de instalação para usuários finais.**

| Sistema | Formato de instalação planejado | Comportamento |
| --- | --- | --- |
| Windows | `SevenDesk-Setup-<versão>.exe` (instalador com assistente) | Instalar e registrar Seven Desk, criar atalhos e desinstalador; solicitar elevação apenas quando necessária |
| Linux | Pacotes nativos `.deb` e `.rpm` conforme distribuição homologada | Instalar pelo gerenciador de pacotes, registrar aplicativo e permitir remoção limpa |
| macOS | Pacote oficial `.dmg` (padrão Tauri), com `.pkg` adicional se necessário | Instalar o aplicativo assinado/notarizado e seguir autorizações nativas do sistema |

**Fluxo de instalação previsto:** executar o pacote oficial → verificar origem e assinatura → conhecer versão e termos aplicáveis → escolher opções permitidas → instalar os componentes → criar os atalhos → abrir o Seven Desk e configurar permissões do host. Atalhos, inicialização com o sistema, instalação de serviço e acesso não assistido devem respeitar as escolhas e políticas autorizadas.

**Requisitos obrigatórios de distribuição:**

- Identidade visual do Seven Desk no assistente, aplicativo, atalhos, lista de programas instalados e desinstalador.
- Versão, fabricante/editor e metadados consistentes; manifestos e hashes publicados nos releases.
- Instalador assinado/verificado por plataforma, com rollback quando viável e mensagens claras em falhas.
- Atualização de versões anteriores sem perder configurações válidas ou duplicar serviços; desinstalação limpa.
- Instalação interativa por padrão; instalações administradas em organizações exigem políticas e documentação específicas.
- Não incorporar credenciais, ID de outra máquina, acesso persistente ativo por padrão ou consentimento presumido.

**Referências de engenharia:** [pesquisa de repositórios públicos e estratégia dos instaladores](docs/REFERENCIAS_TECNICAS.md).

**Situação atual:** a distribuição de teste `v0.1.0-alpha.1` reúne pacotes nativos não assinados para Windows, macOS e Linux, com hashes publicados. O EXE do Windows tem arte própria e barra de progresso colorida. Os instaladores ainda **não foram homologados nem assinados**. Consulte as [notas do pré-lançamento](docs/RELEASE_0.1.0_ALPHA_1.md) antes de instalar. O requisito de distribuição oficial está rastreado nas issues [#15](https://github.com/gabriell211/seven-desk/issues/15), [#23](https://github.com/gabriell211/seven-desk/issues/23) e [#38](https://github.com/gabriell211/seven-desk/issues/38). Uma distribuição portátil não faz parte do fluxo de instalação oficial e, se for considerada no futuro, exigirá escopo e aprovação separados.

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

**Decisões ainda abertas:** transporte definitivo, codec por plataforma, modelo criptográfico E2EE auditável, topologia de relay, empacotamento do serviço e infraestrutura gerenciada/self-hosted. O código original do projeto adota a licença AGPL-3.0-only; a auditoria de licenças de terceiros continua pendente. Devem ser decididas por ADRs, protótipos e testes reproduzíveis — não por suposição. Leia a [arquitetura proposta](docs/ARQUITETURA.md).

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

Para reportar vulnerabilidades, não publique credenciais, detalhes exploráveis nem dados de terceiros em issues públicas. Consulte [SECURITY.md](SECURITY.md) para as orientações de divulgação responsável.

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
| Distribuição | Instalador Seven Desk homologado por SO, instalação, upgrade, desinstalação e assinatura verificáveis |
| Qualidade | Unitários, integração, E2E, interoperabilidade, fuzzing de protocolo e auditoria de dependências |

## Roadmap

Planejamento organizado por entrega incremental, sem datas ou versões fictícias.

| Fase | Entregáveis | Condição de saída |
| --- | --- | --- |
| **P0 — Fundação** | Monorepo, contratos, ADRs, CI, design system, identidade de dispositivo e protótipo de transporte | Build reproduzível, testes básicos e threat model |
| **P1 — MVP assistido** | Windows↔Windows, conexão por ID, aceite, vídeo, teclado/mouse, instalador Seven Desk para Windows e indicadores | E2E em duas máquinas reais, permissão negada e logs auditáveis |
| **P2 — Beta multiplataforma** | Linux/macOS, relay, transferência, clipboard, contatos, reconexão e instaladores | Matriz de SO/rede, testes de segurança e beta controlado |
| **P3 — Operação profissional** | Não assistido, MFA, políticas, serviço, chat, múltiplos monitores e gestão de dispositivos | Fluxos negativos testados e auditoria de acessos |
| **P4 — Enterprise / 1.0** | Organizações/RBAC, SSO, relatórios, atualização assinada, observabilidade e documentação operacional | Release checklist completo, requisitos críticos fechados e rollback testado |
| **P5 — Expansão** | Viewer móvel, impressão remota, Wake-on-LAN, áudio, gravação consentida e infraestrutura própria | Homologação individual por sistema e funcionalidade |

O [roadmap operacional](docs/ROADMAP.md) reúne as **73 issues** por fase e seus gates de homologação. Veja a [matriz funcional](docs/MATRIZ_FUNCIONAL.md), os [critérios de homologação](docs/HOMOLOGACAO.md) e a [auditoria de alinhamento](docs/AUDITORIA_ALINHAMENTO.md); acompanhe também as [issues do projeto](https://github.com/gabriell211/seven-desk/issues). **Uma versão 1.0 só deve ser publicada após cumprir seus critérios de homologação.**

## Desenvolvimento e contribuição

### Pré-requisitos de desenvolvimento atuais

- Rust **1.90.0** e Cargo (toolchain fixada em `rust-toolchain.toml`).
- Node.js **22 LTS** e npm. O monorepo contém também `pnpm-workspace.yaml`, mas os scripts e o CI atuais usam **npm workspaces**.
- Toolchains nativas do Tauri conforme o sistema operacional; consulte [o guia de implementação](docs/IMPLEMENTACAO.md).
- O backend experimental usa armazenamento temporário em memória; **PostgreSQL e Redis não são exigidos para executar a pré-alfa local**.

**Comandos de desenvolvimento e compilação do código pré-alfa** estão registrados em [docs/IMPLEMENTACAO.md](docs/IMPLEMENTACAO.md). A execução dos testes do núcleo Rust, o build React e o cargo check Tauri foram validados pelo CI. Os instaladores definitivos e a conexão remota real ainda não foram homologados. O [guia de contribuição](CONTRIBUTING.md) descreve como validar mudanças e reportar evidências.

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

**Licença do código original: GNU Affero General Public License v3.0 (AGPL-3.0-only).** Consulte o texto integral em [LICENSE](LICENSE) e as orientações em [docs/LICENCIAMENTO.md](docs/LICENCIAMENTO.md). É permitido utilizar, estudar, modificar e redistribuir o software, inclusive comercialmente, respeitando as condições da AGPLv3. Distribuições de versões modificadas e serviços que permitam interação remota pela rede devem cumprir as obrigações aplicáveis de disponibilização do código-fonte correspondente.

**Componentes de terceiros:** bibliotecas, APIs, codecs e materiais importados permanecem sob suas próprias licenças e devem ser auditados antes de releases. A adoção da AGPL não autoriza copiar qualquer código ou marca de terceiros sem respeitar seus avisos e permissões. Consulte também a [issue #67](https://github.com/gabriell211/seven-desk/issues/67).

O Seven Desk destina-se exclusivamente a **acesso autorizado a equipamentos**. Operações em dispositivos de terceiros exigem consentimento ou base legítima e devem seguir as políticas aplicáveis.

---

<div align="center">

**Seven Desk** · Acesso remoto projetado para ser simples de usar, sólido de operar e seguro por padrão.

**Repositório:** https://github.com/gabriell211/seven-desk

</div>
