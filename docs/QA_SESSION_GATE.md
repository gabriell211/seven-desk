# Seven Desk — Gate de integração de sessão assinada

**Escopo:** teste automatizado de sinalização e autorização contra o executável real do servidor Rust/Axum, usando conexões HTTP apenas em loopback e identidades Ed25519 efêmeras. Este gate **não é homologação de acesso remoto entre dois computadores**.

## Execução

```bash
npm run smoke:server
npm run smoke:session -- --skip-build
```

Sem `--skip-build`, o teste compila o servidor antes de iniciar. O runner inicia um processo nativo isolado em porta temporária, cria identidades fictícias sem persisti-las, executa os casos e encerra o processo ao finalizar. Nenhuma tela real, tecla do sistema ou dado de cliente é capturado.

## Critérios verificáveis

| Caso | Resultado exigido |
| --- | --- |
| Registro e presença | Chave pública determina o ID; sessão para host não registrado é recusada |
| Assinaturas | Corpo alterado é recusado e nonce reutilizado não é aceito |
| Convites | Só o host destinatário vê o convite; terceiros não recebem a fila |
| Consentimento | Antes do aceite, mídia/entrada negadas; operador/terceiro não aprovam |
| Escopo | Host autoriza explicitamente; teclado não é liberado pela permissão de mouse |
| Quadro | Host publica PNG sintético e apenas operador autorizado consegue ler |
| Mouse | Fila de eventos assinados entrega uma vez e depois esvazia |
| Pausa | Host interrompe mídia/entrada e remove pixels previamente armazenados |
| Fim | Terceiro não encerra; revogação encerra entrega e remove autorização ativa |

O teste falha com código de saída não zero se qualquer etapa for recusada indevidamente ou uma ação proibida passar. A CI executa em **Linux e Windows** e publica artefatos JSON `session-api-linux.json` e `session-api-win32.json` para evidência da execução específica.

## Evidência e limitações

O JSON registra SHA da execução em CI, data, SO, cenários e resultados, mas **não contém credenciais, capturas ou IDs pessoais**. Um `pass` confirma somente esses contratos HTTP com o servidor real.

**Não comprovado por este gate:** sessão Tauri em duas máquinas, captura gráfica real, injeção nativa de mouse/teclado, serviço Windows, E2EE, relay/P2P, NAT/CGNAT, performance, instalador em usuário final, assinatura e atualização, pen-test, Linux/Wayland, macOS e dispositivos móveis.

Para [#14](https://github.com/gabriell211/seven-desk/issues/14), [#37](https://github.com/gabriell211/seven-desk/issues/37) e [#38](https://github.com/gabriell211/seven-desk/issues/38), evidências reais adicionais são obrigatórias. Nenhum gate de produção fica aprovado apenas por este smoke. O arquivo [TESTE_MULTIMAQUINA.md](TESTE_MULTIMAQUINA.md) descreve o teste presencial de duas instalações e a coleta de evidências.
