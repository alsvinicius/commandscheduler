# Tarefas de implementação — Base e runtime

- Status: ready
- Revisão: 38
- Histórico de revisões: [registro central](../../revision-history.md) — seção `planning/tasks/foundation.md`
- Atualizado: 2026-10-13
- Fontes e revisões: [PRD r17](../../product/prd.md), [requisitos r1](../../product/requirements.md), [índice de jornadas r28](../../product/user-journeys.md), [definição técnica r55](../../engineering/technical-definition.md), [plano r53](../implementation-plan.md)
- Execução: TASK-001–TASK-005 implementadas e validadas no Linux, incluindo BDDs de recorrência, recuperação, executor e retomada; TASK-006 validada no Linux. Reprodução nativa Windows/macOS e CI tripla continuam gates de release.
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.
- Artefatos relacionados: [estado do workflow](../../product/workflow-state.md), [índice de documentação](../../README.md)

#### TASK-001 — Inicializar a aplicação e toolchains no limite único do repositório

- Status de execução: concluída para a etapa de implementação.

- Descrição: criar a estrutura inicial de Rust com Axum/Tokio/SQLx/SQLite e Vue 3 + TypeScript + Vite, scripts e comandos de desenvolvimento/build, estratégia de assets servidos pelo daemon na mesma origem e configurações multiplataforma. Manter um limite de aplicação na raiz; não criar subprojetos backend/frontend separados sem necessidade real de build/deploy.
- Fontes: DEC-001, DEC-003–DEC-006, DEC-009, DEC-015, DEC-025, DEC-036; PRD r17; definição técnica r41.
- Dependências: nenhuma.
- Subtarefas: confirmar toolchains e feature flags; adicionar scaffolds Rust/Vue; configurar formatação/lint estáticos apropriados; documentar comandos conhecidos.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: Nenhum cenário de comportamento direto; tarefa habilitadora.
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: stack e arquitetura de daemon único coordenando scheduler, API, UI, SQLite e executor estão escolhidas em DEC-001; não há aprovação ou validação prévia. Selecionar versões compatíveis das dependências dentro da stack definida.
- Definição de concluído nesta etapa de implementação: daemon e cliente CLI compilam no Linux; UI é servida localmente pelo daemon na origem aprovada; root `README.md`, `CONTRIBUTING.md`, `AGENTS.md` e `.gitignore` refletem comandos e arquivos reais; nenhum arquivo ou comando de build é apenas presumido. Conforme orientação do usuário, validação só em Linux basta para avançar tarefas; compatibilidade nativa Windows/macOS e CI BDD tripla permanecem gates de release no [plano](../implementation-plan.md#regra-temporaria-de-validacao-durante-implementacao).
- Resultado esperado: um comando/processo executável de desenvolvimento e build local para cada toolchain identificada e estrutura de projeto única.
- Documentação afetada: `README.md`, `CONTRIBUTING.md`, `AGENTS.md`, `.gitignore`, `docs/engineering/technical-definition.md`.
- Evidências em 2026-10-13: `pnpm install --frozen-lockfile`, `pnpm build`, `pnpm ui:build`, `cargo build --locked --bins`, `cargo fmt --all -- --check` e `cargo clippy --locked --all-targets --all-features -- -D warnings` passaram no Linux. `cargo check --locked --bins --target x86_64-pc-windows-gnu` e `cargo check --locked --bins --target x86_64-apple-darwin` passaram com Zig 0.17 como compilador C cruzado temporário. O daemon local respondeu `ok` em `/health` e serviu o `ui/dist/index.html` em `/`. A toolchain Rust 1.99.0 e Zig foram instalados temporariamente sob `/tmp`. Sem cenários BDD diretos nesta tarefa habilitadora.
- Limite/gate futuro: os `cargo check` cruzados não linkam executáveis; builds Windows GNU/GNU LLVM falharam sem bibliotecas MinGW. macOS não foi linkado nem executado. Essa evidência fica para o gate de release e não bloqueia TASK-002 nem as tarefas seguintes durante a implementação.
#### TASK-002 — Definir contrato OpenAPI e erros da API

- Status: concluída.
- Descrição: especificar API v1 em OpenAPI 3.1 incluindo schedules, parâmetros, ocorrências, controles, execução imediata, retry, idempotência, metadados consolidados, paginação, schemas, erros e exemplos. Publicar referência navegável localmente.
- Fontes: REQ-002–REQ-006; JRN-002–JRN-003, JRN-006–JRN-007, JRN-010–JRN-012; DEC-019, DEC-023–DEC-024, DEC-027–DEC-028, DEC-032, DEC-037.
- Dependências: TASK-001.
- Subtarefas: detalhar recursos e operações; definir erros/estados HTTP e paginação; incluir `Idempotency-Key`, `missedOccurrences` e convenções de parâmetros; montar referência navegável.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: BDD-JRN-002-*, BDD-JRN-003-*, BDD-JRN-006-*, BDD-JRN-007-*, BDD-JRN-010-*, BDD-JRN-011-*, BDD-JRN-012-*
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: decisões sobre REST/JSON, idempotência, JSON ≤64 KiB, program+args, metadados RFC 3339 e localhost disponíveis nos artefatos fonte.
- Definição de concluído: OpenAPI 3.1 versionado cobre todos os recursos aceitos e exemplos; validação automatizada do documento passa; referência local corresponde ao contrato publicado; documentação técnica atualizada.
- Resultado esperado: uma especificação única revisável que consumidores locais possam consultar antes da implementação de integrações.
- Documentação afetada: artefato OpenAPI versionado a ser criado; `docs/engineering/technical-definition.md`; jornadas se mudanças afetarem comportamento observável.
- Evidência de conclusão em 2026-10-13: OpenAPI canônico em `docs/engineering/api/openapi.yaml`; `pnpm openapi:lint`, `pnpm ui:build`, `cargo fmt --all -- --check`, `cargo build --locked --bins` e `cargo clippy --locked --all-targets --all-features -- -D warnings` passaram no Linux. O daemon respondeu 200 em `/docs/api`, `/docs/api/`, `/openapi.yaml` e nos assets; o YAML servido é idêntico ao documento versionado. Escolha de cron de cinco campos registrada em DEC-037, definição técnica r43 e plano r16.
#### TASK-003 — Implementar persistência SQLite, migrações e retenção

- Status de execução: concluída.

- Descrição: criar modelo durável para schedules, parâmetros, política de recuperação/retry, ocorrências, tentativas, resultados, idempotência e histórico; implementar migrações, transações e retenção de 90 dias para histórico e saída capturada.
- Fontes: REQ-001, REQ-003–REQ-006; JRN-001–JRN-012; DEC-007–DEC-014, DEC-024–DEC-025, DEC-031, DEC-038.
- Dependências: TASK-001, TASK-002.
- Subtarefas: definir tabelas/índices; migrações; atomicidade da criação idempotente; rotina de expurgo após 90 dias; estados após crash/restart.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: BDD-JRN-001-*, BDD-JRN-002-*, BDD-JRN-003-*, BDD-JRN-004-*, BDD-JRN-005-*, BDD-JRN-006-*, BDD-JRN-007-*, BDD-JRN-008-*, BDD-JRN-009-*, BDD-JRN-010-*, BDD-JRN-011-*, BDD-JRN-012-*
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: semântica de schedule/ocorrência e contrato da API definidos; SQLite local confirmado.
- Definição de concluído: dados sobrevivem ao reinício do daemon; migrações avançam versão do banco com segurança; chave idempotente permanece enquanto schedule existir; retenção expurga histórico e saídas após 90 dias sem apagar schedules ativos; documentação de dados atualizada.
- Resultado esperado: schedules e histórico consultáveis de forma consistente depois de reinício e migração.
- Documentação afetada: `docs/engineering/technical-definition.md`; referências de API/schema e guia operacional conforme necessário.
- Evidência de conclusão em 2026-10-13: `cargo fmt --all -- --check`, `cargo test --locked --bins` (4 passaram), `cargo clippy --locked --all-targets --all-features -- -D warnings` e `cargo build --locked --bins` passaram em Linux. Smoke do daemon com `COMMAND_SCHEDULER_DB_PATH` respondeu `ok`, aplicou migração SQLx v1, criou `schedules`/`occurrences` e abriu SQLite em WAL. Persistência após fechar/reabrir pool, replay idempotente/conflito, JSON livre até 64 KiB, rejeição sem sobrescrever parâmetros e retenção após 90 dias estão testados.

#### TASK-004 — Implementar cálculo de recorrência e recuperação de ocorrências vencidas

- Status de execução: implementação e BDDs validados no Linux; CI tripla permanece no gate de release.

- Descrição: calcular execuções únicas e recorrências de calendário flexível no fuso da máquina, regras DST, pausas/alterações e política por schedule para vencidas por reinício, suspensão ou atraso temporário.
- Fontes: REQ-001; JRN-001–JRN-002, JRN-005–JRN-006; DEC-007–DEC-008, DEC-018, DEC-029, DEC-037.
- Dependências: TASK-003.
- Subtarefas: parser/modelo de recorrência; pular horário local inexistente e deduplicar repetido; pular/consolidar/sequenciar vencidas; limitar a 100 individuais e consolidar excedente; incluir count/from/through RFC 3339 e parâmetros mais recentes.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: BDD-JRN-001-*, BDD-JRN-002-*, BDD-JRN-005-*, BDD-JRN-006-*
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: fuso da máquina e semântica DST/recuperação confirmados; sintaxe cron padrão de cinco campos selecionada em DEC-037 por concisão, interoperabilidade e validação idêntica pelo parser em todos os sistemas; exemplos estão no OpenAPI. Validação de desenvolvimento pode rodar apenas em Linux conforme orientação do usuário; CI tripla é gate de release.
- Definição de concluído: ocorrências geradas e recuperadas conforme política salva; limites e metadados de consolidação corretos; cenários BDD relacionados passam; comportamento de pausa/cancelamento é persistido e explicado.
- Resultado esperado: cada horário elegível gera comportamento reproduzível segundo configuração do schedule, inclusive após período indisponível.
- Documentação afetada: OpenAPI; `docs/engineering/technical-definition.md`; jornadas/BDD se sintaxe ou resposta observável mudar.
- Implementação inicial em Linux: parser cron estrito de cinco campos, conversão civil pelo fuso local com regras DST DEC-018, recuperação skip/consolidate/sequence com teto de 100 individuais, consolidação do excedente e cursor de avaliação persistido por migração v2. `cargo test --locked --bins` passou (22 testes); `cargo fmt --all`, `cargo clippy --locked --all-targets --all-features -- -D warnings` e `cargo build --locked --bins` passaram em Linux. A retomada aplica a política salva aos horários vencidos durante a pausa, conforme DEC-039; validação BDD segue na TASK-011.

#### TASK-005 — Implementar executor de processos, contexto, saídas e retries

- Status de execução: executor integrado com API/CLI e BDDs relacionados validados no Linux; reprodução nativa multiplataforma permanece no gate de release.

- Descrição: iniciar executável diretamente com lista `args`, ambiente construído com parâmetros JSON e metadados consolidados, sem shell implícito; controlar cancelamento, estados, saídas e retries.
- Fontes: REQ-001, REQ-004, REQ-006; JRN-001–JRN-003, JRN-007–JRN-012; DEC-011, DEC-017, DEC-019, DEC-023, DEC-026, DEC-030–DEC-031.
- Dependências: TASK-003, TASK-004.
- Subtarefas: construir ambiente com `COMMAND_SCHEDULER_PARAMS_JSON` e variáveis `COMMAND_SCHEDULER_MISSED_*`; capturar stdout/stderr em buffers limitados a 1 MiB por stream; indicar truncamento; aplicar três retries padrão após +1/+5/+15 min configuráveis; classificar falha normal vs interrompida; exigir confirmação manual após interrupção; permitir concorrência sem cota global.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: BDD-JRN-001-*, BDD-JRN-002-*, BDD-JRN-003-*, BDD-JRN-007-*, BDD-JRN-008-*, BDD-JRN-009-*, BDD-JRN-010-*, BDD-JRN-011-*, BDD-JRN-012-*
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: formato `program`+`args`, payload JSON e metadados estão especificados; estados de cancelamento e retry são observáveis pela API.
- Definição de concluído: processo executa sem shell implícito; argumentos e env são entregues sem interpolação; cada stream é limitado/truncado com marcador; código de saída, estado e tempos persistidos; retries respeitam política; encerramento do serviço marca processos interrompidos para confirmação manual; documentação/BDD atualizados.
- Resultado esperado: uma execução oferece estado, resultado e saída consultáveis, obedecendo as políticas definidas.
- Documentação afetada: `docs/engineering/technical-definition.md`, OpenAPI, guia para consumidores de comandos e jornadas relevantes.
#### TASK-006 — Construir scheduler daemon e ciclo de vida multiplataforma

- Status de execução: SIGTERM/SIGINT gracioso e unit systemd Linux implementados; instalação/ativação pelo manager systemd, health, UI, OpenAPI, API, persistência e recuperação supervisionada após SIGKILL foram reproduzidos no Linux pelo `scripts/verify_linux_supervisor.py`; templates LaunchAgent e instalação por Tarefa Agendada no Windows estão documentados; o instalador Windows para a tarefa existente e aguarda seu encerramento antes da atualização; reprodução nativa dos serviços macOS/Windows segue no gate de release.

- Descrição: manter timers e processamento de ocorrência no daemon independentemente do navegador; iniciar automaticamente via serviço do sistema; implementar sinalização/encerramento, recuperação de schedule e reinício supervisionado.
- Fontes: REQ-001; JRN-001–JRN-002, JRN-004–JRN-006; DEC-003–DEC-004, DEC-007, DEC-011–DEC-012, DEC-029, DEC-035.
- Dependências: TASK-003–TASK-005.
- Subtarefas: implementar loop de scheduling; integração de serviço Windows, LaunchAgent/daemon apropriado em macOS e systemd ou opção suportada em Linux; reinício pelo supervisor; varredura de vencidas no start/resume; garantir acesso aos arquivos locais do usuário.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: BDD-JRN-001-*, BDD-JRN-002-*, BDD-JRN-004-*, BDD-JRN-005-*, BDD-JRN-006-*
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: sistemas operacionais mínimos a publicar são definidos conforme versões com suporte de segurança do fornecedor no lançamento; regras de recuperação e estados de processo fechadas.
- Definição de concluído: daemon executa com browser fechado; instalação manual e início automático documentados em cada SO; reinício recupera schedules e aplica políticas; falha/recuperação é registrada e a medição de disponibilidade usa somente período ligado/apto; verificação por SO documentada.
- Evidência Linux (2026-10-13): instalação descrita no guia reproduzida com caminhos finais; o schedule SQLite permaneceu consultável após reiniciar o unit systemd. Unit, drop-in isolado, binário/assets e dados temporários foram limpos. CI e reprodução nativa macOS/Windows ainda são gates de release.
- Resultado esperado: serviço permanece ativo através de sessão web fechada e reinicia/recupera sem perder schedules persistidos.
- Documentação afetada: instruções operacionais por SO em `docs/`; README; `CONTRIBUTING.md`; `AGENTS.md`; definição técnica.
