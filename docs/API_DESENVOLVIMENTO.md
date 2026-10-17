# Seven Desk — API experimental de sinalização

> **Somente desenvolvimento.** A API está restrita a `127.0.0.1` por padrão, usa armazenamento temporário em memória e **não oferece streaming, controle remoto ou garantia de produção**. Não exponha a porta em redes externas; TLS, tenancy, quotas, proteção contra abuso, revogação de identidade e persistência ainda precisam ser implementados.

## Identidade e autenticação

Cada instalação utiliza um par Ed25519 no cofre seguro do SO. O `device_id` é derivado da chave pública e **não serve como senha**. Os endpoints de device/sessão exigem uma prova assinada em headers:

| Header | Conteúdo |
| --- | --- |
| `x-seven-key` | Chave pública Ed25519 (hex) |
| `x-seven-signature` | Assinatura Ed25519 da mensagem canônica (hex) |
| `x-seven-nonce` | UUID de uso único |
| `x-seven-timestamp` | Epoch em milissegundos |

A mensagem canônica assinada é formada pelas linhas `seven-desk/request/v1`, método HTTP em maiúsculas, **path exato**, timestamp, nonce e SHA-256 hexadecimal do body bruto, separadas por `\n`. O servidor rejeita nonce repetido, assinatura inválida, payload acima do limite e timestamp fora da janela configurada.

**Importante:** assinar no código Rust nativo; nunca transferir a chave privada à UI React.

## Endpoints (pre-alpha)

| Método | Caminho | Propósito e autorização |
| --- | --- | --- |
| `GET` | `/healthz` | Status técnico; `production_ready: false` |
| `POST` | `/v1/devices` | Registrar/renovar presença com prova assinada e body exatamente `{}` |
| `POST` | `/v1/devices/heartbeat` | Renovar presença do dispositivo já registrado e ativo, com prova assinada e body `{}` |
| `GET` | `/v1/devices/{id}/presence` | Consultar se dispositivo está online; requer identidade registrada e ativa |
| `POST` | `/v1/sessions` | Solicitar sessão somente com operador e host ativos, assinatura e escopo tipado |
| `GET` | `/v1/sessions/inbox` | Consultar convites pendentes somente do host autenticado |
| `GET` | `/v1/sessions/{id}` | Consultar estado quando autenticado e participante |
| `POST` | `/v1/sessions/{id}/decision` | Host aprovar ou rejeitar capacidades solicitadas |
| `POST` | `/v1/sessions/{id}/revoke` | Host revogar uma capacidade ativa |
| `DELETE` | `/v1/sessions/{id}` | Participante autorizado encerrar |

## Presença (experimental)

Após registrar a identidade, ela fica marcada online por **90 segundos**. Um heartbeat com assinatura válida atualiza a validade. Integrações futuras deverão enviar heartbeats em cadência inferior ao TTL (por exemplo, a cada 30 segundos). Se expirar, o host é tratado offline e não pode receber novas solicitações até se registrar novamente com prova assinada. Um heartbeat atrasado é rejeitado, sem renovação implícita.

A consulta de presença requer um operador cadastrado e devolve `{"device_id":"<uuid>","online":true|false}`. IDs desconhecidos e hosts offline são indistinguíveis no valor `online:false`. Registro **não habilita acesso não assistido** e não fornece controle ou captura.

O armazenamento atual é em memória: presença se perde ao reiniciar o servidor. Não constitui implementação de parque de dispositivos nem descoberta automática de máquinas na LAN. A presença não é sinal de consentimento.

## Convites de suporte

- Pedido exige identidades registradas, nonce, assinatura e expiração curta.
- Apenas o host destinatário consegue consultar o convite pelo inbox e aprovar as capacidades solicitadas.
- Ausência de aceite impede transmissão e controle; as permissões revogadas tornam-se indisponíveis.
- Sessões e convites continuam em memória, sem relay de vídeo/entrada e sem suporte cross-host operacional.
- Não copiar IDs entre organizações para contornar políticas futuras.

## Testes e limitações

Executar no código-fonte: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. A CI deve exercitar autenticação, host offline, heartbeat, rejeição, expiração e isolamento da fila. **Nenhum desses testes substitui homologação E2E de acesso remoto em dois computadores**.
