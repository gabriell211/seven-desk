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

**Estado:** há um caminho completo de código para eventos de mouse entre dois clientes autenticados via servidor HTTPS de laboratório, mas **não existe homologação prática em dois equipamentos Windows** e o backend ainda não implementa E2EE, buffer eficiente de vídeo nem serviço host permanente. O protocolo de teclado/atalhos, monitores secundários, arraste, proteção contra desktop elevado, benchmarks e suporte multiplataforma ainda são pendências. O mouse não habilita acesso não assistido.

## O que ainda bloqueia uso real

1. Implementar serviço host real e permanente, rotação/revogação de identidades e operação multi-dispositivo. A UI já consegue registrar sua presença no servidor local por requisição assinada, mas **isso não habilita controle remoto**.
2. Authn/authz final, persistência PostgreSQL, limpeza de sessões e proteção contra DoS.
3. Comunicação de controle protegida ponta a ponta (P2P/relay), codecs com baixa latência e **teclado/mouse** com APIs nativas. A captura Windows e envio de PNG são experimentais; não equivalem ao motor de vídeo pronto.
4. Serviço do host/IPC e atualização assinada do aplicativo.
5. E2E em duas máquinas reais e testes de segurança cross-platform.
6. Escolha explícita da licença do Seven Desk; o código provisório usa `UNLICENSED`.

Apenas uma issue pode ser fechada quando **todos** seus critérios de aceitação estiverem evidenciados em PR, CI e testes exigidos. Não marcar o MVP como pronto apenas por conseguir abrir a interface.
