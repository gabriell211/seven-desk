# Seven Desk — Roadmap operacional e backlog

> **Estado verificado:** repositório iniciado com documentação e backlog, sem implementação funcional homologada. Todas as issues abaixo representam trabalho planejado. Nenhuma está pronta apenas por estar cadastrada.

## Convenções

- **P0 / Fundação:** pré-requisitos arquiteturais, segurança e organização.
- **P1 / MVP assistido:** primeiro suporte real Windows↔Windows.
- **P2 / Beta desktop:** multiplataforma, estabilidade e ferramentas práticas.
- **P3 / Profissional:** autenticação, acesso não assistido, políticas e gestão.
- **P4 / 1.0/GA:** auditoria, performance, enterprise e release de produção.
- **P5 / Expansão:** móveis, recursos avançados e implantação personalizada.

O prefixo `[P#]` no título é o indicador de fase. A atribuição de versões e datas exige planejamento com evidência. As dependências estão no corpo de cada issue.

## P0 — Fundação e provas de conceito

| Issue | Entregável |
| --- | --- |
| [#1](https://github.com/gabriell211/seven-desk/issues/1) | Monorepo e configuração reproduzível |
| [#2](https://github.com/gabriell211/seven-desk/issues/2) | Modelo de ameaças, ADRs e privacidade |
| [#3](https://github.com/gabriell211/seven-desk/issues/3) | Protocolo versionado |
| [#4](https://github.com/gabriell211/seven-desk/issues/4) | CI, ferramentas e qualidade |
| [#5](https://github.com/gabriell211/seven-desk/issues/5) | Identidade visual, UI e acessibilidade inicial |
| [#6](https://github.com/gabriell211/seven-desk/issues/6) | Identidade persistente de dispositivo |
| [#7](https://github.com/gabriell211/seven-desk/issues/7) | Serviço de sinalização e presença |
| [#8](https://github.com/gabriell211/seven-desk/issues/8) | Máquina de estados e autorização |
| [#9](https://github.com/gabriell211/seven-desk/issues/9) | PoC transporte protegido e relay |

**Gate P0:** contratos aprovados, threat model disponível, CI reproduzível, identidade e protocolo protegidos com testes negativos.

## P1 — MVP de suporte remoto real

| Issue | Entregável |
| --- | --- |
| [#10](https://github.com/gabriell211/seven-desk/issues/10) | Captura de tela Windows |
| [#11](https://github.com/gabriell211/seven-desk/issues/11) | Entrada remota Windows |
| [#12](https://github.com/gabriell211/seven-desk/issues/12) | Pipeline de mídia e renderização |
| [#13](https://github.com/gabriell211/seven-desk/issues/13) | Consentimento, permissão e indicador de sessão |
| [#14](https://github.com/gabriell211/seven-desk/issues/14) | E2E de duas máquinas reais |
| [#15](https://github.com/gabriell211/seven-desk/issues/15) | Instalador oficial Seven Desk para Windows |

**Gate P1:** acesso assistido Windows↔Windows funciona ponta a ponta, nunca aceita sessão sem autorização, mantém encerramento imediato pelo host e pode ser instalado pelo instalador oficial Seven Desk no Windows.

## P2 — Beta multiplataforma

| Issue | Entregável |
| --- | --- |
| [#16](https://github.com/gabriell211/seven-desk/issues/16) | Chat |
| [#17](https://github.com/gabriell211/seven-desk/issues/17) | Clipboard |
| [#18](https://github.com/gabriell211/seven-desk/issues/18) | Transferência e arquivos |
| [#19](https://github.com/gabriell211/seven-desk/issues/19) | NAT/relay LAN/WAN |
| [#20](https://github.com/gabriell211/seven-desk/issues/20) | Linux X11/Wayland |
| [#21](https://github.com/gabriell211/seven-desk/issues/21) | macOS |
| [#22](https://github.com/gabriell211/seven-desk/issues/22) | Entrada multiplataforma |
| [#23](https://github.com/gabriell211/seven-desk/issues/23) | Instaladores oficiais Seven Desk para Linux/macOS e atualização assinada |
| [#24](https://github.com/gabriell211/seven-desk/issues/24) | Contatos, favoritos e grupos |
| [#25](https://github.com/gabriell211/seven-desk/issues/25) | Recuperação de conectividade |
| [#26](https://github.com/gabriell211/seven-desk/issues/26) | A11y e internacionalização |

**Gate P2:** matriz homologada por SO e rede, instalação pelo instalador/pacote oficial Seven Desk em Linux e macOS, fluxo de suporte seguro, sem regressões críticas.

## P3 — Operação profissional

| Issue | Entregável |
| --- | --- |
| [#27](https://github.com/gabriell211/seven-desk/issues/27) | Acesso não assistido opt-in |
| [#28](https://github.com/gabriell211/seven-desk/issues/28) | Contas e MFA |
| [#29](https://github.com/gabriell211/seven-desk/issues/29) | Perfis/ACL |
| [#30](https://github.com/gabriell211/seven-desk/issues/30) | Reinício remoto autorizado |
| [#31](https://github.com/gabriell211/seven-desk/issues/31) | Múltiplos monitores |
| [#32](https://github.com/gabriell211/seven-desk/issues/32) | Organizações e console |
| [#33](https://github.com/gabriell211/seven-desk/issues/33) | Auditoria e LGPD |

**Gate P3:** MFA, acesso não assistido voluntário, política com deny prioritário, audit trail e revogação testada.

## P4 — Preparação empresarial e 1.0

| Issue | Entregável |
| --- | --- |
| [#34](https://github.com/gabriell211/seven-desk/issues/34) | SSO e RBAC |
| [#35](https://github.com/gabriell211/seven-desk/issues/35) | API e webhooks |
| [#36](https://github.com/gabriell211/seven-desk/issues/36) | Benchmarks reproduzíveis |
| [#37](https://github.com/gabriell211/seven-desk/issues/37) | Hardening e avaliação de segurança |
| [#38](https://github.com/gabriell211/seven-desk/issues/38) | Gate 1.0: release e homologação |

**Gate P4:** nenhum bug crítico ou vulnerabilidade grave pendente sem mitigação aprovada; instaladores assinados; runbooks de backup, restauração, observabilidade e rollback; suíte E2E aprovada.

## P5 — Ampliação de capacidades

| Issue | Entregável |
| --- | --- |
| [#39](https://github.com/gabriell211/seven-desk/issues/39) | Viewer mobile |
| [#40](https://github.com/gabriell211/seven-desk/issues/40) | Wake-on-LAN |
| [#41](https://github.com/gabriell211/seven-desk/issues/41) | Impressão remota |
| [#42](https://github.com/gabriell211/seven-desk/issues/42) | Áudio remoto |
| [#43](https://github.com/gabriell211/seven-desk/issues/43) | Gravação consentida |
| [#44](https://github.com/gabriell211/seven-desk/issues/44) | Anotações / colaboração |
| [#45](https://github.com/gabriell211/seven-desk/issues/45) | Fila e sessões em abas |
| [#46](https://github.com/gabriell211/seven-desk/issues/46) | Modo de privacidade |
| [#47](https://github.com/gabriell211/seven-desk/issues/47) | Implantação self-hosted |
| [#48](https://github.com/gabriell211/seven-desk/issues/48) | Cliente personalizado |

**Importante:** não confundir prioridade P5 com requisito funcional obrigatório da versão 1.0. As fases são planejadas, podem ser reordenadas e devem refletir bloqueadores encontrados em testes.

## Fluxo recomendado para cada issue

`Backlog → Ready → In Progress → Review → QA → Done`

- **Ready:** escopo, dependências e cenário de teste entendidos.
- **In Progress:** mudança isolada em branch e PR identificado.
- **Review:** critérios funcionais, segurança e contratos revisados.
- **QA:** testes unitários, integração, E2E e matriz por SO/rede conforme escopo.
- **Done:** evidência reproduzível, CI verde, documentação e changelog atualizados.

Não encerrar issues por decisão informal, mock ou implantação não testada. Bloqueadores devem continuar visíveis.
