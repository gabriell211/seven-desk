# Seven Desk 0.1.0 — pré-lançamento para testes

Instaladores nativos para Windows (EXE e MSI), macOS (DMG para Apple Silicon e Intel) e Linux (DEB e RPM). O instalador EXE do Windows traz a identidade visual do Seven Desk e barra de progresso com a cor da marca.

## Antes de instalar

- Esta versão é experimental. Os pacotes ainda não têm assinatura de código nem notarização Apple. Verifique os hashes publicados em `SHA256SUMS.txt`.
- A conexão entre redes requer um servidor Seven Desk acessível pelos dois computadores. O pacote não fornece infraestrutura pública automaticamente; uma URL da Vercel não substitui um relay persistente.
- O Windows usa WebView2. macOS requer permissão de Gravação de Tela e Acessibilidade. No Linux, a captura Wayland depende do portal da sessão, e o controle de entrada funciona apenas em X11.
- O relay atual é HTTP sobre TLS do servidor, sem criptografia ponta a ponta auditada. Não use para dados sensíveis ou suporte em produção.

## Estado desta versão

Há pedido de conexão com aceite em janela separada, autorização de tela/mouse/teclado, captura nativa experimental, prévia JPEG de até 15 FPS, chat de sessão e configuração básica de proxy. Permanecem pendentes vídeo de baixa latência, serviço host antes do login, transferência de arquivos, contas e MFA, proxy NTLM/PAC e homologação entre sistemas. O pedido de teste no Windows continua importante antes de uma versão estável.
