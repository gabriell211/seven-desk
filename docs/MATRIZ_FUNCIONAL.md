# Seven Desk — Matriz de requisitos e rastreabilidade

**Base:** levantamento funcional orientado a produto de suporte remoto profissional e revisão de escopo em 16/10/2026.

**Legenda:** a tabela registra o **escopo-alvo** e suas fases, não o estado executável de cada item. Consulte [IMPLEMENTACAO.md](IMPLEMENTACAO.md) para o estado comprovado; partes de tela, mouse e teclado existem em pré-alfa, mas a maioria das capacidades segue planejada. "1.0" indica a fase-alvo de backlog; **não equivale à garantia de entrega**. P5 significa fase posterior. Nenhuma funcionalidade de terceiros deve ser copiada literalmente em marca, texto ou identidade.

| Domínio | Capacidade | Issues | Fase-alvo |
| --- | --- | --- | --- |
| Identidade | ID exclusivo, copiar, alias | #6, #65 | P0/P3 |
| Convite | Pedido de sessão, aceite, expiração, links | #7, #8, #13, #64 | P0-P2 |
| Interface | Início, recentes, favoritos, estado, bandeja | #5, #24, #53 | P0-P2 |
| Segurança | Consentimento, indicador, encerrar, revogar | #2, #8, #13, #50 | P0/P1 |
| Acesso | Controle assistido da área de trabalho | #10, #11, #12, #14 | P1 |
| Acesso | Acesso não assistido configurado explicitamente | #27, #28, #29 | P3 |
| Autenticação | Contas, sessões confiáveis, MFA | #28 | P3 |
| Controle | Mouse, teclado, atalhos, tela cheia e escala | #11, #12, #22, #60 | P1/P2 |
| Controle | Elevação legítima e ações Windows protegidas | #54, #59 | P1/P2 |
| Exibição | Seleção e vários monitores, DPI e hotplug | #10, #31 | P1/P3 |
| Qualidade | Codec, FPS, bitrate e adaptação | #12, #36, #51 | P0-P4 |
| Redes | P2P, autenticação, relay, NAT, CGNAT | #9, #19, #50 | P0/P2 |
| Redes | Proxy HTTP CONNECT/PAC e diagnóstico | #55, #58 | P2 |
| Redes | Descoberta voluntária na LAN | #57 | P2 |
| Redes | Tunelamento TCP por sessão e política | #61 | P3 |
| Redes | VPN virtual e rotas isoladas | #62 | P5 |
| Arquivos | Upload/download, progresso, integridade | #18 | P2 |
| Arquivos | Navegador de dois painéis, modo arquivo | #56 | P2 |
| Clipboard | Texto bidirecional com autorização | #17 | P2 |
| Comunicação | Chat textual na sessão | #16 | P2 |
| Comunicação | Convites por link e fila simples | #64 | P2 |
| Colaboração | Várias sessões, fila avançada de técnicos | #45 | P5 |
| Colaboração | Anotações sobre tela e mais participantes | #44 | P5 |
| Operação | Reconexão e retomada de rede | #25 | P2 |
| Operação | Reinicialização remota autorizada | #30 | P3 |
| Operação | Terminal remoto opt-in | #63 | P3 |
| Operação | Wake-on-LAN | #40 | P5 |
| Operação | Impressão remota | #41 | P5 |
| Operação | Áudio remoto | #42 | P5 |
| Privacidade | Modo de privacidade e limites do SO | #46 | P5 |
| Gravação | Gravação consentida de sessão | #43 | P5 |
| Gravação | Captura/gravação local fora de sessão | #70 | P5 |
| Gestão | Contatos, grupos, pesquisa, favoritos | #24 | P2 |
| Gestão | ACL, perfis de permissão, allowlist de peers | #29, #65 | P3 |
| Gestão | Organizações, membros e parque de endpoints | #32 | P3 |
| Gestão | SSO/OIDC e papéis corporativos | #34 | P4 |
| Gestão | GPO/MDM e política distribuída | #66, #73 | P4/P5 |
| Gestão | Branding/cliente por organização | #48 | P5 |
| Gestão | API e webhooks autenticados | #35 | P4 |
| Gestão | Infraestrutura self-hosted | #47 | P5 |
| Localização | PT-BR, outros idiomas e acessibilidade | #5, #26 | P0/P2 |
| Suporte | Erros, logs sanitizados e export de diagnóstico | #58 | P2 |
| Instalação | Instalador Seven Desk Windows e serviço opcional | #15, #54 | P1 |
| Instalação | Instaladores/pacotes Linux/macOS e atualização assinada | #23 | P2 |
| Instalação | Deep links, CLI segura e automação autorizada | #71 | P5 |
| Mobile | Viewer móvel e restrições de host | #39, #73 | P5 |
| Qualidade | E2E reais, compatibilidade, benchmark e pentest | #14, #36, #37, #72 | P1-P4 |
| Produção | Logs, custos, SLO, upgrade/restore, release | #33, #38, #68, #69 | P3/P4 |
| Legal | Direitos, licença, dependências, avisos | #67 | P0 |

## Fora de escopo ou não garantido

- Execução invisível, controle de terceiros sem autorização, bypass de UAC/TCC/portal, instalação disfarçada.
- Controle irrestrito de dispositivos iOS/Android em violação às APIs oficiais; declarar limites por versão/plataforma.
- Codec universal, latência zero ou conexão P2P garantida em todas as redes.
- Reutilização de código sob licença incompatível sem decisão jurídica de distribuição.
- Recursos P5 não são condição automática para 1.0, a menos que o escopo de release seja alterado formalmente.

## Diferença entre cobertura do backlog e homologação

A matriz verifica que a função **possui uma issue responsável**. Ela não prova implementação, funcionalidade, paridade exata ou performance. A conclusão exige código, teste real, revisão de segurança e evidência ligada à issue.

Ao incluir novo requisito, apontar issue principal, dependências, SOs suportados, permissão, erros e teste E2E.
