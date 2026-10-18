# Seven Desk — Licenciamento

**Licença para o código original do Seven Desk:** GNU Affero General Public License, versão 3.0, **somente** (identificador SPDX: `AGPL-3.0-only`). O texto normativo integral está em [../LICENSE](../LICENSE).

Copyright (C) 2026 gabriell211 e colaboradores do Seven Desk, quanto às respectivas contribuições originais.

## O que isso permite

O código original licenciado pode ser executado, estudado, modificado, distribuído e utilizado comercialmente, desde que as condições da AGPLv3 sejam respeitadas. A licença é de software livre; não implica contratação de hospedagem, licença adicional do servidor ou serviços pagos.

## Obrigações importantes

- Distribuições de executáveis e cópias modificadas devem preservar avisos, licença e as informações/forma de obter o **código-fonte correspondente**, incluindo scripts relevantes de compilação e instalação, conforme a AGPLv3.
- Conforme a seção 13 da AGPLv3, versões modificadas que permitam interação de usuários pela rede devem oferecer a esses usuários uma oportunidade claramente acessível de receber o código-fonte correspondente da versão efetivamente executada. O link para a branch `main` não substitui necessariamente o código da versão implantada.
- Usuários que recebam alterações AGPL devem conservar os direitos concedidos pela licença. A AGPL não exige publicar chaves privadas, credenciais ou dados pessoais.
- O projeto está em **pré-alfa**. Licença livre **não** equivale a software auditado, seguro ou homologado para acesso remoto pela Internet.

## Terceiros, dependências e identidade visual

- Este licenciamento **não altera** os termos de projetos externos usados como dependências (por exemplo, Tauri, React e crates Rust). Conserve os respectivos avisos e licenças e faça a revisão de compatibilidade da árvore de dependências antes da distribuição.
- Não há integração atual do protocolo ou do código do servidor RustDesk. Se houver reaproveitamento no futuro, a importação deverá preservar as licenças, avisos de copyright, origem dos arquivos e cumprir os termos aplicáveis.
- Marcas, nomes de terceiros, obras e arquivos para os quais o projeto não detém direitos não são automaticamente relicenciados. Conferir a procedência dos ativos e as regras de marca na issue [#67](https://github.com/gabriell211/seven-desk/issues/67).

## Checklist para release

1. Confirmar autoria e permissões de todo código/ativo incorporado.
2. Gerar inventário de dependências e avisos de licença por plataforma.
3. Distribuir o texto completo da AGPL-3.0 e os avisos legalmente obrigatórios.
4. Fornecer acesso ao código-fonte correspondente exato da versão distribuída, incluindo modificações próprias; para versões de rede modificadas, atender à seção 13.
5. Revisar instaladores, atualizador, telas **Sobre/Licença** e meios de acesso ao código-fonte.

Este documento resume obrigações práticas; o texto [LICENSE](../LICENSE) prevalece.
