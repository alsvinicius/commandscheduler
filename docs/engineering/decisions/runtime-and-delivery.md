# Decisões técnicas — Runtime e entrega

- Estado: ready
- Revisão: 3
- Histórico de revisões: [registro central](../../revision-history.md) — seção `engineering/decisions/runtime-and-delivery.md`
- Atualizado: 2026-10-13
- Fontes e revisões: [PRD r17](../../product/prd.md), [requisitos r1](../../product/requirements.md), [jornadas r28](../../product/user-journeys.md)
- Artefatos relacionados: [definição técnica](../technical-definition.md), [estado do workflow](../../product/workflow-state.md)
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.

### DEC-001 — Coordenar capacidades locais em um runtime

- Estado: selecionada para MVP (decisão de planejamento)
- Fontes: REQ-001, REQ-002; JRN-001, JRN-002
- Decisão e alternativas: um daemon local Rust coordena scheduler, API Axum, UI Vue servida na mesma origem, armazenamento SQLite e executor de processos. Programas agendados são processos filhos. Componentes operacionais separados não fazem parte do MVP.
- Justificativa e consequências: reduz instalação/diagnóstico e fornece um único serviço supervisionado, coerente com execução local, interface em localhost e início automático. Falhas do daemon afetam todos os recursos; persistência e recuperação são requisitos essenciais.
- Evidência: requisitos locais do PRD r16 e fechamento arquitetural durante planejamento para não deixar decisão pendente antes da implementação, 2026-10-13.
- Implementação no MVP e extensão futura: um processo daemon coordena as capacidades; execução remota/distribuída e serviços separados permanecem fora do MVP.
### DEC-003 — Executar como serviço/daemon do sistema

- Estado: confirmada
- Fontes: REQ-001, REQ-002; JRN-001, JRN-002
- Decisão e alternativas: scheduler inicia como serviço/daemon do sistema automaticamente; alternativa de processo manual ou aplicação de bandeja não escolhida.
- Justificativa e consequências: usuário confirmou que schedules devem funcionar sem manter aberta a interface web. Exige comportamento de início, parada, atualização e diagnóstico apropriado ao sistema operacional.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-12.
- Implementação no MVP e extensão futura: serviço local no MVP; execução em host remoto não confirmada.
### DEC-004 — Suportar Windows, macOS e Linux

- Estado: confirmada
- Fontes: REQ-001, REQ-002; JRN-001, JRN-002
- Decisão e alternativas: o MVP deve suportar os três sistemas operacionais; suporte a apenas uma plataforma não foi escolhido.
- Justificativa e consequências: confirmado pelo usuário; implementação e validação precisam cobrir diferenças de serviços, execução de processos e empacotamento.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-12.
- Implementação no MVP e extensão futura: Windows, macOS e Linux no MVP; versões mínimas ainda não definidas.
### DEC-005 — Usar Rust como linguagem principal

- Estado: preferência confirmada
- Fontes: requisito técnico derivado de REQ-001 a REQ-006; JRN-001 a JRN-012
- Decisão e alternativas: Rust é a linguagem preferida pelo usuário; JavaScript e Python foram consideradas, mas não escolhidas como linguagem principal.
- Justificativa e consequências: preferência explícita do usuário; maturidade da equipe e tecnologias auxiliares ainda precisam ser avaliadas antes de fixar framework e divisão frontend/backend.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-12.
- Implementação no MVP e extensão futura: Rust é a linguagem do daemon e do cliente CLI; UI em Vue 3 + TypeScript + Vite; componentes definidos em DEC-015 e DEC-025.
### DEC-006 — Abrir a interface web no navegador local

- Estado: confirmada
- Fontes: REQ-002; JRN-001, JRN-004, JRN-005
- Decisão e alternativas: interface será acessada em navegador por localhost; aplicativo desktop wrapper não foi escolhido.
- Justificativa e consequências: decisão explícita; daemon local deve servir a interface sem depender de browser aberto para executar schedules.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-12.
- Implementação no MVP e extensão futura: interface web local no MVP; distribuição como aplicativo desktop pode ser avaliada futuramente.
### DEC-012 — Instalar o daemon manualmente

- Estado: confirmada
- Fontes: REQ-001, REQ-002; JRN-001, JRN-002
- Decisão e alternativas: instalação manual por sistema operacional no MVP; instalador unificado que configura o serviço automaticamente não escolhido.
- Justificativa e consequências: decisão explícita do usuário. Documentação deve cobrir os passos necessários para o daemon iniciar automaticamente em Windows, macOS e Linux.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-13.
- Implementação no MVP e extensão futura: instruções manuais por SO no MVP; automatização da instalação pode ser avaliada posteriormente.
### DEC-015 — Usar Vue 3 + TypeScript + Vite para a interface

- Estado: confirmada
- Fontes: REQ-002; JRN-001, JRN-004, JRN-005, JRN-008, JRN-009
- Decisão e alternativas: Vue 3 com TypeScript e Vite para implementar a SPA local. React e Svelte foram comparados como alternativas e não escolhidos.
- Justificativa e consequências: a documentação oficial do Vue descreve adoção incremental, cobre SPA e oferece tutorial baseado em HTML/JavaScript; é uma opção direta para um painel local e para aprendizado assistido por IA. React recomenda usar um framework para novos apps; Svelte compila componentes e se apresenta como uma opção de footprint reduzido. A recomendação foi confirmada pelo usuário.
- Evidência: [Vue introduction](https://vuejs.org/guide/introduction.html), [Vue quick start](https://vuejs.org/guide/quick-start), [React installation](https://react.dev/learn/installation), [Svelte overview](https://svelte.dev/docs/svelte/overview).
- Implementação no MVP e extensão futura: Vue 3 + TypeScript + Vite para UI SPA servida pelo daemon local.
### DEC-016 — Fornecer instruções de compilação e instalação manual

- Estado: confirmada
- Fontes: REQ-001, REQ-002; JRN-001, JRN-002; decisão de instalação manual em DEC-012
- Decisão e alternativas: o repositório fornece instruções reproduzíveis para compilar e instalar o daemon manualmente em Windows, macOS e Linux. Binários prontos e instalador automático não são requisitos do MVP.
- Justificativa e consequências: permite que o usuário instale e opere o serviço nos três sistemas sem introduzir distribuição automatizada. Os passos precisam refletir comandos reais de build e registro do serviço e ser verificados por plataforma.
- Evidência: resposta do usuário sobre instalação manual e suporte multiplataforma, registrada na definição técnica r39.
- Implementação no MVP e extensão futura: instruções de compilação/instalação fazem parte de TASK-012; automação de instalação e distribuição de binários pode ser considerada depois.

### DEC-025 — Usar Axum + Tokio + SQLx para o runtime local

- Estado: confirmada
- Fontes: REQ-001–REQ-006; JRN-001–JRN-012
- Decisão e alternativas: Axum para HTTP/API, Tokio como runtime assíncrono e SQLx com driver SQLite para acesso persistente. Alternativa de acesso síncrono a SQLite com rusqlite não foi escolhida.
- Justificativa e consequências: Axum se integra ao ecossistema Tower e oferece roteamento/extractors; Tokio fornece I/O, tarefas e timers; SQLx oferece driver SQLite assíncrono. O recurso SQLite do SQLx inclui SQLite e o compila/estaticamente vincula, exigindo ferramentas C nos ambientes de build.
- Evidência: [Axum docs](https://docs.rs/axum/latest/axum/), [Tokio docs](https://docs.rs/tokio/latest/tokio/), [SQLx SQLite docs](https://docs.rs/sqlx/latest/sqlx/sqlite/).
- Implementação no MVP e extensão futura: Axum + Tokio + SQLx/SQLite no daemon/API Rust; versões das crates ficam para a implementação.
### DEC-036 — Criar e gerenciar schedules por CLI cliente da API local

- Estado: selecionada para MVP
- Fontes: REQ-001–REQ-002, REQ-004, REQ-006; JRN-013–JRN-015; escopo do repositório em `AGENTS.md` e resposta de produto de 2026-10-13.
- Decisão e alternativas: fornecer CLI companion para criar e gerenciar schedules e execuções chamando a API REST/JSON do daemon em localhost. A CLI não mantém lógica própria de validação, persistência ou scheduling; oferece operações equivalentes às de gerenciamento da API, incluindo consulta e manutenção de schedules, parâmetros, execuções e recuperação.
- Justificativa e consequências: atende ao requisito de operação por terminal mantendo uma fonte única de comportamento e compatibilidade com aplicações locais.
- Implementação no MVP e extensão futura: incluir comandos de criação, listagem/consulta/edição, pausa/retomada/cancelamento, execução imediata, histórico/resultado/saídas, cancelamento de ocorrência, retry e confirmação de retry interrompido; instalar e documentar nos três sistemas.

### DEC-038 — Guardar o banco no diretório local de dados do usuário

- Estado: selecionada para implementação na TASK-003.
- Fontes: DEC-009, DEC-025; REQ-001, REQ-003–REQ-006; TASK-003.
- Decisão e alternativas: SQLite usa o diretório local de dados por usuário encontrado por `ProjectDirs`; `COMMAND_SCHEDULER_DB_PATH` permite sobrescrever o arquivo. A aplicação cria diretório/arquivo e aplica migrações SQLx embutidas na inicialização. SQLite usa WAL, chaves estrangeiras e espera até 5 s por lock; tempos são guardados como microssegundos Unix e convertidos para RFC 3339 na API. Expurgo remove ocorrências finalizadas após 90 dias sem remover schedules.
- Justificativa e consequências: mantém dados no perfil do mesmo usuário que executa comandos e segue os diretórios locais padrão de Linux/XDG, macOS e Windows; o override permite instalação/teste controlados. WAL favorece leituras concorrentes com um escritor e mantém banco autocontido; a inicialização deve reportar falha clara se diretório, migration ou lock não puder ser usado.
- Evidência técnica: `ProjectDirs` documenta diretório de dados por aplicação nos três sistemas; SQLx documenta migrações embutidas e opções SQLite para WAL, chaves estrangeiras, timeout e criação de arquivo.
- Implementação no MVP e extensão futura: migrações versionadas, retenção diária e caminho configurável na CLI/daemon; backup/restore permanece fora do MVP.
