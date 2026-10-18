# Servidor e relay de laboratório

Esta pasta sobe a API Rust e o relay HTTP assinado **no mesmo processo**. O gateway Caddy termina HTTPS. O backend continua em memória, sem contas, MFA, criptografia ponta a ponta ou homologação para Internet pública. Use somente numa VPN controlada.

1. Instale Docker e Docker Compose em uma máquina que ficará ligada.
2. Aponte um domínio para essa máquina e obtenha um certificado TLS confiável para os clientes. Se usar Cloudflare Free, mantenha o registro **DNS only** para este transporte e use um certificado publicamente confiável no Caddy. Um certificado Cloudflare Origin CA isolado não é confiável pelo aplicativo quando o acesso é direto.
3. Copie `.env.example` para `.env` e substitua domínio e faixa da VPN. Coloque `fullchain.pem` e `privkey.pem` em `tls/`. Proteja a chave privada no host.
4. Nesta pasta, execute `docker compose up -d --build`.
5. Pela VPN, confira `https://SEU_DOMINIO/healthz` e `https://SEU_DOMINIO/v1/relay/healthz`. Nos dois aplicativos, salve o mesmo endereço HTTPS em **Configurações → Endereço do servidor**.

O backend escuta somente em `127.0.0.1:43821` dentro do namespace de rede compartilhado com o gateway. A porta pública é 443, e o gateway recusa IPs fora de `SEVEN_VPN_CIDR`. Configure o firewall para a mesma restrição. Quando houver um proxy corporativo no cliente, configure-o em **Proxy de rede**; a detecção lê as variáveis de proxy e a configuração estática do Windows ou macOS. PAC e NTLM ainda não são suportados.

O relay encaminha o último quadro JPEG/PNG, mouse, teclado e chat de sessões autorizadas. Ele não é TURN, não cria ligação P2P e não oculta os quadros do servidor. A prévia atual é limitada a 15 FPS. Não rode réplicas do backend: o estado em memória não é compartilhado.
