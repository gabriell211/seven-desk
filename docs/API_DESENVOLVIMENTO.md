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
| `GET` | `/v1/sessions/host/active` | Listar somente autorizações ativas do host autenticado para revogar após reabrir o aplicativo |
| `GET` | `/v1/sessions/operator/active` | Retornar somente sessões ativas do operador autenticado, mesmo após reiniciar a janela |
| `POST` | `/v1/sessions/{id}/frame` | Host autorizado publica o último quadro PNG, limitado a 2 MiB |
| `GET` | `/v1/sessions/{id}/frame` | Operador da sessão autorizada lê o quadro recente, com `Cache-Control: no-store` |
| `GET` | `/v1/sessions/{id}` | Consultar estado quando autenticado e participante |
| `POST` | `/v1/sessions/{id}/decision` | Host aprovar ou rejeitar capacidades solicitadas |
| `POST` | `/v1/sessions/{id}/revoke` | Host revogar uma capacidade ativa |
| `DELETE` | `/v1/sessions/{id}` | Participante autorizado encerrar |

## Aplicativo desktop pré-alfa

No aplicativo Tauri instalado, o usuário pode **Ativar** uma identidade segura e escolher **Registrar no servidor local**. O registro é assinado no processo Rust, não no React. Com a janela aberta e a presença ativada, o frontend solicita um heartbeat nativo a cada 30 segundos. Ao falhar, o estado muda para inativo e exige novo registro manual.

O comando nativo usa `http://127.0.0.1:43821` por padrão ou a URL definida em `SEVEN_DESK_DEV_SERVER_URL`, validada no Rust: **HTTPS obrigatório fora do loopback**, sem credenciais nem caminhos extras, timeout de 5 segundos e nova assinatura por requisição. A chave privada não sai do Rust. Existe captura manual estática no Windows; streaming contínuo e controle de teclado/mouse permanecem pendentes.

## Presença (experimental)

Após registrar a identidade, ela fica marcada online por **90 segundos**. Um heartbeat com assinatura válida atualiza a validade. Integrações futuras deverão enviar heartbeats em cadência inferior ao TTL (por exemplo, a cada 30 segundos). Se expirar, o host é tratado offline e não pode receber novas solicitações até se registrar novamente com prova assinada. Um heartbeat atrasado é rejeitado, sem renovação implícita.

A consulta de presença requer um operador cadastrado e devolve `{"device_id":"<uuid>","online":true|false}`. IDs desconhecidos e hosts offline são indistinguíveis no valor `online:false`. Registro **não habilita acesso não assistido** e não fornece controle ou captura.

O armazenamento atual é em memória: presença se perde ao reiniciar o servidor. Não constitui implementação de parque de dispositivos nem descoberta automática de máquinas na LAN. A presença não é sinal de consentimento.

## Negociação de sessão no aplicativo desktop (pré-alfa)

Depois de ativar a identidade e registrar a presença no servidor de desenvolvimento, a interface permite:

1. Inserir o ID UUID de outro dispositivo registrado **no mesmo servidor local** e solicitar autorização para **visualização futura**. O pedido é assinado no Rust nativo e expira em 120 segundos.
2. O host autenticado consulta automaticamente a própria fila de convites a cada quatro segundos e pode **recusar** ou **autorizar a negociação**. Não existe aceite automático.
3. O solicitante consulta o estado a cada três segundos; pode cancelar. O host pode ver as autorizações ativas com uma consulta autenticada, inclusive após reabrir o aplicativo (enquanto o servidor mantiver as sessões), e encerrar qualquer uma delas. Ambas as ações são assinadas e verificadas no backend.
4. A interface indica explicitamente que uma negociação autorizada **não significa que os pixels estejam sendo transmitidos**. Nenhuma API de captura, injeção de teclado/mouse, porta WAN ou acesso não assistido é ativada.

**Limite importante:** a API local continua escutando em `127.0.0.1`. Dois computadores podem apontar para o mesmo servidor através de **HTTPS, proxy TLS e VPN de laboratório**, conforme [teste multimaquina](TESTE_MULTIMAQUINA.md). Ainda não existe E2EE, streaming ou infraestrutura pronta para Internet aberta. Não exponha a porta 43821 diretamente à rede.

## Canal experimental de quadros de tela (loopback no servidor e HTTPS/VPN no cliente)

O backend agora dispõe de um **buffer temporário real de bytes PNG**, não um vídeo simulado: host autenticado publica e operador autenticado consulta o último quadro de uma sessão explicitamente aprovada com capacidade `screen`. A API verifica **papel host/operador e autorização vigente a cada solicitação**, inclusive depois de revogação; nenhum dispositivo que não participa da sessão pode baixar os pixels. A resposta é `image/png` com `Cache-Control: no-store`.

**Limites da prova de conceito:** cada quadro tem no máximo 2 MiB, o cache só mantém até 8 sessões simultâneas e só disponibiliza quadros recebidos nos últimos 15 segundos. Os bytes ficam **apenas em memória**. O servidor verifica assinatura, cabeçalho IHDR e dimensões declaradas (máximo 8192 × 8192), mas a decodificação da imagem ainda deve ser tratada como entrada não confiável. A revogação da capacidade `screen` ou encerramento da sessão bloqueia novas leituras **e apaga imediatamente o quadro armazenado em memória**. As operações de upload/leitura/encerramento compartilham uma ordem consistente de bloqueio (`sessões` → `quadros`) para impedir que um upload em andamento restaure pixels depois do encerramento. A expiração da sessão observada em uma consulta de estado também descarta o quadro.

**Segurança:** o backend continua escutando apenas em `127.0.0.1`. O cliente pode alcançá-lo através de proxy HTTPS restrito à VPN. O servidor intermediário **consegue ler os quadros**; ainda não existe E2EE, streaming contínuo, relay de mídia escalável nem implantação de produção. O frame só é enviado após clique do host e verificação da sessão.

## Captura manual nativa no Windows (pré-alfa)

O aplicativo Tauri no **Windows** passa a usar a biblioteca Rust [XCap](https://github.com/nashaofu/xcap) (licença Apache-2.0) para capturar um quadro real do **monitor principal** somente quando o **usuário no host clica** em `Compartilhar um quadro`, depois de uma sessão ter sido aprovada. A solicitação HTTP e os pixels são tratados no Rust; o React não acessa chaves privadas.

O programa reduz progressivamente a resolução de capturas grandes antes de codificar PNG, com limite de 2 MiB por quadro. O operador pode clicar em `Ver último quadro autorizado` para baixar a imagem estática da própria sessão através de uma requisição assinada. Nenhuma captura fica salva em disco pelo fluxo implementado. Encerrar a autorização impede novas leituras e remove a imagem da visualização daquele solicitante.

**Limitações:** somente Windows captura nesta primeira etapa; não existe captura automática, streaming contínuo, mouse, teclado ou áudio remoto. O teste **em dois computadores** passa a ser possível mediante backend local + proxy HTTPS restrito à VPN, mas precisa de montagem, execução e evidências reais. O CI verifica compilação em Windows/Linux, não o funcionamento de captura em todos os monitores; E2E e segurança ainda são obrigatórios.

## Atualização voluntária de tela (~1 quadro por segundo)

O host Windows agora dispõe de `Iniciar quadros (~1 FPS)` e `Parar quadros`. Essa função **não inicia sozinha**: ela só fica disponível após aceite explícito do host para a capacidade `screen`, e permanece claramente indicada no topo do aplicativo. A interface envia no máximo um quadro por segundo, pulando ciclos se uma captura ainda estiver em andamento. Minimizar/ocultar a janela ou perder a autorização/conectividade interrompe novos envios. O botão **Encerrar autorização de tela** invalida a sessão no servidor.

O operador autenticado pode iniciar `Atualizar automaticamente (~1 FPS)` ou parar a atualização. Falta de quadro novo limpa a imagem exibida; falha de autenticação ou conexão interrompe o modo de visualização. Cada solicitação consulta novamente as permissões no backend.

Isto é uma **sequência de PNGs estáticos**, não uma sessão de vídeo codificada (H.264/AV1), não usa bitrate adaptativo nem oferece latência de controle profissional. O fluxo de teste em dois computadores exige servidor intermediário HTTPS em VPN, ainda **sem E2EE**. Não há execução independente da janela Tauri, serviço privilegiado, gravação em disco, controle de mouse/teclado ou instalação de host permanente.

## Convites de suporte

- Pedido exige identidades registradas, nonce, assinatura e expiração curta.
- Apenas o host destinatário consegue consultar o convite pelo inbox e aprovar as capacidades solicitadas.
- Ausência de aceite impede transmissão e controle; as permissões revogadas tornam-se indisponíveis.
- Sessões e convites continuam em memória, sem relay de vídeo/entrada e sem suporte cross-host operacional.
- Não copiar IDs entre organizações para contornar políticas futuras.

## Testes e limitações

Executar no código-fonte: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. A CI deve exercitar autenticação, host offline, heartbeat, rejeição, expiração e isolamento da fila. **Nenhum desses testes substitui homologação E2E de acesso remoto em dois computadores**.
