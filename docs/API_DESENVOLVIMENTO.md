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

## Aplicativo desktop pré-alfa

No aplicativo Tauri instalado, o usuário pode **Ativar** uma identidade segura e escolher **Registrar no servidor local**. O registro é assinado no processo Rust, não no React. Com a janela aberta e a presença ativada, o frontend solicita um heartbeat nativo a cada 30 segundos. Ao falhar, o estado muda para inativo e exige novo registro manual.

O comando nativo usa exclusivamente o endpoint de desenvolvimento `http://127.0.0.1:43821`, com timeout de 5 segundos e assinatura nova por requisição. Não há configuração para expor credenciais a JavaScript nem para conectar a servidores arbitrários. O aplicativo ainda **não inicia sessão remota**, pois captura e serviço do host permanecem pendentes.

## Presença (experimental)

Após registrar a identidade, ela fica marcada online por **90 segundos**. Um heartbeat com assinatura válida atualiza a validade. Integrações futuras deverão enviar heartbeats em cadência inferior ao TTL (por exemplo, a cada 30 segundos). Se expirar, o host é tratado offline e não pode receber novas solicitações até se registrar novamente com prova assinada. Um heartbeat atrasado é rejeitado, sem renovação implícita.

A consulta de presença requer um operador cadastrado e devolve `{"device_id":"<uuid>","online":true|false}`. IDs desconhecidos e hosts offline são indistinguíveis no valor `online:false`. Registro **não habilita acesso não assistido** e não fornece controle ou captura.

O armazenamento atual é em memória: presença se perde ao reiniciar o servidor. Não constitui implementação de parque de dispositivos nem descoberta automática de máquinas na LAN. A presença não é sinal de consentimento.

## Negociação de sessão no aplicativo desktop (pré-alfa)

Depois de ativar a identidade e registrar a presença no servidor de desenvolvimento, a interface permite:

1. Inserir o ID UUID de outro dispositivo registrado **no mesmo servidor local** e solicitar autorização para **visualização futura**. O pedido é assinado no Rust nativo e expira em 120 segundos.
2. O host autenticado consulta automaticamente a própria fila de convites a cada quatro segundos e pode **recusar** ou **autorizar a negociação**. Não existe aceite automático.
3. O solicitante consulta o estado a cada três segundos; pode cancelar. O host pode encerrar a autorização. Ambas as ações são assinadas e auditáveis no código do protocolo.
4. A interface indica explicitamente que uma negociação autorizada **não significa que os pixels estejam sendo transmitidos**. Nenhuma API de captura, injeção de teclado/mouse, porta WAN ou acesso não assistido é ativada.

**Limite importante:** esta etapa só negocia metadados de sessão. O servidor bindado em `127.0.0.1` não conecta dois computadores diferentes via Internet. Para usar o recurso em duas máquinas reais faltam control plane seguro acessível de ambas, transporte autenticado e serviço do host. Não exponha o servidor experimental na rede para contornar essa limitação.

## Convites de suporte

- Pedido exige identidades registradas, nonce, assinatura e expiração curta.
- Apenas o host destinatário consegue consultar o convite pelo inbox e aprovar as capacidades solicitadas.
- Ausência de aceite impede transmissão e controle; as permissões revogadas tornam-se indisponíveis.
- Sessões e convites continuam em memória, sem relay de vídeo/entrada e sem suporte cross-host operacional.
- Não copiar IDs entre organizações para contornar políticas futuras.

## Testes e limitações

Executar no código-fonte: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. A CI deve exercitar autenticação, host offline, heartbeat, rejeição, expiração e isolamento da fila. **Nenhum desses testes substitui homologação E2E de acesso remoto em dois computadores**.
