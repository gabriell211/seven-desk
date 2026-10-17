# Seven Desk — Primeira entrega executável

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

O serviço backend inicial expõe `GET /healthz` e rotas protegidas por assinatura para registro/convites. Não iniciar controle de dispositivo sem handshake, autorização do host e implantação segura.

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

`POST /v1/devices` aceita exatamente `{}` assinado. A sessão requer ambos os dispositivos cadastrados e operador autenticado. A decisão é exclusivamente do host. Após o aceite, a sessão tem prazo próprio (até 8 horas inicialmente) separado da expiração do convite. O host pode revogar capacidades por `POST /v1/sessions/{id}/revoke` com assinatura. O canal de mídia **não existe nesta entrega**, e o backend informa `production_ready: false`.

## O que ainda bloqueia uso real

1. Integração de cadastro de identidade assinada no serviço host, revogação/rotação de chaves e operação multi-dispositivo. A identidade local já usa armazenamento seguro, mas ainda não concede acesso remoto.
2. Authn/authz final, persistência PostgreSQL, limpeza de sessões e proteção contra DoS.
3. Comunicação de controle protegida ponta a ponta (P2P/relay), captura/codec e teclado/mouse com API nativa.
4. Serviço do host/IPC e atualização assinada do aplicativo.
5. E2E em duas máquinas reais e testes de segurança cross-platform.
6. Escolha explícita da licença do Seven Desk; o código provisório usa `UNLICENSED`.

Apenas uma issue pode ser fechada quando **todos** seus critérios de aceitação estiverem evidenciados em PR, CI e testes exigidos. Não marcar o MVP como pronto apenas por conseguir abrir a interface.
