# Contribuindo com Seven Desk

O desenvolvimento deverá preservar segurança, reprodutibilidade, acessibilidade e rastreabilidade. O repositório está em **pré-alfa**, com código executável e testes automatizados: não atribua a funcionalidades planejadas status de implementação nem confunda CI verde com homologação em dois computadores.

## Antes de escrever código

1. Escolha uma issue com dependências resolvidas; as fases estão em [docs/ROADMAP.md](docs/ROADMAP.md).
2. Confira [arquitetura](docs/ARQUITETURA.md), [matriz funcional](docs/MATRIZ_FUNCIONAL.md), [segurança](SECURITY.md) e [homologação](docs/HOMOLOGACAO.md).
3. Para transporte, autenticação, serviço privilegiado, criptografia ou banco de dados, documente primeiro a ADR e o threat model.
4. Não importe código ou mídia de terceiros sem autorização de licença e inventário em #67.

## Pull requests

- Branch curta, objetivo único, commits `feat:`, `fix:`, `docs:`, `test:`, `security:` etc.
- Referencie issue, descreva decisões, riscos, comportamento antes/depois, plataformas testadas, exceções e procedimentos de rollback.
- Cobrir caminhos felizes, nulos/vazios/erros, autorização negada e revogação. Não confiar apenas em testes com mocks.
- Rust com fmt/clippy/test; TypeScript em strict e testes de UI. Os workflows `Check`, `Seven Desk CI` e `native installer QA` já existem; auditoria de dependências, branch protection e cobertura ampliada permanecem na issue #4.
- Documentação e contratos devem evoluir junto ao código.

## Definition of Done

- [ ] Código real e sem caminhos mockados em produção
- [ ] Sem credenciais e tokens em código, prints ou logs
- [ ] Autorização validada no host e backend quando aplicável
- [ ] Unitários, contrato, integração, E2E e negativas adequadas ao escopo
- [ ] Evidências por SO/versão, build SHA e cenário reproduzível
- [ ] Segurança, licença e documentação revisadas
- [ ] CI configurado e verde, quando disponível
- [ ] Nenhum recurso documentado como pronto antes de homologação

## Relatórios de falha

Use os modelos de issue. Para vulnerabilidades, siga [SECURITY.md](SECURITY.md) e canal privado, nunca uma issue pública.
