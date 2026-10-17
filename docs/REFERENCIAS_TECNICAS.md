# Pesquisa técnica — acesso remoto e instaladores

> Verificação em 16/10/2026. Este documento registra **referências públicas de engenharia**. Nenhum código de terceiros foi incorporado ao Seven Desk. Os recursos continuam planejados.

## Repositórios analisados

| Referência | URL | O que estudar | Cuidados |
| --- | --- | --- | --- |
| RustDesk — cliente desktop | https://github.com/rustdesk/rustdesk | Sessão remota, captura, transporte, serviço do host, empacotamento e install/upgrade | Repositório licenciado sob **AGPL-3.0**; não reutilizar código sem estratégia de conformidade |
| RustDesk — servidor OSS | https://github.com/rustdesk/rustdesk-server | Sinalização, rendezvous e relay self-hosted | Avaliar licença do componente e escopo de protocolos antes de integrar |
| Tauri v2 | https://github.com/tauri-apps/tauri | Bundler oficial para UI React/TypeScript + Rust | Integração de serviço privilegiado precisa ser projetada e testada à parte |
| Tauri GitHub Action | https://github.com/tauri-apps/tauri-action | Builds e publicação de instaladores multiplataforma no GitHub Releases | Fixar versões de actions e toolchain, usar secrets protegidos para signing |

**Orientação arquitetural:** seguir com implementação própria em Rust + Tauri + React e utilizar o bundler oficial do Tauri para **instaladores Seven Desk**. A pesquisa em código aberto é para identificar padrões, limitações e testes. Eventual reutilização de código precisa de análise de licença e segurança.

## Estratégia oficial de distribuição

- **Windows:** Tauri v2 com target **NSIS** (`.exe`), produto identificado como Seven Desk; MSI via WiX pode ser opção adicional para implantação corporativa. O assistente instala o app, cria entradas de sistema e desinstalador; nenhuma permissão remota persistente é concedida por instalar o programa.
- **Linux:** pacotes `.deb`/`.rpm`, gerenciados pela distribuição suportada. A instalação de serviço systemd, quando necessária, não implica autorização de acesso não assistido.
- **macOS:** formato **`.dmg`** como rota nativa usual do bundler Tauri para distribuição direta, com `.app` assinado/notarizado; **`.pkg`** como alternativa de instalador com assistente, mediante etapa adicional de empacotamento/assinatura via ferramentas Apple, se os requisitos de instalação de componentes e políticas exigirem.
- **Release:** build nos sistemas-alvo, checksums publicados, assinatura conforme requisitos do SO, artefatos anexados ao GitHub Release e testes de instalação/desinstalação em máquina limpa.
- **Nomes de artefatos:** `SevenDesk-Setup-<versão>.exe`, `SevenDesk-<versão>.msi` (opcional), `SevenDesk-<versão>.deb`, `SevenDesk-<versão>.rpm`, `SevenDesk-<versão>.dmg`, `SevenDesk-<versão>.pkg` (opcional).

## Documentação oficial de empacotamento

- Windows NSIS/MSI: https://v2.tauri.app/distribute/windows-installer/
- macOS DMG: https://v2.tauri.app/distribute/dmg/
- Distribuição Tauri: https://v2.tauri.app/distribute/
- Pipelines GitHub: https://v2.tauri.app/distribute/pipelines/github/
- Exemplo reutilizável de action: https://github.com/tauri-apps/tauri-action

## Provas obrigatórias antes de encerrar issues

1. Inicializar aplicação real no desktop em modo instalado.
2. Abrir o instalador Seven Desk em Windows de teste, concluir e desinstalar sem toolchain.
3. Testar upgrade, cancelamento, serviço opcional e recuperação de erro, sem ativar controle remoto implícito.
4. Produzir Linux/macOS apenas nas plataformas efetivamente declaradas como suportadas; observar assinaturas e aprovações do SO.
5. Validar uma release com hashes, metadados, changelog e resultado reproduzível.

**Não confundir o fato de existir um instalador funcional com o acesso remoto já estar homologado.** O empacotamento de um shell Tauri não significa que os componentes de captura, transporte, autenticação e entrada estejam prontos.

Referências associadas: [#15](https://github.com/gabriell211/seven-desk/issues/15), [#23](https://github.com/gabriell211/seven-desk/issues/23), [#38](https://github.com/gabriell211/seven-desk/issues/38).
