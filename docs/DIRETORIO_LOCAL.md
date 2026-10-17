# Diretório local de dispositivos (pré-alfa)

A página **Dispositivos** do Seven Desk permite cadastrar até 50 IDs públicos com apelidos, grupos e favoritos; oferece busca case-insensitive e ordenação por favorito/grupo/nome.

## Segurança e isolamento

- Os contatos são salvos no armazenamento local do webview Tauri, por ID **da identidade do dispositivo local**. Não são enviados a um servidor de diretório nem sincronizados com outras instalações.
- Esse armazenamento **não é um cofre de segredos**: persistimos somente IDs públicos, apelidos, grupos e favorito — jamais senhas, chaves privadas, tokens, permissões ou conteúdo da sessão.
- IDs malformados, duplicados e iguais ao dispositivo local são rejeitados. O carregamento ignora dados locais inválidos e é limitado a 50 registros.
- Contato salvo, favorito, nome e grupo **não são prova de identidade nem autorização para operar o host**. O botão **Usar ID no convite** apenas preenche o alvo na tela Início. O usuário ainda precisa solicitar e obter consentimento.
- **Online** significa que houve `heartbeat` recente segundo o TTL do servidor; não prova disponibilidade do serviço remoto. **Offline** é somente ausência de presença recente, não revogação de identidade. Falha de conectividade/autenticação aparece como **presença indisponível**, não como offline.
- Enquanto a presença local está ativa, a UI faz consultas assinadas `GET /v1/devices/{id}/presence` via Rust nativo, em até 50 contatos, atualizadas aproximadamente a cada 45 segundos. Nunca lista identidades de terceiros por endpoint global.
- Exclusão do contato só remove dados locais; não revoga nem cria permissões no host.

## Evidências e pendências

`npm run smoke:session` valida no servidor Rust real as consultas de presença online/offline e a recusa da consulta anônima. CI compila UI TypeScript e invocação Tauri Windows/Linux; instaladores Windows/Linux/macOS têm QA de geração de pacote.

**Não é diretório corporativo de produção**: faltam histórico real de sessões, tags avançadas, paginação para milhares de dispositivos, sincronização opcional com conta, console multi-tenant, controle de revogação e homologação em dois computadores. Não fechar a issue #24 por este incremento.
