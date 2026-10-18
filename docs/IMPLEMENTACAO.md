# Seven Desk — Estado técnico das entregas executáveis

Esta documentação descreve o **código que está começando a existir**. Não declarar captura de tela, controle remoto ou acesso não assistido como funcionais até que estejam integrados e testados.

## Componentes adicionados

- `crates/protocol`: contratos versionados e máquina de estados fail-closed de sessão; testes unitários de aceite, rejeição, revogação, expiração e escalada de permissões.
- `crates/security`: verificação de assinaturas Ed25519 para solicitações, vinculação de identidade à chave pública e proteção contra nonce reutilizado.
- `crates/identity`: geração criptográfica de identidade local persistente com armazenamento seguro nativo (quando disponível), sem colocar a chave privada na interface.
- `apps/server`: API de desenvolvimento Rust/Axum com registro de identidade (proof of possession), convites de sessão e decisões verificadas. Armazena dados em memória e restringe bind remoto.
- `apps/desktop`: aplicativo React 19 + TypeScript + Tauri 2 com interface, navegação, estado local nativo e preferências visuais, sem simular sessões.
- `scripts/generate-icons.mjs`: geração determinística de ícones binários para os três instaladores desktop usando apenas Node.js.
- `.github/workflows/ci.yml`: formato, lint, testes Rust, TypeScript/build e checagem nativa Tauri.
- `.github/workflows/installers.yml`: geração **manual de pacotes de QA não assinados**. Não são releases finais.

Para instalação local reproduzível sem Docker, fixture de teste, validação HTTP real do backend e preview React em ambiente limpo, consulte **[Ambiente local](AMBIENTE_LOCAL.md)**. O smoke comprova bootstrap e controles básicos, não acesso remoto de produção.

## Pré-requisitos de desenvolvimento

- Rust 1.90.0 e Cargo.
- Node.js 22 LTS e npm. Sem Docker obrigatório.
- Windows: dependências nativas exigidas pelo Tauri 2, MSVC Build Tools e WebView2 quando aplicável.
- Linux: WebKit2GTK, librsvg, appindicator, libdbus-1-dev, pkg-config e dependências de sistema; consulte a documentação Tauri.
- macOS: Xcode Command Line Tools.

## Comandos

```bash
npm install
npm run desktop:dev             # preview da UI React no navegador
npm run tauri:dev               # desktop nativo (gera ícones automaticamente)
npm run desktop:build           # typecheck + Vite build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p seven-server       # API dev, apenas loopback 127.0.0.1:43821
```

O serviço backend expõe `GET /healthz`, registro, convites, autorização, revogação, quadros PNG e pausas de entrega protegidos por assinatura. Não iniciar controle de dispositivo sem handshake, autorização do host e implantação segura.

## Protocolo de assinatura (versão 1)

Headers: `X-Seven-Key` (hex da chave pública Ed25519), `X-Seven-Signature` (assinatura hex), `X-Seven-Nonce` (UUID), `X-Seven-Timestamp` (Unix ms).

Bytes assinados:

```text
seven-desk/request/v1
<METODO_HTTP>
<CAMINHO_EXATO>
<TIMESTAMP_MS>
<NONCE_UUID>
<SHA256_HEX_DO_CORPO>
```

A chave identifica o dispositivo por UUID derivado do primeiro bloco de 16 bytes de SHA-256 da chave pública. O servidor verifica timestamp dentro de 60 segundos, validade da assinatura e rejeita reuso de nonce. **Isto é um protocolo inicial de pesquisa, não uma especificação auditada para produção.**

`POST /v1/devices` aceita exatamente `{}` assinado. A sessão requer ambos os dispositivos cadastrados e operador autenticado. A decisão é exclusivamente do host. Após o aceite, a sessão tem prazo próprio (até 8 horas inicialmente) separado da expiração do convite. O host pode revogar capacidades por `POST /v1/sessions/{id}/revoke` com assinatura. Existe um **canal experimental de PNGs recentes**, exclusivamente após autorização do host, mas **não há vídeo E2EE nem transporte de input**. O backend informa `production_ready: false`.

## Evolução validada em 17/10/2026

Três entregas adicionais foram integradas à `main` após passar pelos cinco jobs disponíveis nos workflows `Check` e `Seven Desk CI`:

- [PR #78](https://github.com/gabriell211/seven-desk/pull/78): inbox de solicitações autenticadas `GET /v1/sessions/inbox`, restrita ao host, com expiração e rejeição sem exposição a terceiros.
- [PR #79](https://github.com/gabriell211/seven-desk/pull/79): registro de presença por 90 segundos, heartbeat assinado `POST /v1/devices/heartbeat` e consulta `GET /v1/devices/{id}/presence`; convites são rejeitados para hosts offline.
- [PR #80](https://github.com/gabriell211/seven-desk/pull/80): Tauri registra e renova a presença via comandos Rust nativos assinados, acionados voluntariamente pela interface; nenhum segredo de assinatura é enviado ao React.

A documentação experimental dos endpoints está em [API_DESENVOLVIMENTO.md](API_DESENVOLVIMENTO.md). O backend continua **local, em memória e não preparado para produção**. Não houve homologação visual ou E2E em duas máquinas; os builds e testes de CI não devem ser apresentados como validação de conexão remota funcional.

## Integração funcional da negociação — 17/10/2026

- [PR #81](https://github.com/gabriell211/seven-desk/pull/81) — **merged, CI 5/5**: comandos nativos Tauri de criação de convite, leitura da fila, aceite/recusa do host, consulta de estado e encerramento; interface React usa essas operações assinadas de verdade, com solicitação limitada à capacidade `screen`.
- [PR #82](https://github.com/gabriell211/seven-desk/pull/82) — **merged, CI 5/5**: host consegue listar todas as autorizações ativas destinadas à sua identidade, inclusive após reabrir a janela, e encerrá-las. Testes HTTP impedem listagem por operador/terceiros e conferem revogação.

**Situação posterior a #82:** foi implementado um caminho de pixels reais de captura manual Windows e de quadros repetidos voluntariamente (~1 FPS), além de URL HTTPS de desenvolvimento configurável. O servidor ainda escuta em `127.0.0.1`, não tem E2EE nem serviços permanentes; testes de CI não substituem ensaios de dois desktops reais. A infraestrutura segura e o serviço host seguem como prioridades.

## Captura e controles de tela — 17/10/2026

- [PR #83](https://github.com/gabriell211/seven-desk/pull/83) — buffer temporário de quadros PNG por sessão, limite de 2 MiB, acesso assinado e validado pelo host e pelo operador.
- [PR #84](https://github.com/gabriell211/seven-desk/pull/84) — captura manual de um quadro real no Windows e exibição autorizada no desktop.
- [PR #85](https://github.com/gabriell211/seven-desk/pull/85) — endereço do control plane configurável para testes privados com HTTPS obrigatório fora do loopback.
- [PR #86](https://github.com/gabriell211/seven-desk/pull/86) — atualização voluntária de tela de até ~1 FPS, com parada local e visualização de quadros recentes.
- [PR #87](https://github.com/gabriell211/seven-desk/pull/87) — recuperação da sessão autorizada do operador após reabrir a interface, com isolamento por identidade assinada.
- [PR #88](https://github.com/gabriell211/seven-desk/pull/88) — comentários de implementação corrigidos, sem mudar funcionalidades.
- [PR #89](https://github.com/gabriell211/seven-desk/pull/89) — elimina quadros privados da memória ao revogar ou encerrar e serializa verificação de autorização e publicação para não reintroduzir quadros após o encerramento.
- [PR #90](https://github.com/gabriell211/seven-desk/pull/90) — interruptor host-only para pausar/retomar entrega de quadros, limpa cache e rejeita novas leituras/uploads enquanto pausado; implementação Tauri e UI, testes e CI 6/6 aprovados.

**Validação:** os PRs #87–#90 estão integrados. Todos os workflows verificados para #89 e #90 tiveram seis jobs aprovados. Isso confirma compilação e testes automatizados, **não** usabilidade E2E ou produção. Ainda não há controle remoto completo, E2EE, codificação de vídeo profissional, instalação de serviço host autônomo nem assinatura/homologação de instaladores.

## Canal de mouse interativo — 17/10/2026

- [PR #91](https://github.com/gabriell211/seven-desk/pull/91) **merged, CI 6/6**: contrato normalizado de eventos de mouse, fila assinada operador→host, rate-limit, TTL curto, isolamento por papel/permissão e descarte imediato ao revogar ou pausar.
- [PR #92](https://github.com/gabriell211/seven-desk/pull/92) **merged, CI 6/6**: Tauri Windows aplica mouse via Enigo ao monitor principal somente após permissão `input` escolhida pelo host e habilitação local explícita. Interface envia movimento e clique com controle de taxa; host pode parar mouse e compartilhamento a qualquer momento.

**Estado:** há caminhos experimentais de código para mouse e teclado básico entre dois clientes autenticados via servidor HTTPS de laboratório, mas **não existe homologação prática entre dois equipamentos** e o backend ainda não implementa E2EE, buffer eficiente de vídeo nem serviço host permanente. Atalhos/modificadores, validação de layouts regionais, monitores secundários, arraste, proteção contra desktop elevado e benchmarks ainda são pendências. A entrada não habilita acesso não assistido.

## Teclado básico consentido e correção da assinatura de quadros — 17/10/2026

- O protocolo separa `keyboard` de `input` (mouse). O operador solicita cada permissão, o host escolhe quais conceder e ativa a injeção local de teclado com uma ação adicional durante o compartilhamento de tela. Pausa, revogação, janela oculta e término interrompem a entrada. O conjunto de teclas é limitado a caracteres, edição e navegação sem modificadores ou atalhos.
- O servidor mantém uma fila temporária e limitada de teclado, isolada por sessão. Eventos exigem assinatura, identidade de participante, permissão vigente e entrega de tela ativa. Pausa, revogação e término limpam a fila.
- A assinatura de requisições agora aceita o mesmo limite de 2 MiB do endpoint PNG. Antes, o limite de 64 KiB na camada de assinatura impedia o envio da maioria dos quadros reais. Um teste de API exercita publicação e leitura de quadro assinado acima de 64 KiB.

As verificações locais com Rust 1.90 passaram: formatação, Clippy sem avisos e 33 testes do workspace, incluindo permissão/revogação de teclado e quadro assinado acima de 64 KiB. O build TypeScript/Vite também passou. **Ainda faltam compilação nativa Tauri final, E2E em duas máquinas, testes de layout do teclado, avaliação de segurança e vídeo de baixa latência**; isso não representa paridade com o AnyDesk nem autorização para release de produção.

## O que ainda bloqueia uso real

1. Implementar serviço host real e permanente, rotação/revogação de identidades e operação multi-dispositivo. O aplicativo já registra presença e permite captura e entrada básicas por sessão consentida no laboratório, mas depende da janela aberta e não sustenta acesso remoto contínuo.
2. Authn/authz final, persistência PostgreSQL, limpeza de sessões e proteção contra DoS.
3. Comunicação de controle protegida ponta a ponta (P2P/relay), codecs com baixa latência e **teclado/mouse** com APIs nativas. A captura Windows e a prévia PNG/JPEG por HTTP são experimentais; não equivalem ao motor de vídeo pronto.
4. Serviço do host/IPC e atualização assinada do aplicativo.
5. E2E em duas máquinas reais e testes de segurança cross-platform.
6. Escolha explícita da licença do Seven Desk; o código provisório usa `UNLICENSED`.

## Chat por sessão no laboratório — 17/10/2026

Foi integrado um canal de chat textual opcional ao convite e à interface Tauri. O host concede `chat` separadamente; o servidor valida a permissão em cada envio/leitura, limita o histórico temporário e elimina as mensagens ao encerrar ou revogar. O conteúdo ainda é legível pelo servidor de laboratório (mesmo quando o transporte usa HTTPS), portanto **não representa chat seguro de produção nem paridade funcional com o AnyDesk**.

## Seleção de monitor Windows no laboratório — 17/10/2026

O host pode listar os monitores disponíveis e escolher localmente qual será capturado na próxima publicação de quadro. Uma troca de monitor interrompe o envio periódico, apaga o quadro no servidor quando alcançável e desliga o mouse remoto no processo nativo; o host precisa iniciar os quadros novamente. A captura falha se o monitor escolhido desaparecer ou se o estado de monitor principal mudar durante a captura. **Monitor secundário é somente visualização/teclado nesta fase**: Enigo ainda mapeia coordenadas absolutas de mouse apenas no monitor principal. Compilação Windows passou, mas troca de monitor e DPI ainda exigem teste em uma máquina com múltiplos monitores.

## Endereço de servidor configurável no laboratório — 17/10/2026

O usuário pode salvar uma URL base no aplicativo instalado. A validação nativa exige HTTPS fora de `localhost`/loopback, rejeita credenciais/caminhos/parâmetros e mantém prioridade para `SEVEN_DESK_DEV_SERVER_URL` quando definida. Ao mudar de servidor, a interface para captura e entrada locais, tenta revogar sessões no servidor anterior, limpa os dados exibidos e exige novo registro. Falhas de revogação são avisadas e as sessões temporárias expiram pelo TTL do backend. O endereço salvo não altera a limitação de segurança ou disponibilidade do servidor de desenvolvimento.

## Verificação local desta revisão — 17/10/2026

Na revisão anterior, passaram **38 testes Rust do workspace**, **4 testes da biblioteca Tauri**, build TypeScript/Vite, smoke da interface e **7 checagens de sessão assinada** contra o servidor local. O build Tauri Windows gerou MSI e NSIS de QA. O instalador Windows está **sem assinatura digital**. Não foi executada instalação em máquina limpa nem teste presencial de duas máquinas; essas evidências continuam necessárias antes de release. Após a inclusão da prévia JPEG, o teste nativo de codificação RGBA elevou a suíte Tauri para **5 testes**; o teste de autorização da API cobre agora os dois formatos e a rejeição de tipo MIME incorreto.

## Prévia JPEG mais frequente — 17/10/2026

O host usa o gravador contínuo de Windows.Graphics.Capture oferecido pelo XCap, codifica os quadros recebidos em JPEG e tenta publicar no máximo um a cada ~67 ms. Codificação e envio ficam em um único trabalhador com contrapressão; parar a sessão, perder a autorização ou trocar de monitor encerra o gravador. Minimizar a janela de atendimento mantém a captura autorizada. O envio manual continua em PNG. O servidor valida formato, tipo MIME, dimensões, tamanho de até 2 MiB, identidade, consentimento e prazo da sessão antes de guardar somente o último quadro em memória. Revogação e parada apagam o buffer. A taxa efetiva e a latência ainda não foram medidas em duas máquinas; isto **não implementa codec de vídeo, transporte em tempo real, relay NAT nem E2EE**.

A captura avulsa anterior usava GDI/BitBlt porque a funcionalidade `wgc` do XCap estava desativada. Ela agora usa Windows.Graphics.Capture também. O caminho atual exige Windows moderno com suporte à API e precisa de validação presencial de hotplug, DPI, janela minimizada, indicadores visíveis e encerramento de recursos.

## Captura e entrada em três instaladores — 17/10/2026

O mesmo fluxo de consentimento e quadros foi ligado aos backends nativos do XCap em Windows, macOS e Linux. Windows usa Windows.Graphics.Capture; macOS usa AVFoundation/AVCaptureScreenInput e exige permissão de Gravação de Tela; Linux usa X11 ou a combinação ScreenCast portal/PipeWire no Wayland, quando a enumeração de monitores e o portal da sessão funcionam. A entrada via Enigo usa as APIs do Windows/macOS e X11 no Linux. O macOS exige permissão de Acessibilidade, verificada ao ativar o controle; Wayland permanece sem injeção de mouse/teclado até integrar uma sessão autorizada do portal RemoteDesktop. O mouse fica limitado ao monitor principal em todas as plataformas.

O workflow `installers.yml` compila e executa os testes nativos no Windows, macOS e Ubuntu e empacota NSIS/MSI, DMG e DEB/RPM de QA. Os pacotes não são assinados ou notarizados. Nesta máquina Windows passaram a compilação nativa, 5 testes Tauri, 38 testes do workspace e o build da interface; os jobs macOS/Linux e uma sessão real entre sistemas ainda precisam ser executados. A prévia permanece JPEG/HTTP a até 15 FPS e não deve ser chamada de vídeo fluido de baixa latência.

Apenas uma issue pode ser fechada quando **todos** seus critérios de aceitação estiverem evidenciados em PR, CI e testes exigidos. Não marcar o MVP como pronto apenas por conseguir abrir a interface.

## Defesa contra abuso do servidor de desenvolvimento — 17/10/2026

O registro assinado foi limitado a **256 identidades online** simultâneas (TTL de presença existente: 90 segundos), preservando a capacidade de renovar uma identidade já registrada. Convites pendentes são limitados a **8 por operador** e **16 por host**, com contagem após expiração de convites, evitando que um operador preencha a fila do host. O control plane mantém no máximo **512 estados de sessão**; quando a capacidade é alcançada, somente recibos terminados/rejeitados são reciclados e quadros/filas associados são eliminados. Sessões ativas e convites válidos nunca são desalocados por essa limpeza. Excesso retorna `429 rate_limited`.

A janela de prevenção de replay mantém no máximo **8192 nonces** válidos; ao atingir o limite, recusa novos pedidos autenticados com HTTP 429, sem descartar evidências ainda válidas. Isso é mitigação de esgotamento de memória **em uma instância experimental**, não proteção completa contra ataques distribuídos, persistência de estado ou substituto de rate limit por origem/chave/tenant em ambiente de produção. Testes Rust cobrem esgotamento, renovação, limites entre operadores, recuperação de convite expirado e preservação de sessão ativa durante limpeza de recibos.

## Integração de infraestrutura RustDesk OSS — 18/10/2026

- Receita de infraestrutura de `hbbs` e `hbbr` em [deploy/rustdesk-oss](../deploy/rustdesk-oss/README.md), sem substituir silenciosamente o backend anterior.
- Diagnóstico Tauri/Rust `probe_rustdesk_server`: valida hostname, envia `TestNatRequest` binário na porta TCP 21116 e exige `TestNatResponse` com porta válida. Frontend informa explicitamente que a conexão de sessão **não** está disponível nesta nova infraestrutura.
- Testes unitários de formatação binária e rejeição de respostas inválidas/truncadas implementados no módulo; homologação em servidor real e CI ainda pendentes de execução.
- Próximas entregas obrigatórias: identidade do servidor com chave pública, registro `hbbs` dos dispositivos, conexão P2P, fallback `hbbr`, E2EE, autorização granular e testes E2E. **Nenhum fluxo de sessão do Seven Desk é servido por RustDesk OSS ainda.** Ver [RUSTDESK_MIGRACAO.md](RUSTDESK_MIGRACAO.md).
