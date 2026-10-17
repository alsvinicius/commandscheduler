# Contexto do repositório

- Para criar ou gerenciar schedules como agente, integrar clientes pela API ou responder dúvidas de uso do sistema, consulte `.agents/skills/command-scheduler/SKILL.md`. A skill usa `docs/diataxis/` como fonte de documentação de uso.
- Quando uma mudança alterar de forma relevante o comportamento, a API, a configuração ou os fluxos de uso, atualize as páginas afetadas em `docs/diataxis/` e mantenha `.agents/skills/command-scheduler/SKILL.md` alinhada. Atualize também `docs/engineering/api/openapi.yaml` quando mudar o contrato da API. Faça essa propagação na mesma tarefa da mudança; preserve a separação entre documentação de uso Diátaxis e artefatos de PRD/planejamento.
- Este repositório contém a documentação e a implementação do Command Scheduler.
- O escopo registrado atualmente é uma plataforma para gerenciar agendamentos de comandos de terminal, com criação e gerenciamento via terminal (CLI), criação por aplicações externas, schedules e parâmetros preparados pelas aplicações externas antes da execução.
- Artefatos de produto ficam em `docs/product/`; documentação de engenharia e planejamento usam `docs/engineering/` e `docs/planning/` quando esses passos forem solicitados.
- Stack escolhida para o MVP: Rust com Axum, Tokio e SQLx/SQLite; interface Vue 3 + TypeScript + Vite. Ver `docs/engineering/technical-definition.md`.
- Há um scaffold do projeto único: daemon e CLI Rust na raiz, interface Vue em `ui/`, servida pelo daemon na mesma origem. Os comandos conhecidos estão no `README.md` e `CONTRIBUTING.md`; não presumir comandos fora deles.
- Durante a implementação, validações podem rodar apenas em Linux, conforme orientação do usuário em 2026-10-13; não bloquear tarefas aguardando hosts Windows/macOS. Isso não altera os critérios de release: compatibilidade e cenários BDD em CI nos três sistemas continuam obrigatórios no plano.
- Ao mudar requisitos, propagar para artefatos dependentes ou marcá-los como `needs-review`.
