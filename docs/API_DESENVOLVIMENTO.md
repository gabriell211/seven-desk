# Seven Desk — API experimental de sinalização

> **Somente desenvolvimento.** A API está restrita a `127.0.0.1` por padrão e usa armazenamento temporário em memória. Ela oferece quadro manual PNG e prévia periódica JPEG até 15 FPS, além de filas experimentais de mouse/teclado, mas **não oferece vídeo profissional, E2EE ou garantia de produção**. Não exponha a porta em redes externas; TLS, tenancy, quotas, proteção contra abuso, revogação de identidade e persistência ainda precisam ser implementados.

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
| `POST` | `/v1/sessions/{id}/screen-delivery` | Somente o host ativa ou pausa o canal com `{"enabled":true|false}`; ao pausar, apaga o quadro e impede novas publicações até retomar |
| `POST` | `/v1/sessions/{id}/input` | Operador autorizado envia evento de mouse normalizado e assinado, somente se host concedeu `screen` **e** `input` |
| `GET` | `/v1/sessions/{id}/input` | Host autenticado consome lote temporário de eventos, sem executar OS input no servidor |
| `POST` | `/v1/sessions/{id}/keyboard` | Operador autorizado envia caractere ou tecla especial sem modificadores, com concessão independente `keyboard` |
| `GET` | `/v1/sessions/{id}/keyboard` | Host autenticado consome o lote temporário de teclado; injeção nativa depende de ativação local no host Windows |
| `POST` | `/v1/sessions/{id}/chat` | Participante autorizado envia mensagem de até 500 caracteres e 2048 bytes somente quando o host concedeu `chat` |
| `GET` | `/v1/sessions/{id}/chat` | Participante autorizado consulta o histórico temporário recente |
| `POST` | `/v1/sessions/{id}/frame` | Host autorizado publica o último quadro PNG ou JPEG, limitado a 2 MiB e com `Content-Type` correspondente |
| `GET` | `/v1/sessions/{id}/frame` | Operador da sessão autorizada lê o quadro recente, com `Cache-Control: no-store` |
| `GET` | `/v1/sessions/{id}` | Consultar estado quando autenticado e participante |
| `POST` | `/v1/sessions/{id}/decision` | Host aprovar ou rejeitar capacidades solicitadas |
| `POST` | `/v1/sessions/{id}/revoke` | Host revogar uma capacidade ativa |
| `DELETE` | `/v1/sessions/{id}` | Participante autorizado encerrar |

## Aplicativo desktop pré-alfa

No aplicativo Tauri instalado, a identidade segura é criada automaticamente na primeira abertura e o dispositivo se registra quando o servidor configurado está acessível. O registro é assinado no processo Rust, não no React. Com a janela aberta, o frontend solicita um heartbeat nativo a cada 30 segundos e tenta reconectar quando a presença falha. Registro não concede acesso remoto.

O comando nativo usa `http://127.0.0.1:43821` por padrão, aceita uma URL salva em **Configurações → Endereço do servidor** e prioriza `SEVEN_DESK_DEV_SERVER_URL` quando definida. A URL é validada no Rust: **HTTPS obrigatório fora do loopback**, sem credenciais nem caminhos extras, timeout de 5 segundos e nova assinatura por requisição. A chave privada não sai do Rust. A troca de servidor interrompe os controles locais e tenta revogar sessões ativas no servidor anterior. Existem captura de quadros e controle experimental de mouse e teclado no Windows; vídeo codificado contínuo e homologação E2E permanecem pendentes.

## Presença (experimental)

Após registrar a identidade, ela fica marcada online por **90 segundos**. Um heartbeat com assinatura válida atualiza a validade. Integrações futuras deverão enviar heartbeats em cadência inferior ao TTL (por exemplo, a cada 30 segundos). Se expirar, o host é tratado offline e não pode receber novas solicitações até se registrar novamente com prova assinada. Um heartbeat atrasado é rejeitado, sem renovação implícita.

A consulta de presença requer um operador cadastrado e devolve `{"device_id":"<uuid>","online":true|false}`. IDs desconhecidos e hosts offline são indistinguíveis no valor `online:false`. Registro **não habilita acesso não assistido** e não fornece controle ou captura.

O armazenamento atual é em memória: presença se perde ao reiniciar o servidor. Não constitui implementação de parque de dispositivos nem descoberta automática de máquinas na LAN. A presença não é sinal de consentimento.

## Negociação de sessão no aplicativo desktop (pré-alfa)

Depois que a identidade e a presença são preparadas automaticamente, a interface permite:

1. Inserir o ID UUID de outro dispositivo registrado **no mesmo servidor de laboratório** e solicitar autorização para tela e, opcionalmente, mouse e teclado. O pedido é assinado no Rust nativo e expira em 120 segundos.
2. O host autenticado consulta automaticamente a própria fila de convites a cada quatro segundos. O pedido aparece em uma janela separada com identidade, permissões e botões **Aceitar** e **Rejeitar**. Não existe aceite automático.
3. O solicitante consulta o estado a cada três segundos; pode cancelar. O host pode ver as autorizações ativas com uma consulta autenticada, inclusive após reabrir o aplicativo (enquanto o servidor mantiver as sessões), e encerrar qualquer uma delas. Ambas as ações são assinadas e verificadas no backend.
4. Ao aceitar na janela separada de atendimento, o host inicia a captura e habilita somente mouse/teclado que concedeu. Após confirmar o início da captura, apenas a janela de atendimento minimiza; a janela principal fica aberta. Ao restaurar a janela de atendimento, ela mostra **Finalizar**. O operador entra na visualização em janela maximizada automaticamente. Nenhum acesso não assistido é ativado.

**Limite importante:** a API local continua escutando em `127.0.0.1`. Dois computadores podem apontar para o mesmo servidor através de **HTTPS, proxy TLS e VPN de laboratório**, conforme [teste multimaquina](TESTE_MULTIMAQUINA.md). Ainda não existe E2EE, streaming ou infraestrutura pronta para Internet aberta. Não exponha a porta 43821 diretamente à rede.

## Canal experimental de quadros de tela (loopback no servidor e HTTPS/VPN no cliente)

O backend dispõe de um **buffer temporário de quadros PNG/JPEG**, não de um transporte de vídeo: host autenticado publica e operador autenticado consulta o último quadro de uma sessão explicitamente aprovada com capacidade `screen`. A API verifica **papel host/operador e autorização vigente a cada solicitação**, inclusive depois de revogação; nenhum dispositivo que não participa da sessão pode baixar os pixels. A resposta usa `image/png` ou `image/jpeg` com `Cache-Control: no-store`.

**Limites da prova de conceito:** cada quadro tem no máximo 2 MiB, o cache só mantém até 8 sessões simultâneas e só disponibiliza quadros recebidos nos últimos 15 segundos. Os bytes ficam **apenas em memória**. O servidor verifica assinatura, cabeçalho IHDR e dimensões declaradas (máximo 8192 × 8192), mas a decodificação da imagem ainda deve ser tratada como entrada não confiável. A revogação da capacidade `screen` ou encerramento da sessão bloqueia novas leituras **e apaga imediatamente o quadro armazenado em memória**. As operações de upload/leitura/encerramento compartilham uma ordem consistente de bloqueio (`sessões` → `quadros`) para impedir que um upload em andamento restaure pixels depois do encerramento. A expiração da sessão observada em uma consulta de estado também descarta o quadro.

**Segurança:** o backend continua escutando apenas em `127.0.0.1`. O cliente pode alcançá-lo através de proxy HTTPS restrito à VPN. O servidor intermediário **consegue ler os quadros**; ainda não existe E2EE, streaming contínuo, relay de mídia escalável nem implantação de produção. O frame só é enviado após o aceite do host e verificação da sessão.

## Captura manual nativa no Windows (pré-alfa)

O aplicativo Tauri no **Windows** usa a biblioteca Rust [XCap](https://github.com/nashaofu/xcap) (licença Apache-2.0) para capturar um quadro real do **monitor escolhido pelo host** somente quando o **usuário no host clica** em `Compartilhar um quadro`, depois de uma sessão ter sido aprovada. A solicitação HTTP e os pixels são tratados no Rust; o React não acessa chaves privadas. O monitor principal é o padrão; trocar de monitor interrompe os quadros e o mouse remoto, exigindo nova ação explícita do host para retomar.

O programa reduz progressivamente a resolução de capturas grandes antes de codificar PNG, com limite de 2 MiB por quadro. O operador pode clicar em `Ver último quadro autorizado` para baixar a imagem estática da própria sessão através de uma requisição assinada. Nenhuma captura fica salva em disco pelo fluxo implementado. Encerrar a autorização impede novas leituras e remove a imagem da visualização daquele solicitante.

**Limitações:** somente Windows captura nesta primeira etapa; quadros voluntários e entrada básica existem, mas não há vídeo codificado de baixa latência nem áudio remoto. O teste **em dois computadores** passa a ser possível mediante backend local + proxy HTTPS restrito à VPN, mas precisa de montagem, execução e evidências reais. Compilação local não comprova o funcionamento de captura e entrada em dois equipamentos; E2E e segurança ainda são obrigatórios.

## Prévia voluntária de tela (até 15 quadros por segundo)

O host inicia a prévia ao clicar em **Aceitar** para uma solicitação de `screen`; também pode retomá-la em **Autorizações ativas**. O estado aparece de forma compacta no topo do aplicativo. No Windows, o processo nativo abre uma sessão contínua Windows.Graphics.Capture, recebe quadros e tenta publicar JPEG com intervalo mínimo de cerca de 67 ms; codificação e rede impõem contrapressão, portanto a taxa real depende da máquina e da conexão. Minimizar a janela separada de atendimento após o aceite mantém a captura ativa; trocar monitor ou perder a autorização/conectividade a interrompe. Restaurar essa janela permite clicar em **Finalizar** para encerrar a sessão.

O operador autenticado abre a visualização automaticamente quando o host aceita. Pode voltar ao aplicativo ou encerrar a conexão. Falta de quadro recente limpa a imagem exibida; falha de autenticação ou conexão interrompe o modo de visualização. Cada solicitação consulta novamente as permissões no backend.

Isto é uma **sequência de JPEGs estáticos**, não uma sessão de vídeo codificada (H.264/AV1), não usa bitrate adaptativo nem oferece latência de controle profissional. O fluxo de teste em dois computadores exige servidor intermediário HTTPS em VPN, ainda **sem E2EE**. Não há execução independente do processo Tauri, serviço privilegiado, gravação em disco ou instalação de host permanente. Mouse e teclado básicos dependem do aplicativo em execução e do aceite do host.

## Parada imediata do compartilhamento

O host pode clicar em **Parar quadros** ou **Parar compartilhamento agora**. O desktop interrompe imediatamente a captura local e envia uma solicitação nativa assinada `POST /v1/sessions/{id}/screen-delivery` com `{"enabled":false}`. O servidor verifica **identidade do host**, sessão ativa, consentimento para `screen`, suspende novas leituras/publicações de pixels e elimina o quadro anterior do cache sob a mesma trava da sessão. Um envio que terminar após a parada será negado; um envio concluído antes dela será eliminado.

Retomar os quadros ou compartilhar outro quadro exige uma **nova ação explícita do host**, que solicita `{"enabled":true}`. O operador não pode reativar a entrega nem controlar a captura por API. A autorização original da sessão continua existindo até ser encerrada ou revogada; pausar a imagem não concede outras capacidades.

Se o servidor estiver offline, o aplicativo interrompe imediatamente a captura local, mas não consegue confirmar a limpeza remota: a imagem já enviada expira pelo limite de 15 segundos do cache e a interface deve comunicar essa incerteza. A saída brusca do processo também impede novos quadros, embora não seja garantia de uma solicitação de parada concluída.

## Canal experimental de eventos de mouse (sem injeção automática)

O backend contém uma fila **em memória** de eventos `move`, `click` (botões esquer/direito/meio) e `scroll`. O operador precisa solicitar a capacidade `input` juntamente com `screen`, e somente o host pode concedê-la. A fila **não** executa eventos no sistema operacional: o aplicativo host os lê e aplica somente depois do aceite explícito com mouse marcado e compartilhamento ativo.

Os eventos usam coordenadas relativas de **0 a 10000**, com validação rigorosa. Eventos de scroll têm `steps` entre -5 e 5, exceto zero. Há controle de taxa (mínimo 20ms entre eventos por sessão), fila de até 32 eventos por sessão, até 8 filas simultâneas e validade de 2 segundos. A consulta por host remove os eventos entregues, evitando reexecução do mesmo lote. A leitura e escrita ocorrem sob a mesma trava de autorização que revogação/encerramento. Parar o compartilhamento, revogar `input` ou encerrar uma sessão descarta imediatamente a fila pendente; o operador não pode consultar o lote ou ligar o controle por conta própria.

**Não implementado nesta etapa:** atalhos com modificadores, seleção de monitores, captura contínua de baixa latência e transporte E2EE. A injeção nativa básica de mouse e teclado ocorre somente no host Windows após ativação voluntária. O backend intermediário consegue inspecionar eventos. Este canal é de pesquisa no laboratório, não está homologado para uso na Internet pública.

## Controle interativo de mouse no Windows (pré-alfa)

O cliente desktop solicita **visualização + mouse** por padrão, e o host pode desmarcar mouse na tela de aceite. Se aceitar com mouse marcado, o compartilhamento e o controle iniciam sem um segundo botão de ativação. A UI mostra o estado claramente e oferece **Parar mouse remoto imediatamente** e encerramento da sessão.

O Rust/Tauri do operador assina os eventos normalizados de mouse, e o servidor os enfileira apenas para sessões com as permissões `screen` e `input`. O host lê por requisição assinada, verifica a concessão novamente e aplica os eventos usando a biblioteca nativa Enigo 0.6.1 no **monitor principal**. O controle local inicia desabilitado a cada inicialização do aplicativo e só é habilitado após uma escolha explícita do host na tela de aceite. A desativação limpa o interruptor em memória e a captura deixa de consumir novos eventos; lotes com até 32 eventos e TTL de 2 segundos minimizam a janela de execução tardia. O mouse é pausado ao interromper a imagem, perder rede ou encerrar/revogar a sessão. A minimização após o aceite mantém o receptor nativo ativo.

O operador só controla a imagem após o backend confirmar que sua própria sessão recebeu a capacidade `input`; a interface ativa o mouse automaticamente ao abrir o primeiro quadro autorizado. O mapeamento usa coordenadas normalizadas do conteúdo da imagem e filtra eventos de ponteiro fora dela. Eventos de movimento e rolagem são limitados no frontend a intervalos de ~90 ms. Há suporte inicial a movimento, clique com botão esquerdo/direito/meio e rolagem vertical sobre a imagem remota.

**Limitações obrigatórias:** fase de pesquisa, teclado apenas com caracteres e teclas básicas sem homologação ABNT2, sem drag-and-drop, atalhos, **controle de mouse no monitor secundário**, portal Wayland, UAC/desktop seguro, E2EE ou homologação E2E em duas máquinas físicas. O host pode **visualizar** um monitor secundário, mas o mouse remoto é bloqueado nele porque o mapeamento nativo atual cobre apenas o monitor principal. A aplicação não deve ganhar privilégios elevados automaticamente. O intermediário HTTPS de laboratório ainda consegue ler os eventos de entrada. Testes locais de compilação não comprovam a usabilidade real do controle remoto.

## Teclado remoto básico no Windows (pré-alfa)

O convite pode pedir `keyboard` independentemente de `input` (mouse). O host escolhe explicitamente se concede essa capacidade na tela de aceite; ao aceitar, o teclado acompanha o compartilhamento sem outro botão de ativação. Há um botão visível para parar o teclado imediatamente. Ocultar a janela, pausar os quadros, perder autorização ou encerrar a sessão interrompe a aplicação local de eventos.

O operador só envia teclas enquanto a imagem remota está focada e o controle do teclado está ativado. O protocolo aceita um caractere imprimível por evento ou uma tecla de edição/navegação limitada (`Backspace`, `Delete`, `Enter`, `Tab`, `Escape`, setas). Não transmite teclas modificadoras nem atalhos; cada evento é pressionado e solto no host. O servidor aplica a autorização `keyboard` separada, limite de taxa, fila curta com TTL e limpeza imediata na pausa, revogação ou encerramento.

Esse fluxo ainda não foi homologado em dois computadores ou layouts de teclado diferentes. O servidor de laboratório pode ler as teclas encaminhadas; não use para digitar senhas ou outros dados sensíveis antes de existir E2EE auditado.

## Chat temporário por sessão (pré-alfa)

O operador pode solicitar `chat` no convite. O host precisa marcar a concessão explicitamente antes de aprovar; a permissão pode ser revogada. Com a sessão ativa, ambos enviam e recebem mensagens assinadas. O servidor verifica identidade e concessão em cada requisição, limita cada mensagem a 500 caracteres e 2048 bytes, mantém até 24 mensagens recentes por sessão em até 32 sessões e descarta o histórico ao encerrar ou revogar `chat`. A leitura retorna esse histórico enquanto a sessão estiver ativa.

Este chat é **experimental e não tem E2EE**: o servidor de laboratório pode ler as mensagens. Não use para senhas ou dados sensíveis. Pausar a entrega de tela não revoga `chat`; encerre a sessão ou revogue a permissão para interrompê-lo.

## Convites de suporte

- Pedido exige identidades registradas, nonce, assinatura e expiração curta.
- Apenas o host destinatário consegue consultar o convite pelo inbox e aprovar as capacidades solicitadas.
- Ausência de aceite impede transmissão e controle; as permissões revogadas tornam-se indisponíveis.
- Sessões e convites continuam em memória, sem relay de vídeo/entrada e sem suporte cross-host operacional.
- Não copiar IDs entre organizações para contornar políticas futuras.

## Testes e limitações

Executar no código-fonte: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`. A CI deve exercitar autenticação, host offline, heartbeat, rejeição, expiração e isolamento da fila. **Nenhum desses testes substitui homologação E2E de acesso remoto em dois computadores**.
