# Seven Desk — Relatório de alinhamento para desenvolvimento

**Data da auditoria:** 16/10/2026 · **Escopo:** README, roadmap, política de segurança, templates e backlog.

## Estado verificado

- Repositório iniciado com **documentação e issues**, sem cliente, servidor, testes ou instaladores verificados.
- A criação das issues assegura rastreabilidade, mas não prova conclusão.
- Não há metas reais de FPS/latência medidas, infraestrutura implantada ou versões Windows/Linux/macOS homologadas.
- Nenhum PR havia sido identificado na primeira inspeção desta rodada.

## Lacunas tratadas nesta revisão

| Lacuna | Tratamento |
| --- | --- |
| Fronteira UI vs serviço nativo | #49 e documentação de arquitetura |
| Autenticação real do par e revisão de chaves | #50 |
| Seleção de codec e pipeline validável | #51 |
| Ambiente local Windows sem Docker obrigatório | #52 |
| Primeiro uso e status/bandeja | #53 |
| Serviço do Windows separado, permissões e updates | #54 |
| Proxy, LAN discovery e diagnóstico | #55, #57, #58 |
| File manager independente | #56 |
| UAC/operações Windows e controle de sessão | #59, #60 |
| Port forwarding, VPN e terminal | #61, #62, #63 |
| Convites via link/filas e aliases/ACL | #64, #65 |
| MDM/GPO e políticas distribuídas | #66, #73 |
| Licenças e ativos | #67 |
| Custos/escala, migrações/backup e QA cross-OS | #68, #69, #72 |
| Gravação fora da sessão e CLI/URL handler | #70, #71 |
| Dependências inconsistentes e sobreposição de escopo | Ajustadas #14, #38, #45, #66 |
| Mac: formato do bundle oficial | DMG Tauri como padrão; PKG opcional para assistente específico |

## Decisões bloqueantes — não confundir planejamento com aprovação

1. **Transporte:** WebRTC/ICE/TURN ou QUIC/relay após PoC e avaliação de autenticação (#9).
2. **Codecs/licenças:** pipeline selecionado e compatibilidade por SO (#51, #67).
3. **Host service:** IPC e política do processo privilegiado, gestão do ciclo de vida (#49, #54).
4. **Identidade/cripto:** protocolo de autenticação mútua, chaves, revogação, logs e threat model (#2, #50).
5. **Deploy/control plane:** custeio do relay, armazenagem, backups e SLOs (#7, #68, #69).
6. **Licença final:** licença do Seven Desk e compatibilidade com bibliotecas de terceiros (#67).
7. **Versão 1.0:** backlog necessário e recursos da fase P5 que permanecem futuros (#38).

## Critério de alinhamento documental

- Todo recurso está classificado como planejado ou implementado com evidências.
- Há responsável (issue) para cada funcionalidade do escopo.
- Dependências não têm ciclos e não demandam fase futura para encerrar fase anterior.
- Cenários de negação, revogação e operação normal aparecem em cada módulo de acesso.
- Instalador é um produto Seven Desk com identidade única em cada sistema.
- A arquitetura respeita APIs do SO e licenciamento de componentes.
- O README não apresenta instruções de build inventadas.
- O roadmap e as issues de gate são coerentes entre si.

## Próximo trabalho concreto

Começar pela **P0**: #1, #2, #3, #4, #9, #49, #50, #51, #52 e #67. Depois implementar MVP real Windows↔Windows e seu instalador. **Ainda não há homologação de produção.**

Ver [matriz funcional](MATRIZ_FUNCIONAL.md), [arquitetura](ARQUITETURA.md) e [homologação](HOMOLOGACAO.md).
