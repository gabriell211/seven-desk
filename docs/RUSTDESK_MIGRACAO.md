# Seven Desk — migração gradual para servidores RustDesk OSS

## Decisão

Usar **RustDesk Server OSS** como infraestrutura futura de rendezvous e relay (`hbbs` / `hbbr`) em instância própria; manter a identidade visual e a aplicação Seven Desk. O protocolo OSS é **Protobuf binário sobre TCP/UDP**, não HTTP/JSON e nem o serviço de API da edição Pro. Esta decisão **não** equivale a integração completa.

A implantação em [deploy/rustdesk-oss](../deploy/rustdesk-oss/README.md) cria somente os serviços externos. O diagnóstico `probe_rustdesk_server` envia `RendezvousMessage.test_nat_request` ao `hbbs` via TCP/21116, com framing RustDesk, e exige `test_nat_response` válido. O teste não registra um dispositivo nem autentica o servidor; **não deve ser usado como sinal de canal remoto seguro**.

## Contratos que NÃO são intercambiáveis

| Atual no Seven Desk (laboratório) | No RustDesk Server OSS |
| --- | --- |
| `POST /v1/devices` com Ed25519/HTTP | Registro `RegisterPeer` / `RegisterPk` via UDP, chaves e estado específicos |
| `GET /v1/sessions/inbox` + `decision` | Rendezvous e mensagens peer-to-peer do cliente RustDesk |
| `/v1/sessions/{id}/frame` (JPEG/PNG) | Transporte de mídia peer-to-peer cifrado ou stream pelo `hbbr` |
| `/v1/sessions/{id}/input` | Mensagens de sessão e autorização negociadas entre pares |
| HTTPS/Caddy restrito à VPN | TCP/UDP 21115-21117 + chaves de servidor; WebSocket opcional |

## Gates da substituição definitiva

1. **Infraestrutura e verificação de protocolo — parcialmente implementado.** `hbbs`/`hbbr` em Compose (revisão operacional pendente); probe de TestNatRequest em TCP 21116 com testes de framing, valores inválidos, respostas truncadas e timeout.
2. **Registro e presença do host.** Cliente Rust com `RegisterPeer/RegisterPk`, autenticação do servidor via chave pública configurada, persistência local segura da identidade e reconexão. Verificar em pelo menos duas máquinas/identidades distintas.
3. **Conexão P2P.** TCP/UDP hole punching real, limites/expiração de tentativas, negociação de sessão, compatibilidade do codec. Nenhum bypass de consentimento; não habilitar conexões não assistidas automaticamente.
4. **Relay de fallback.** `hbbr` quando o P2P falhar, atestação de identidade dos pares, E2EE auditável, revogação imediata, timeouts, limites de banda e defesa contra replay.
5. **Adaptador dos recursos do Seven Desk.** Tela, mouse, teclado, clipboard, chat e arquivos preservam permissões granulares e indicadores de sessão. Não interpretar o ID do servidor como prova de identidade do usuário.
6. **Migração da UI/instalador.** Configuração de ID server, relay e chave pública, migração de configurações antigas, atualização do instalador, mensagens de erro claras, rollback. Só remover comandos/rotas HTTP após E2E LAN/WAN, VPN, CGNAT e autorização negada comprovados.
7. **Produção.** Testes reais de Windows/Linux/macOS, monitoramento, métricas, vulnerabilidades, licenças/NOTICE, releases assinados e operação documentada.

## Segurança

Nunca faça fallback silencioso de E2EE para quadros HTTP legíveis pelo servidor, nunca valide identidade só pelo ID público, nunca associe a prova de disponibilidade `TestNatRequest` a autorização. A infraestrutura livre **não** oferece a API Pro (`21114`) para login/console; não prometer que uma API REST foi substituída se os clientes ainda usam `/v1/*`.

## Evidência de compatibilidade

Verificar:
- Servidor `hbbs` real (ou fixture TCP de teste), request framed de 4 bytes (`0c a2 01 00`), resposta Protobuf com `test_nat_response` e porta válida.
- Hostname inválido, timeout, frame grande, frame curto, protobuf malformado, resposta de tipo errado e indicação falsa de saúde.
- Fluxo de implantação com Compose em Linux ARM64/amd64, persistência da identidade e conectividade TCP/UDP.
- Testes E2E com sessão assistida antes de ativar/ substituir o backend antigo.

Referências: [rendezvous.proto](https://github.com/rustdesk/hbb_common/blob/main/protos/rendezvous.proto), [BytesCodec](https://github.com/rustdesk/hbb_common/blob/main/src/bytes_codec.rs), [Servidor OSS](https://github.com/rustdesk/rustdesk-server).
