# Seven Desk — Roadmap operacional e rastreabilidade

> **Situação em 16/10/2026:** produto em planejamento. O backlog tem **73 issues** e não corresponde a funcionalidades implementadas. Não há 1.0 homologada.

## Convenções e critérios de priorização

- P0/P1/P2/P3/P4 definem a sequência proposta para chegar à 1.0; dependências podem mover uma tarefa, mas não antecipar permissões ou testes.
- P5 contém funcionalidades de expansão; só entram na 1.0 mediante nova decisão explícita e atualização de todas as matrizes.
- Toda issue precisa de testes positivos/negativos, critérios de aceite e evidência real. Estados: `Backlog → Ready → In Progress → Review → QA → Done`.
- Uma dependência citada como `#numero` precisa estar concluída ou ter interface estável e risco aceito antes de iniciar integração dependente.
- **Documentação de referência:** [Matriz funcional](MATRIZ_FUNCIONAL.md), [Arquitetura](ARQUITETURA.md), [Homologação](HOMOLOGACAO.md), [Relatório de auditoria](AUDITORIA_ALINHAMENTO.md).

## P0 — Fundação, segurança e decisões arquiteturais — 14 issues

| Issue | Entrega verificável |
| --- | --- |
| [#1](https://github.com/gabriell211/seven-desk/issues/1) | Estruturar monorepo Rust, Tauri, React e contratos |
| [#2](https://github.com/gabriell211/seven-desk/issues/2) | Modelo de ameaças, privacidade e ADRs obrigatórias |
| [#3](https://github.com/gabriell211/seven-desk/issues/3) | Contratos versionados de sessão, dispositivo e permissões |
| [#4](https://github.com/gabriell211/seven-desk/issues/4) | CI, testes, auditoria de dependências e branch protection |
| [#5](https://github.com/gabriell211/seven-desk/issues/5) | Identidade visual, design system e protótipo navegável |
| [#6](https://github.com/gabriell211/seven-desk/issues/6) | Identidade persistente e ID de dispositivo copiável |
| [#7](https://github.com/gabriell211/seven-desk/issues/7) | Serviço de sinalização e presença em Rust/Axum |
| [#8](https://github.com/gabriell211/seven-desk/issues/8) | Máquina de estados e autorização de sessões |
| [#9](https://github.com/gabriell211/seven-desk/issues/9) | PoC de transporte criptografado P2P e relay |
| [#49](https://github.com/gabriell211/seven-desk/issues/49) | Separar UI Tauri, host service Rust e IPC privilegiado |
| [#50](https://github.com/gabriell211/seven-desk/issues/50) | Vinculação criptográfica dos pares e identidade do host |
| [#51](https://github.com/gabriell211/seven-desk/issues/51) | ADR e PoC de codecs, captura e renderização de baixa latência |
| [#52](https://github.com/gabriell211/seven-desk/issues/52) | Ambiente local sem Docker obrigatório, fixtures e scripts reproduzíveis |
| [#67](https://github.com/gabriell211/seven-desk/issues/67) | Definir licença, direitos de dependências e marcas antes do código |

**Gate P0:** Build reproduzível; arquitetura de processos e IPC definida; autenticidade do par, ameaça e licença revisadas; protocolo e seleção preliminar de transporte/codecs com evidências.

## P1 — MVP Windows assistido e instalador — 8 issues

| Issue | Entrega verificável |
| --- | --- |
| [#10](https://github.com/gabriell211/seven-desk/issues/10) | Captura nativa de tela e monitores |
| [#11](https://github.com/gabriell211/seven-desk/issues/11) | Mouse, teclado e atalhos remotos com permissão |
| [#12](https://github.com/gabriell211/seven-desk/issues/12) | Decodificação, renderização e qualidade adaptativa |
| [#13](https://github.com/gabriell211/seven-desk/issues/13) | Solicitação de suporte, consentimento e indicador permanente |
| [#14](https://github.com/gabriell211/seven-desk/issues/14) | Homologar MVP Windows↔Windows em duas máquinas reais |
| [#15](https://github.com/gabriell211/seven-desk/issues/15) | Instalador oficial Seven Desk para Windows |
| [#53](https://github.com/gabriell211/seven-desk/issues/53) | Primeira execução, bandeja, inicialização e estados do aplicativo |
| [#54](https://github.com/gabriell211/seven-desk/issues/54) | Serviço Windows, instalação opcional e limites de privilégios |

**Gate P1:** Duas máquinas reais Windows↔Windows, LAN e WAN controlada; consentimento, controle, revogação e instalador oficial Windows validados.

## P2 — Beta e recursos profissionais de suporte — 18 issues

| Issue | Entrega verificável |
| --- | --- |
| [#16](https://github.com/gabriell211/seven-desk/issues/16) | Chat textual seguro dentro da sessão |
| [#17](https://github.com/gabriell211/seven-desk/issues/17) | Sincronização de área de transferência sob permissão |
| [#18](https://github.com/gabriell211/seven-desk/issues/18) | Transferência bidirecional e gerenciador de arquivos |
| [#19](https://github.com/gabriell211/seven-desk/issues/19) | ICE/NAT traversal, relay autenticado e modos LAN/WAN |
| [#20](https://github.com/gabriell211/seven-desk/issues/20) | Host e viewer X11/Wayland com portais nativos |
| [#21](https://github.com/gabriell211/seven-desk/issues/21) | Host e viewer com captura e permissões do sistema |
| [#22](https://github.com/gabriell211/seven-desk/issues/22) | Abstração multiplataforma de teclado, mouse e layout |
| [#23](https://github.com/gabriell211/seven-desk/issues/23) | Instaladores Seven Desk Linux/macOS e atualização assinada |
| [#24](https://github.com/gabriell211/seven-desk/issues/24) | Lista de dispositivos, contatos, grupos e favoritos |
| [#25](https://github.com/gabriell211/seven-desk/issues/25) | Reconexão, timeouts e recuperação de sessão |
| [#26](https://github.com/gabriell211/seven-desk/issues/26) | Acessibilidade, PT-BR e internacionalização |
| [#55](https://github.com/gabriell211/seven-desk/issues/55) | Proxy corporativo HTTP CONNECT, PAC e diagnóstico de firewall |
| [#56](https://github.com/gabriell211/seven-desk/issues/56) | Gerenciador de arquivos em modo independente e dois painéis |
| [#57](https://github.com/gabriell211/seven-desk/issues/57) | Descoberta local opt-in e conexão LAN |
| [#58](https://github.com/gabriell211/seven-desk/issues/58) | Centro de diagnóstico, logs sanitizados e relatório de suporte |
| [#59](https://github.com/gabriell211/seven-desk/issues/59) | Elevação autorizada, bloqueio de estação e teclas especiais |
| [#60](https://github.com/gabriell211/seven-desk/issues/60) | Diagnóstico visual, seletor de entrada e troca de direção |
| [#64](https://github.com/gabriell211/seven-desk/issues/64) | Convites de sessão via link profundo e fila de solicitações |

**Gate P2:** Transferência, chat, LAN/proxy, arquivos, reconexão e suporte Linux/macOS nas versões testadas; instaladores e erros de permissão homologados.

## P3 — Autenticação e administração segura — 10 issues

| Issue | Entrega verificável |
| --- | --- |
| [#27](https://github.com/gabriell211/seven-desk/issues/27) | Acesso não assistido com inscrição explícita e revogação |
| [#28](https://github.com/gabriell211/seven-desk/issues/28) | Contas, autenticação forte, MFA e recuperação |
| [#29](https://github.com/gabriell211/seven-desk/issues/29) | Perfis de permissão, ACL e políticas locais por dispositivo |
| [#30](https://github.com/gabriell211/seven-desk/issues/30) | Reinicialização remota autorizada e retomada segura |
| [#31](https://github.com/gabriell211/seven-desk/issues/31) | Múltiplos monitores, seleção, escala e resoluções dinâmicas |
| [#32](https://github.com/gabriell211/seven-desk/issues/32) | Organizações, usuários, equipes e parque de dispositivos |
| [#33](https://github.com/gabriell211/seven-desk/issues/33) | Auditoria de sessões, privacidade, retenção e LGPD |
| [#61](https://github.com/gabriell211/seven-desk/issues/61) | Tunelamento TCP direto e reverso com escopo de destino |
| [#63](https://github.com/gabriell211/seven-desk/issues/63) | Console remoto interativo opt-in e sem privilégios implícitos |
| [#65](https://github.com/gabriell211/seven-desk/issues/65) | Apelidos, namespace corporativo e allowlist de operadores |

**Gate P3:** Não assistido opt-in, autenticação forte, autorização no host, gestão de dispositivos, auditoria, capacidade de revogação e serviços privilegiados corretamente isolados.

## P4 — Empresa, testes de produção e release 1.0 — 9 issues

| Issue | Entrega verificável |
| --- | --- |
| [#34](https://github.com/gabriell211/seven-desk/issues/34) | SSO/OIDC, RBAC corporativo e desligamento de usuários |
| [#35](https://github.com/gabriell211/seven-desk/issues/35) | API de integração, webhooks assinados e rate limit |
| [#36](https://github.com/gabriell211/seven-desk/issues/36) | Benchmarks de latência, FPS, consumo e qualidade da imagem |
| [#37](https://github.com/gabriell211/seven-desk/issues/37) | Hardening, auditoria e testes de penetração antes do GA |
| [#38](https://github.com/gabriell211/seven-desk/issues/38) | Homologação 1.0, instaladores Seven Desk e operação em produção |
| [#66](https://github.com/gabriell211/seven-desk/issues/66) | Políticas distribuídas, provisionamento gerenciado e MDM |
| [#68](https://github.com/gabriell211/seven-desk/issues/68) | Escalabilidade de sinalização/relay, SLOs e custo operacional |
| [#69](https://github.com/gabriell211/seven-desk/issues/69) | Migrações, backup, restore e compatibilidade entre versões |
| [#72](https://github.com/gabriell211/seven-desk/issues/72) | Matriz completa de compatibilidade por SO, versões e arquitetura |

**Gate P4:** Matriz de compatibilidade publicada; SSO/gestão, custo de relay, backup/restore, hardening, performance e instalação/atualização assinada; gate #38 aprovado.

## P5 — Expansão pós-1.0 (escopo futuro) — 14 issues

| Issue | Entrega verificável |
| --- | --- |
| [#39](https://github.com/gabriell211/seven-desk/issues/39) | Viewer Android e iOS/iPadOS com controle compatível |
| [#40](https://github.com/gabriell211/seven-desk/issues/40) | Wake-on-LAN e descoberta autorizada em rede local |
| [#41](https://github.com/gabriell211/seven-desk/issues/41) | Impressão remota com driver/ponte autorizada |
| [#42](https://github.com/gabriell211/seven-desk/issues/42) | Áudio remoto e dispositivo de saída configurável |
| [#43](https://github.com/gabriell211/seven-desk/issues/43) | Gravação de sessões com consentimento e retenção |
| [#44](https://github.com/gabriell211/seven-desk/issues/44) | Quadro de anotações e sessão colaborativa |
| [#45](https://github.com/gabriell211/seven-desk/issues/45) | Fila de atendimentos, convites e sessões em abas |
| [#46](https://github.com/gabriell211/seven-desk/issues/46) | Modo de privacidade e restrições de sessão |
| [#47](https://github.com/gabriell211/seven-desk/issues/47) | Implantação self-hosted de sinalização, relay e console |
| [#48](https://github.com/gabriell211/seven-desk/issues/48) | Cliente personalizado e política de implantação corporativa |
| [#62](https://github.com/gabriell211/seven-desk/issues/62) | Rede virtual VPN isolada por dispositivo |
| [#70](https://github.com/gabriell211/seven-desk/issues/70) | Captura e gravação local fora de sessão remota |
| [#71](https://github.com/gabriell211/seven-desk/issues/71) | Atalhos de URL, CLI documentada e suporte a automação segura |
| [#73](https://github.com/gabriell211/seven-desk/issues/73) | Distribuição móvel corporativa, políticas e restrições nativas |

**Gate P5:** Apenas após avaliação de produto/segurança e validação por sistema. Não bloqueia 1.0 se explicitamente excluído do escopo.

## Sequência recomendada de início

1. **Fundação e risco:** #1, #2, #3, #4, #49, #50, #51, #52 e #67.
2. **Backend e sessão:** #6, #7, #8, #9; implementar o handshake e a validação no host antes de transmitir mídia.
3. **Primeiro host/viewer real:** #10, #11, #12, #13, #53, #54 e #14. Servidor de sinalização, transporte e host precisam ser reais; permissões negadas obrigatoriamente testadas.
4. **Distribuição:** #15 para Windows, #23 para Linux/macOS. A instalação padrão é sempre pelo **Instalador Seven Desk**, com serviço opt-in.
5. **Beta e expansão:** recursos P2 em paralelo conforme dependências; depois P3/P4 e homologação; P5 sem prometer entrega na 1.0.

## Revisão de escopo e prontidão

- **Alinhamento documental:** backlog sem ciclos de dependências e com critérios/testes por issue, segundo auditoria feita após as últimas alterações.
- **Prontidão de código:** **não atingida**; ainda precisam ser escritos serviços, clientes, contratos, testes, instaladores e pipelines.
- **Produção:** **não homologada**. O gate final é a [issue #38](https://github.com/gabriell211/seven-desk/issues/38), com evidências da [matriz de testes](HOMOLOGACAO.md).
- **Licença:** pendente (#67); não tratar repositório público como licença de redistribuição.
- **Serviços externos:** definir endpoints, segredos, topologia, quotas, logs e política de custo antes de disponibilizar a produção.

> Não fechar issues por protótipo, mock, screenshots, docs ou afirmação sem testes. A conclusão deve referenciar PR/commit e evidências reproduzíveis.
