# Ambiente de desenvolvimento local — Seven Desk (pré-alfa)

Esta configuração **não exige Docker, PostgreSQL ou Redis**. A API Rust/Axum atual funciona apenas em memória e escuta `127.0.0.1` por padrão. Não exponha esse backend em redes públicas. O desktop é executável, mas acesso remoto pela Internet, mídias E2EE e serviço host não estão homologados.

## Dependências e versões

- Windows 11: Node.js **22 LTS**, npm e Rust **1.90.0** (fixado em `rust-toolchain.toml`); MSVC Build Tools e WebView2 para compilar/executar o Tauri.
- Linux de desenvolvimento: Node.js 22, Rust 1.90 e headers WebKit2GTK/DBus descritos em `.github/workflows/ci.yml`.
- macOS de desenvolvimento: Node.js 22, Rust 1.90 e Xcode Command Line Tools.
- **Sem banco externo atualmente**: a API não possui esquema/migrações PostgreSQL de produção. O teste de inicialização configura `DATABASE_URL` deliberadamente inválido para provar que a pré-alfa não depende de um banco em rede.

O arquivo `.env.example` só documenta variáveis locais não secretas. **O processo não carrega arquivos .env automaticamente.** Em PowerShell, por exemplo:

```powershell
$env:SEVEN_BIND = '127.0.0.1:43821'
$env:RUST_LOG = 'seven_server=info'
```

## Bootstrap reproduzível

```powershell
git clone https://github.com/gabriell211/seven-desk.git
cd seven-desk
npm install --no-package-lock --include=optional --no-audit --no-fund
cargo test --workspace
npm run desktop:build
cargo run -p seven-server
```

Em outro terminal:

```powershell
Invoke-RestMethod http://127.0.0.1:43821/healthz
npm run tauri:dev
```

O launcher `npm run tauri:dev` executa desktop nativo; `npm run desktop:dev` executa apenas o preview web e não tem acesso aos comandos nativos. A pasta `apps/desktop` é um workspace npm; não use scripts que presumam pnpm. O lock npm atual possui dependências opcionais nativas, portanto a instalação local/CI usa `--include=optional` e `--no-package-lock` enquanto a consolidação multiplataforma do lockfile permanece pendente.

## Testes de fumaça automatizados, sem Docker

```powershell
npm run smoke:server   # compila e executa backend real no localhost
npm run desktop:build
npm run smoke:desktop  # inicia preview Vite real e verifica bundle + asset
npm run dev:clean      # apaga apenas artefatos gerados no desktop
```

O `smoke:server` lê `tests/fixtures/devices-and-sessions.json` (apenas identificadores fictícios), inicia o executável real em uma porta efêmera de `127.0.0.1` e verifica status, `production_ready:false`, recusa de registro sem assinatura e rejeição de payload inválido. Também confirma que o servidor recusa escuta em `0.0.0.0` e funciona com banco inacessível. O script sempre tenta encerrar o processo filho; não guarda chaves de assinatura, senhas nem endereços de clientes reais.

O `smoke:desktop` verifica o bundle React de produção em um servidor Vite local e os assets de marca. **Não simula uma sessão real nem comprova operação de Tauri com permissões do sistema.** A CI executa os smoke tests em Linux e Windows, e compila o Tauri separadamente. A matriz e os cenários de múltiplas máquinas permanecem em `docs/TESTE_MULTIMAQUINA.md`.

Para uma limpeza profunda e explícita dos artefatos Cargo locais, execute `node scripts/clean-local.mjs --deep`. O script remove somente diretórios de build conhecidos e preserva fontes, locks, `.env` e arquivos de credenciais. Ele não remove aplicativos já instalados.

## Limitações

- Sem `DATABASE_URL` funcional requerido atualmente; **não** existe ainda teste de migração/restauração em banco real. Isso pertence às issues de persistência/operação.
- Sem autorização para acesso não assistido ou execução invisível. A UI precisa estar visível para suporte assistido.
- A infraestrutura de laboratório HTTPS, instalada conforme `deploy/lab/Caddyfile.example`, continua experimental e sem E2EE.
- Smoke de HTTP e build em runners limpos **não substituem** teste de instaladores e sessão em duas máquinas físicas.
