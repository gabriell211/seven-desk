# Política de segurança — Seven Desk

> **Fase atual:** planejamento. Nenhuma versão de produção, política de suporte de versões ou SLA de correção foi anunciada.

## Como relatar vulnerabilidades

**Não publique exploits, credenciais ou dados reais de sessão em issues ou discussões públicas.**

1. Verifique a aba **Security** do repositório para saber se a opção de **private vulnerability reporting** está habilitada.
2. Se estiver disponível, utilize esse canal privado com descrição do impacto, reprodução, pré-condições, versão/commit e mitigação sugerida.
3. Se não estiver, contate o mantenedor por um meio privado indicado no perfil do GitHub; **não publique detalhes de exploração até estabelecer contato seguro**.

O projeto ainda não possui canal de e-mail de segurança divulgado. Não se deve inferir SLA de resposta ou garantias de correção.

## Regras fundamentais

- Acesso assistido requer aceite visível do host e delimitação das permissões.
- Acesso não assistido requer ativação explícita, autenticação forte, revogação e auditoria.
- Nenhuma sessão pode ser iniciada apenas por conhecimento do ID público.
- Transferência, clipboard, microfone/áudio, reinicialização e gravação requerem permissões próprias.
- Cada ação sensível exige autorização no host; não confiar exclusivamente no operador ou control plane.
- Comunicação segura com verificação de identidade, chaves protegidas e proteção contra replay.
- Sessões devem apresentar indicador e mecanismo de encerramento acessíveis ao dispositivo de destino.
- Não capturar tela, senhas, arquivos ou clipboard em logs por padrão.
- Não criar recursos de sessão invisível, instalação enganosa ou bypass de permissões do sistema.
- Builds devem ser assinadas, verificáveis e distribuídas por canais reconhecidos.

## Área de auditoria prioritária

Identidade do dispositivo, autorização em tempo real, tokens e credenciais, serviço privilegiado, captura/entrada nativa, IPC Tauri↔Rust, relay, transferência de arquivos, atualização e isolamento multi-tenant.

## Privacidade

Tratar dados relacionados a dispositivos, sessões, logs e gravações segundo finalidade declarada, retenção mínima, controles por papel, revogação e requisitos legais aplicáveis. Qualquer gravação deve ter aviso explícito, retenção e possibilidade de exclusão conforme política.

Veja também [a issue de modelo de ameaças](https://github.com/gabriell211/seven-desk/issues/2) e [a auditoria pré-lançamento](https://github.com/gabriell211/seven-desk/issues/37).
