# Seven Desk — Estratégia de testes e critérios de release

> Documento normativo para iniciar implementação. **Nenhum gate está atualmente aprovado.** Aprovação exige evidência real, não checklists preenchidos sem executar teste.

## Classes de testes

| Tipo | O que validar | Quando |
| --- | --- | --- |
| Unitários | FSM de sessão, permissões, validação de contratos, codecs auxiliares | Cada PR |
| Contrato | Rust ↔ TypeScript, cliente ↔ servidor, versões N/N-1 | Cada alteração de protocolo |
| Integração | PostgreSQL, identidade, sinalização, relay, arquivos, IPC nativo | Cada PR de backend/core |
| Segurança | Replay, MITM, ACL, tenant isolation, bypass de consentimento, path traversal, fuzzing | Cada PR sensível + release |
| E2E | Duas máquinas reais, sessões autorizadas, captura/controle, desconexão, instaladores | MVP e por release |
| Compatibilidade | Windows, Linux X11/Wayland, macOS, CPU e versões anunciadas | A cada release |
| Performance | FPS, RTT, input-to-photon, CPU/GPU, banda, p50/p95 sob LAN/WAN | Cada mudança de mídia |
| Resiliência | Perda de rede, CGNAT, failover relay, reinício, update, disco cheio | Beta e 1.0 |
| Acessibilidade | Teclado, foco, contraste, etiquetas, avisos de consentimento | Cada entrega UI |

## Gate P0 — Pronto para implementação integrada

- [ ] #1 estrutura, contratos e toolchain rodando em ambiente limpo.
- [ ] #2 ameaça/privacidade e fluxos consentimento/negativa aprovados.
- [ ] #3 mensagens versionadas com limites e testes de entradas inválidas.
- [ ] #4 pipeline de build, lint, testes e auditoria em PR.
- [ ] #6 ID/chave de dispositivo não dependem de IP.
- [ ] #9 decisão de transporte fundamentada em PoC LAN/WAN.
- [ ] #49 host/UI/IPC separados, com boundary de privilégio.
- [ ] #50 handshake autenticado e regras de revogação.
- [ ] #51 seleção de codec/captura sustentada por benchmark/licença.
- [ ] #52 desenvolvimento local sem Docker obrigatório.
- [ ] #67 licença do código e inventário de dependências compatíveis.

## Gate P1 — MVP Windows assistido

- [ ] [#14](https://github.com/gabriell211/seven-desk/issues/14) Windows↔Windows em **duas máquinas físicas ou VMs com interfaces gráficas reais**, sem simular vídeo e input.
- [ ] Host identifica claramente quem pede suporte; rejeitar ou expirar não cria sessão.
- [ ] Vídeo só flui depois de aceite, entrada respeita escopo e revogação corta canais.
- [ ] Testar em LAN e WAN controlada através de relay funcional, mais falha de rede e baixa banda.
- [ ] O host possui indicação visível de sessão ativa e botão de encerrar.
- [ ] Instalador `SevenDesk-Setup-<versão>.exe` instalado/desinstalado numa máquina sem toolchain.
- [ ] Testes negativos: ID falso, assinatura inválida, token expirado, usuário sem permissão, duplicate session.
- [ ] Evidência: commit, build, SO e hardware, setup de rede, passos, saída de testes e vídeo demonstrativo com dados fictícios se necessário.

## Gate P2 — Beta desktop

- [ ] Linux/macos homologados em versões concretas com limitações públicas, ou marcados explicitamente como experimentais.
- [ ] Arquivos, clipboard, chat, gerenciador, proxy, descoberta e reconexão possuem testes de autorização.
- [ ] Pacotes Windows/Linux/macOS são assinados ou têm política documentada de assinatura pré-beta, conforme SO.
- [ ] LAN/WAN, CGNAT, proxy, relay bloqueado e IPv6 submetidos a cenários de conectividade.
- [ ] Mudança de monitor, teclado regional, tela bloqueada e permissões nativas testadas onde aplicáveis.
- [ ] Nenhum blocker crítico ou violação de permissão conhecido.

## Gate P3 — Operação profissional

- [ ] Acesso não assistido só entra por configuração voluntária e pode ser desabilitado/revogado.
- [ ] MFA, perfis, RBAC local e auditoria são verificados no host e control plane.
- [ ] Tunelamento e terminal, se liberados, têm políticas, limites, auditoria e testes de abuso.
- [ ] Revogar conta/tenant/dispositivo impede acesso independentemente do estado de UI.
- [ ] Banco isolado por tenant, retenção aplicável e trilha operacional sem conteúdo privado.

## Gate P4 — Release 1.0

- [ ] Checklist das issues #37 e #38, revisão de segurança e SBOM.
- [ ] Matriz de compatibilidade #72 confirma cada plataforma anunciada.
- [ ] Capacidade e custo real do relay #68 dimensionados e acompanhados de alertas.
- [ ] Backup, migrações e restauração #69 testados e documentados.
- [ ] Instaladores Seven Desk oficiais publicados com versão, hash, changelog e assinatura/verificação adequada.
- [ ] Política de atualização/reversão testada, nenhuma vulnerabilidade crítica/alta sem tratamento formal.
- [ ] Funcionalidades P5 listadas como futuras, nunca anunciadas como já disponíveis.
- [ ] Documentação de suporte e erros, política de versões e limitações conhecida publicadas.

## Resultados obrigatórios por execução

Para cada execução E2E capturar: `commit SHA`, build ID, OS+versão, CPU/GPU, codec, topologia da rede, configuração de relay, p50/p95 quando mensurável, passos, resultado (pass/fail/blocked), logs **sanitizados**, issue vinculada e responsável pela validação. Definir metas quantitativas em ADR após os primeiros benchmarks; **não inventar métricas de desempenho**.

## Matriz negativa mínima

- Conexão com ID correto e chave do par incorreta ⇒ **nega**.
- Pedido expirado, negado ou revogado ⇒ **não transmite mídia/controle**.
- Sem permissão de teclado, arquivo ou clipboard ⇒ **nenhuma ação** nesse canal.
- Serviço iniciado sem usuário ou app aberta, mas sem autorização de sessão ⇒ **nenhum controle**.
- Alterar usuário/tenant/alias por payload ⇒ **falha autorizativa**, não acesso cruzado.
- Perder relay/rede e restabelecer conexão ⇒ **renegociar e revalidar** permissões.
- Atualização não assinada/corrompida ⇒ **recusa**.
- Proxy, firewall ou SO sem API de controle ⇒ **erro específico**, nunca bypass.
