# Seven Desk — Validação automatizada de instaladores

**Status:** esta etapa valida o pacote desktop, não o sistema de acesso remoto funcional ou uma versão de produção.

## Artefatos gerados no GitHub Actions

O workflow [Seven Desk native installer QA](../.github/workflows/installers.yml) roda em Windows, macOS e Linux quando a implementação do instalador muda na branch `main` ou quando acionado manualmente. Usa Tauri v2 e gera o ícone Seven Desk deterministicamente antes de empacotar.

| Sistema | Bundle de avaliação | Observação |
| --- | --- | --- |
| Windows | NSIS `.exe`, MSI `.msi` | Não assinado para distribuição pública |
| Linux | `.deb`, `.rpm` | Compatibilidade depende da distribuição |
| macOS | `.dmg` | Ad-hoc signing; sem notarização oficial |

1. Abra **Actions** no repositório.
2. Selecione **Seven Desk native installer QA**.
3. Escolha uma execução **verde**.
4. Na seção **Artifacts**, consulte o pacote correspondente ao seu sistema.
5. Faça testes somente em ambiente de QA controlado, sem credenciais reais ou usuários finais.

## Critérios para passar de QA a release

- [ ] Assinatura oficial verificada para Windows e macOS; notarização Apple conforme distribuição.
- [ ] Tauri instala, atualiza e desinstala corretamente em máquina limpa.
- [ ] Serviço do host, captura, controle, sinalização e relay testados entre **duas máquinas reais**.
- [ ] Consentimento, revogação, autenticação de pares e logs atendem a matriz de segurança.
- [ ] Versão, changelog, checksums e compatibilidade por SO/documentação publicados.
- [ ] Licença e direitos das dependências e ícones revisados.
- [ ] Gate da [issue #38](https://github.com/gabriell211/seven-desk/issues/38) aprovado.

**Nunca publicar os artefatos de QA como versão final do Seven Desk.**
