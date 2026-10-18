# Seven Desk — infraestrutura necessária

> **Nova opção OSS em preparação (18/10/2026):** [deploy/rustdesk-oss](../deploy/rustdesk-oss/README.md) sobe o rendezvous `hbbs` e o relay `hbbr` de protocolo Protobuf TCP/UDP. O diagnóstico `TestNatRequest` do Seven Desk não significa integração de sessões; consulte [plano de migração](RUSTDESK_MIGRACAO.md) e mantenha o laboratório HTTP atual sob VPN até E2E real.

## Resposta direta

**Vercel não é obrigatório.** O aplicativo desktop é instalado nos computadores. Para um teste local, basta executar o servidor Rust em loopback na mesma máquina; para duas máquinas no laboratório, o projeto já documenta proxy HTTPS e VPN privada em [TESTE_MULTIMAQUINA.md](TESTE_MULTIMAQUINA.md).

Um endereço gratuito `*.vercel.app` identifica um projeto implantado na Vercel; ele não é um domínio próprio para apontar diretamente ao servidor Rust. Para publicar este servidor com um nome controlado pelo projeto, use um domínio próprio e um subdomínio dedicado, além de uma máquina que mantenha o processo ligado. Vercel pode hospedar um painel web separado.

Para uma sessão remota semelhante à do AnyDesk, a VPN de sistema não precisa iniciar junto com o aplicativo. O AnyDesk usa conexão direta ou seu relay para a sessão; a função AnyDesk VPN é um modo separado. O Seven Desk usa somente o relay HTTP assinado nesta fase, sem conexão direta entre pares. Uma VPN externa continua sendo a barreira de rede recomendada para os testes atuais, pois ainda faltam criptografia ponta a ponta e endurecimento do servidor para Internet pública.

O endereço HTTPS desse laboratório pode ser salvo em **Configurações** no aplicativo instalado. Isso facilita o teste em duas máquinas, mas ainda depende de uma VPN, proxy TLS e servidor executados pelo responsável pelo laboratório.

O servidor atual guarda presença, sessões, quadros e chat **somente em memória**, aceita bind apenas em loopback e sinaliza `production_ready: false`. O relay HTTP de quadros e comandos autorizados está integrado ao mesmo processo. Há uma receita de implantação HTTPS restrita à VPN em [deploy/self-hosted](../deploy/self-hosted/README.md). Publicá-lo sem essa restrição não cria um serviço seguro para Internet pública.

## Topologia necessária para uso pela Internet

| Componente | Necessidade | Estado neste repositório |
| --- | --- | --- |
| Aplicativos desktop assinados e atualizáveis | Host e operador | Pacotes de QA sem assinatura; homologação pendente |
| Serviço host nativo e consentimento | Captura e controle quando a janela principal está fechada, com usuário conectado | Agente na sessão do usuário, com ícone na área de notificação e opção de iniciar ao entrar; não é serviço de sistema após logout |
| Sinalização e presença sempre disponíveis | Descobrir pares, negociar, revogar | API Rust experimental em memória; gateway HTTPS de laboratório documentado |
| Transporte de mídia e relay/TURN | Levar tela e entrada entre redes | Relay HTTP assinado integrado, até 15 FPS, via servidor HTTPS; sem TURN, P2P ou codec de vídeo |
| Criptografia ponta a ponta e autenticação de pares | Proteger tela, entrada, chat e arquivos inclusive diante do relay | Não implementada |
| PostgreSQL e migrações/backup | Persistir contas, dispositivos, políticas e auditoria | Não implementados |
| DNS, TLS, monitoramento e limites de abuso | Operação e recuperação | Apenas exemplo de proxy restrito a laboratório |

Um **servidor/serviço contínuo** (VM, container ou plataforma de web service) será necessário para sinalização e relay. O relay de mídia terá consumo de banda; a escolha de instância e custo depende de medições reais de sessões simultâneas, bitrate, latência e regiões. PostgreSQL pode ser gerenciado. Um painel web pode ser hospedado na Vercel, mas é opcional; o cliente remoto continuará sendo nativo.

Não escolher provedor, capacidade ou custo definitivo antes da PoC de transporte e dos testes de duas máquinas em LAN/WAN. A arquitetura e os gates estão em [ARQUITETURA.md](ARQUITETURA.md) e [HOMOLOGACAO.md](HOMOLOGACAO.md).
