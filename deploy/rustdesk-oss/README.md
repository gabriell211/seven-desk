# Seven Desk | Infraestrutura RustDesk Server OSS (migração em desenvolvimento)

Esta implantação sobe **o servidor livre do RustDesk**, `hbbs` e `hbbr`, em uma máquina **Linux** com Docker Compose. O servidor RustDesk OSS não tem uma API REST pública compatível com a API `/v1/*` do Seven Desk e **não** habilita sessões Seven Desk por si só.

O código do Seven Desk já possui um diagnóstico nativo de Protobuf/TCP para `hbbs` (sem registro de dispositivo). **O mecanismo de autenticação, registro de identidade, NAT traversal, E2EE, streaming e autorização ainda precisa ser integrado ao cliente Seven Desk.** Mantenha o backend HTTPS atual somente para os testes existentes em VPN.

## Preparar e iniciar

1. Disponibilize um Linux x86_64 ou ARM64 com Docker + Compose, IP público e DNS sob seu controle.
2. Execute `cp .env.example .env` nesta pasta; substitua `desk.example.com` pelo seu nome/IP, mantendo `:21117` no valor do relay.
3. Crie o diretório `data` com permissões restritas ao operador. O par de chaves será gerado pelos serviços e **nunca** deve ser commitado.
4. Rode `docker compose up -d` nesta pasta. O `hbbr` espera os arquivos da identidade gerados pelo `hbbs`.
5. Confira `docker compose ps`, `docker compose logs hbbs hbbr` e leia a chave **pública** usando `cat data/id_ed25519.pub`. Guarde a chave privada somente no servidor.
6. No aplicativo Seven Desk, abra Configurações → Servidor RustDesk OSS (hbbs) e teste apenas o **nome** do host (sem protocolo/porta). Um resultado positivo verifica somente o handshake de diagnóstico, sem registrar IDs nem autorizar tela.

## Portas de firewall

| Protocolo | Porta | Serviço |
| --- | --- | --- |
| TCP | 21115 | Teste de tipo NAT |
| TCP e UDP | 21116 | hbbs: ID, registro e rendezvous |
| TCP | 21117 | hbbr: relay |
| TCP | 21118 e 21119 | WebSocket opcional para cliente web; **não habilitado** neste laboratório |

Configure também as regras da cloud/VPS (além do firewall local). Estas portas **não** devem ser substituídas por `https://dominio/healthz`: esse endereço pertence ao control plane antigo, de protocolo distinto. Se precisar operar somente dentro de um laboratório, restrinja as origens à VPN e valide as rotas de NAT.

## Segurança e limitações

- A imagem é fixada na tag `1.1.16` para evitar trocas automáticas de versão; **confira o digest e faça a fixação por SHA-256 antes de produção**.
- `-k _` carrega ou gera a identidade e permite configurar a validação de chave no relay; ambas as instâncias usam a **mesma pasta de dados**. Não exponha os arquivos `id_ed25519`, backups privados ou volumes a terceiros.
- A chave de servidor `id_ed25519.pub` **não** é uma senha de acesso ao host. O aplicativo Seven Desk atual ainda precisa implementar o pinning da chave, o handshake criptográfico correto, as permissões locais e a criptografia ponto a ponto para a arquitetura nova.
- Não use a infraestrutura pública do RustDesk para transportar sessões privadas do Seven Desk sem um cliente protocolarmente compatível e sem avaliar as condições do serviço.
- Esta pasta não modifica nem copia binários de terceiros; utiliza a imagem externa identificada. Código original do Seven Desk: AGPL-3.0-only; o servidor segue a licença e avisos upstream.

Documentação de referência: [RustDesk OSS](https://github.com/rustdesk/rustdesk-server), [portas oficiais](https://rustdesk.com/docs/en/self-host/) e [plano de migração](../../docs/RUSTDESK_MIGRACAO.md).
