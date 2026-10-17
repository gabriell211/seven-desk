# Seven Desk — Teste controlado em dois computadores

> **Somente pré-alfa e laboratório privado.** Nenhuma configuração desta página é homologada para produção. O servidor ainda usa memória temporária e a mídia **não é criptografada ponta a ponta**: o servidor intermediário tem acesso ao quadro PNG, embora a conexão cliente↔proxy possa usar TLS.

## Objetivo

Testar duas instalações Seven Desk com identidades Ed25519 distintas, conectadas ao mesmo **servidor HTTPS de desenvolvimento** por uma VPN privada. O modo experimental já inclui convite → consentimento → envio manual ou periódico de quadros PNG pelo host Windows (~1 FPS) → visualização pelo operador → revogação; mouse e teclado básicos no Windows exigem permissões separadas e acionamento voluntário no host. **Não há vídeo de baixa latência, criptografia ponta a ponta nem teste E2E homologado entre computadores reais.**

## Pré-requisitos

- Um servidor Linux/Windows com Rust e backend do repositório, mais HTTPS terminado em proxy reverso (exemplo [Caddyfile](../deploy/lab/Caddyfile.example)).
- Domínio DNS controlado, certificado válido e acesso restrito a endereços da VPN do laboratório. *Nunca abrir a porta HTTP 43821 diretamente na rede.*
- Dois computadores com o Seven Desk Tauri instalado, com identities distintas; um deles deve ser Windows para captura nesta fase.
- Conectividade VPN entre os computadores e o proxy. Configurar firewall para negar acesso de fora da VPN.
- Comandos de build e pré-requisitos em [IMPLEMENTACAO.md](IMPLEMENTACAO.md).

## Iniciar o servidor

No servidor, use `cargo run -p seven-server`. Ele somente escuta em `127.0.0.1:43821`. O proxy Caddy usa a mesma máquina para encaminhar HTTPS para esse loopback. Não use variáveis para permitir `SEVEN_BIND` em interface externa: **o backend rejeita binding não loopback**.

A configuração de exemplo restringe o cliente à faixa VPN `10.42.0.0/16`. Ajuste a faixa e o hostname para o seu laboratório, além de criar o DNS e TLS válidos. A instalação do Caddy e do certificado segue a documentação oficial da sua infraestrutura. A ausência de certificado válido deve causar **falha fechada** no cliente.

## Configurar os dois clientes

O aplicativo lê a variável **`SEVEN_DESK_DEV_SERVER_URL`** somente no processo nativo. Sem a variável, usa `http://127.0.0.1:43821`. Quando definido para outro host, o esquema **HTTPS é obrigatório**, e o certificado é validado pelo cliente Rust. URLs com credenciais, query string, fragmentos ou caminhos extras são rejeitadas. HTTP para LAN, IP externo e qualquer domínio é recusado.

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

`lab-seven.example.com` é **apenas um exemplo**, não existe um servidor compartilhado administrado pelo projeto. Uma instalação distribuída precisará de interface e persistência da configuração do servidor antes de homologação.

## Cenário de ponta a ponta

1. Em cada dispositivo, **Ativar identidade**, copiar o próprio ID e **Registrar no servidor**. O app renova presença com heartbeat enquanto a janela permanece aberta.
2. No operador, digitar o ID do host e **Solicitar autorização**. O host deve aparecer na própria fila de convites.
3. No host, clicar em **Autorizar negociação**. Sem aceitar, **nenhum quadro deve ser entregue**.
4. Em **Autorizações ativas**, no host Windows, clicar em **Compartilhar um quadro**. No operador, clicar em **Ver último quadro autorizado** para receber a imagem estática.
5. Para baixa frequência, o host pode clicar em **Iniciar quadros (~1 FPS)**, com indicador persistente de compartilhamento. O operador pode clicar em **Atualizar automaticamente (~1 FPS)**. Confirmar que os frames mudam enquanto há atividade visível no host e que apenas uma captura roda por vez.
6. O host clica em **Parar quadros** ou minimiza/oculta a janela. Verificar que novos quadros deixam de chegar; após 15 segundos, os últimos quadros no servidor expiram. O operador deve deixar de visualizar conteúdo antigo como se fosse atual.
7. No host, **Encerrar autorização de tela**. A próxima leitura no operador deve ser negada, mesmo que um quadro estivesse presente antes.
8. Repetir teste recusando convite, expirando convite, desligando servidor, expirando presença e tentando usar ID inválido.

## Evidências de homologação

Registrar SHA do commit, build e versão dos sistemas, topologia VPN/DNS/TLS, tempo de envio, tamanho do PNG, sucesso/falha de cada etapa, logs sem dados sensíveis e vídeos/screenhots fictícios. Não gravar material privado. O CI automatizado não comprova este teste presencial.

**Antes de ambiente produtivo:** persistência e revogação de dispositivos, hardening contra abuso, ACL multi-tenant, criptografia ponta a ponta da mídia, relay/stream contínuo, serviço do host, instaladores assinados, permissões nativas e testes de invasão.
