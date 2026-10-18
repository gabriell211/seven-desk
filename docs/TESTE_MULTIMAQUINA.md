# Seven Desk — Teste controlado em dois computadores

> **Somente pré-alfa e laboratório privado.** Nenhuma configuração desta página é homologada para produção. O servidor ainda usa memória temporária e a mídia **não é criptografada ponta a ponta**: o servidor intermediário tem acesso aos quadros PNG/JPEG, embora a conexão cliente↔proxy possa usar TLS.

## Objetivo

Testar duas instalações Seven Desk com identidades Ed25519 distintas, conectadas ao mesmo **servidor HTTPS de desenvolvimento** por uma VPN privada. O modo experimental inclui convite → consentimento → envio manual PNG ou prévia JPEG periódica pelo host Windows, macOS ou Linux (até 15 FPS) → visualização pelo operador → revogação. Mouse e teclado básicos exigem permissões separadas e acionamento voluntário no host; no Linux são habilitados apenas em X11. **Não há vídeo codificado de baixa latência, criptografia ponta a ponta nem teste E2E homologado entre computadores reais.**

## Pré-requisitos

- Um servidor Linux/Windows com Rust e backend do repositório, mais HTTPS terminado em proxy reverso (exemplo [Caddyfile](../deploy/lab/Caddyfile.example)).
- Domínio DNS controlado, certificado válido e acesso restrito a endereços da VPN do laboratório. *Nunca abrir a porta HTTP 43821 diretamente na rede.*
- Dois computadores com o Seven Desk Tauri instalado, com identidades distintas. Host Windows: Windows.Graphics.Capture; host macOS: conceder Gravação de Tela e Acessibilidade para controle; host Linux: testar primeiro em X11. Wayland depende de portal ScreenCast/PipeWire compatível e o controle de entrada ainda não está habilitado.
- Conectividade VPN entre os computadores e o proxy. Configurar firewall para negar acesso de fora da VPN.
- Comandos de build e pré-requisitos em [IMPLEMENTACAO.md](IMPLEMENTACAO.md).

## Iniciar o servidor

No servidor, use `cargo run -p seven-server`. Ele somente escuta em `127.0.0.1:43821`. O proxy Caddy usa a mesma máquina para encaminhar HTTPS para esse loopback. Não use variáveis para permitir `SEVEN_BIND` em interface externa: **o backend rejeita binding não loopback**.

A configuração de exemplo restringe o cliente à faixa VPN `10.42.0.0/16`. Ajuste a faixa e o hostname para o seu laboratório, além de criar o DNS e TLS válidos. A instalação do Caddy e do certificado segue a documentação oficial da sua infraestrutura. A ausência de certificado válido deve causar **falha fechada** no cliente.

## Configurar os dois clientes

Em **Configurações → Endereço do servidor**, informe a URL HTTPS do proxy e clique em **Salvar** nos dois clientes. A escolha fica salva no aplicativo. Salvar interrompe a captura e a entrada locais, tenta encerrar as sessões no servidor anterior e reconecta o dispositivo automaticamente. Se o servidor anterior estiver indisponível, uma revogação pode não ser confirmada; as sessões temporárias expiram automaticamente.

Em redes que exigem proxy de saída, use **Configurações → Proxy de rede**. O modo manual aceita um proxy HTTP(S) sem credenciais; a detecção usa `HTTP_PROXY`, `HTTPS_PROXY` e `NO_PROXY`, além do proxy estático configurado no Windows ou macOS. No Linux, usa as variáveis da rede. O endereço de loopback sempre é acessado diretamente. Trocar o proxy encerra as sessões atuais e reconecta. Proxy PAC ou autenticação NTLM do Windows ainda não são suportados.

O host pode continuar ativo na área de notificação quando a janela principal é fechada. **Iniciar ao entrar no sistema** registra o aplicativo na inicialização deste usuário; ele não funciona após logout e nunca dispensa o aceite do convite. Use **Sair do Seven Desk** no ícone para encerrar o agente.

Sem configuração, o cliente usa `http://127.0.0.1:43821`. Fora do loopback, **HTTPS é obrigatório** e o certificado é validado pelo cliente Rust. URLs com credenciais, query string, fragmentos ou caminhos extras são rejeitadas. HTTP para LAN, IP externo e qualquer domínio é recusado.

Para automação de laboratório, a variável **`SEVEN_DESK_DEV_SERVER_URL`** do processo nativo tem prioridade sobre a opção salva e bloqueia a edição na interface. Exemplos:

PowerShell no Windows (inicie o cliente a partir desse mesmo terminal):

```powershell
$env:SEVEN_DESK_DEV_SERVER_URL = "https://lab-seven.example.com"
npm run tauri:dev
```

Linux/macOS:

```bash
export SEVEN_DESK_DEV_SERVER_URL="https://lab-seven.example.com"
npm run tauri:dev
```

`lab-seven.example.com` é **apenas um exemplo**; não existe um servidor compartilhado administrado pelo projeto. A configuração salva não torna o backend apto para produção.

## Cenário de ponta a ponta

1. Abrir o aplicativo em cada dispositivo e copiar o próprio ID. A identidade e o registro no servidor são preparados automaticamente; o app renova presença com heartbeat enquanto a janela permanece aberta.
2. No operador, digitar o ID do host e **Solicitar autorização**. O host deve aparecer na própria fila de convites.
3. No host, conferir operador e permissões na janela separada e clicar em **Aceitar** ou **Rejeitar**. Sem aceitar, **nenhum quadro deve ser entregue**.
4. Ao aceitar, confirmar que o compartilhamento começa e apenas a janela separada de atendimento minimiza automaticamente. A janela principal do host permanece aberta e a janela do operador mostra a tela remota. Mouse/teclado marcados ficam ativos sem outro botão. Ao restaurar a janela de atendimento, ela mostra **Finalizar**.
5. Confirmar que os quadros mudam enquanto há atividade visível no host e que apenas uma captura roda por vez; medir FPS e latência reais. O host pode enviar um quadro isolado em **Autorizações ativas** como teste adicional.
6. O host restaura a janela e clica em **Finalizar**. Verificar que novos quadros deixam de chegar; após 15 segundos, os últimos quadros no servidor expiram. O operador deve deixar de visualizar conteúdo antigo como se fosse atual.
7. No host, **Encerrar autorização de tela**. A próxima leitura no operador deve ser negada, mesmo que um quadro estivesse presente antes.
8. Repetir teste recusando convite, expirando convite, desligando servidor, expirando presença e tentando usar ID inválido.
9. Quando o convite pedir **chat**, conceder essa permissão separadamente e conferir troca de mensagens nos dois sentidos. Revogar a permissão e confirmar que novas mensagens são recusadas.
10. Em cada host com dois monitores, selecionar o secundário antes de compartilhar e confirmar que ele é exibido sem habilitar mouse remoto. Voltar ao principal e reiniciar os quadros; o mouse autorizado pode ser reativado para a mesma sessão quando o sistema permitir.
11. Alterar o endereço do servidor em **Configurações** durante uma sessão de laboratório. Confirmar parada imediata da captura e entrada, tentativa de encerramento da sessão anterior e reconexão automática.
12. Minimizar a janela do host e confirmar que quadros e entrada autorizada continuam. Restaurar e clicar em **Finalizar** para confirmar parada. Repetir com troca de monitor, desconexão de rede e revogação enquanto um quadro está em envio.
13. Executar a matriz Windows→macOS, macOS→Windows, Linux X11→Windows e Windows→Linux X11. Em macOS, negar e depois conceder cada permissão do sistema. Em Linux Wayland, registrar se o portal mostra o seletor, se o monitor aparece e se há quadros; entrada remota deve permanecer indisponível.

## Evidências de homologação

Registrar SHA do commit, build e versão dos sistemas, topologia VPN/DNS/TLS, tempo de envio, tamanho dos quadros PNG/JPEG, sucesso/falha de cada etapa, logs sem dados sensíveis e vídeos/screenhots fictícios. Não gravar material privado. O CI automatizado não comprova este teste presencial.

**Antes de ambiente produtivo:** persistência e revogação de dispositivos, hardening contra abuso, ACL multi-tenant, criptografia ponta a ponta da mídia, relay/stream contínuo, serviço do host, instaladores assinados, permissões nativas e testes de invasão.
