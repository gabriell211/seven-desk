# Chat textual autorizado — pré-alfa

O Seven Desk implementa um chat de texto **temporário e explícito** no plano de sinalização Rust, acessível pelos clientes desktop nativos.

## Comportamento

- O operador solicita a capacidade `chat` no convite; o host seleciona separadamente se quer concedê-la.
- Apenas as identidades Ed25519 dos dois participantes autenticados da sessão ativa podem enviar ou ler mensagens.
- O servidor verifica a permissão em **toda** leitura e escrita; convite pendente, rejeição, ausência de concessão, revogação ou encerramento bloqueiam o acesso.
- `POST /v1/sessions/{id}/chat` envia `{"text":"..."}` assinado e retorna o número sequencial, dispositivo remetente e instante. `GET` no mesmo caminho retorna as mensagens mais recentes.
- Texto limitado a 500 caracteres/2.048 bytes; 400 ms mínimos entre envios por remetente; 24 últimas mensagens por sessão; até 32 sessões com histórico simultaneamente.
- O histórico reside apenas em **RAM** e é descartado no encerramento, na revogação, na remoção da sessão e no reinício do servidor. Não há gravação nem exportação automática.
- A interface usa polling periódico, sem envio invisível, e exibe os remetentes com segurança como texto (sem interpolar HTML).
- Pausar o vídeo não altera a capacidade de chat que o host concedeu; encerrar a sessão interrompe todos os canais.

## Verificação

`npm run smoke:session` testa o backend real por HTTP no loopback em Windows e Linux via CI: negativa antes do consentimento, terceiro não autorizado, concessão, troca de mensagens, limite/rate-limit, ausência de capacidade, revogação e término.

## Limites para o lançamento

Este é **apenas um protótipo local**: não implementa E2EE de mensagens, persistência, sincronização offline, confirmação de leitura nem conectividade WAN. O servidor não é autorizado para produção e `/healthz` segue com `production_ready:false`. Faltam validação com dois equipamentos e análise de segurança. Não considerar a issue de chat concluída para release antes disso.
