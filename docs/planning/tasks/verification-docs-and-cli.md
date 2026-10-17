# Tarefas de implementação — Verificação, documentação e CLI

- Status: ready
- Revisão: 49
- Histórico de revisões: [registro central](../../revision-history.md) — seção `planning/tasks/verification-docs-and-cli.md`
- Atualizado: 2026-10-13
- Fontes e revisões: [PRD r19](../../product/prd.md), [requisitos r5](../../product/requirements.md), [índice de jornadas r34](../../product/user-journeys.md), [definição técnica r61](../../engineering/technical-definition.md), [plano r59](../implementation-plan.md)
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.
- Artefatos relacionados: [estado do workflow](../../product/workflow-state.md), [índice de documentação](../../README.md)

#### TASK-011 — Medir e verificar confiabilidade e pontualidade do MVP

- Status de execução: coletor Linux de disponibilidade, unit systemd independente e relatório mensal implementados e ativos; método mensal documentado em `docs/engineering/guides/availability.md`; doze verificações do relatório aprovadas, incluindo suspensão, ajuste de relógio, reinício do coletor, exclusão auditável de boot e proteção contra janela curta. A observação começou em 2026-10-13 15:31:55 UTC, `Linger=yes` foi confirmado e o serviço segue ativo; ainda não cobre mês completo e não permite declarar o alvo atingido. O medidor Linux executou duas vezes 100/100 anteriormente (P95/P99 0,252/0,268 s e 0,255/0,274 s); em 2026-10-13, uma janela prévia de 60 s registrou CPU entre 95,29% e 96,08% ociosa em cada intervalo de 10 s, seguida por nova medição 100/100 sem falhas (P95 0,2505 s; P99 0,269469 s; máximo 0,275043 s; load average ao final 1,018/0,986/1,416 em 12 CPUs). O gate Linux de pontualidade em host ocioso está satisfeito. Os 100 cenários BDD estão ligados a verificações executáveis, após a conclusão das verificações funcionais de TASK-016 e dos BDDs de JRN-001/JRN-016 em TASK-017/018. O medidor está configurado para a matriz CI Linux/macOS/Windows. Em 2026-10-13, rodada Linux passou 21 testes Rust do daemon + 2 da CLI, 46 Python, 15 Playwright, fmt, Clippy, lint OpenAPI, builds dos binários e da UI. `scripts/verify_linux_supervisor.py` comprovou novamente um restart após SIGKILL, PID alterado e schedule concluído (`succeeded`). A janela mensal completa e a reprodução nativa macOS/Windows e CI tripla continuam pendentes para release.

- Descrição: instrumentar disponibilidade local e executar verificações mensuráveis: início automático/recovery, disponibilidade alvo 99,5% no tempo ligado/apto, latência de início agendado e BDD multiplataforma. Não estabelecer limite de carga/volume ausente de decisão.
- Fontes: REQ-001–REQ-011; JRN-001–JRN-016; DEC-022, DEC-033–DEC-036, DEC-042.
- Dependências: TASK-004–TASK-010, TASK-013–TASK-018.
- Subtarefas: definir cálculo observável de disponibilidade e origem dos períodos aptos; medir atraso previsto/início em cenário ocioso; obter P95/P99; conferir reinício, serviço ativo com UI fechada, interrupção de processo, persistência e recuperação; mapear BDD de JRN-001–JRN-016 (incluindo cockpit, agenda e filtros) para CI nos três sistemas.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: BDD-JRN-001-*, BDD-JRN-002-*, BDD-JRN-003-*, BDD-JRN-004-*, BDD-JRN-005-*, BDD-JRN-006-*, BDD-JRN-007-*, BDD-JRN-008-*, BDD-JRN-009-*, BDD-JRN-010-*, BDD-JRN-011-*, BDD-JRN-012-*, BDD-JRN-013-*, BDD-JRN-014-*, BDD-JRN-015-*
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: implementação integrada; matriz de sistemas definida para versões com suporte de segurança na data do release.
- Definição de concluído: CI executa os cenários BDD rastreados às 16 jornadas, incluindo criação, manutenção e controle via CLI JRN-013–JRN-015 e cockpit/agenda/histórico JRN-016, em Windows, macOS e Linux; evidência de P95 ≤5 s/P99 ≤30 s em máquina ociosa; observação mensal de 99,5% completa e auditável conforme método e exclusões de desligamento/suspensão documentados; mecanismo de reinício/recovery exercitado; falhas registradas e sem alegação de SLO atingido sem período observado.
- Resultado esperado: comportamento multi-SO e limites de qualidade definidos são verificáveis e seus resultados/contexto ficam registrados.
- Evidência Linux (2026-10-13): `python3 scripts/measure_punctuality.py` terminou com 100/100 inícios, P95 0,252 s, P99 0,268 s; load average 0,48/1,87/2,01 no fechamento. Não classificada como evidência definitiva de máquina ociosa.
- Evidência Linux (2026-10-13): antes de `python3 scripts/measure_punctuality.py`, seis amostras de 10 s mostraram CPU ociosa entre 95,29% e 96,08%; a medição terminou com 100/100 inícios, zero falhas, P95 0,2505 s, P99 0,269469 s e máximo 0,275043 s. Load average no relatório: 1,018/0,986/1,416 em 12 CPUs.
- Evidência Linux (2026-10-13): `systemd-analyze --user verify` passou para ambas as units; serviço ativo e `/health` respondeu `ok`; observador systemd ativo e primeiro probe saudável gravado às 15:31:55 UTC. `python3 scripts/verify_linux_supervisor.py` comprovou restart após SIGKILL, PID alterado e schedule concluído. O relatório de integração local marcou 100% saudável numa janela curta, sem alegar meta mensal. Suíte Python passou em 46 testes (32 de jornada, dois do contrato CI e doze de relatório); cobertura confirmou 100/100 BDDs ligados a verificações executáveis; a configuração CI inclui o medidor de pontualidade nos três sistemas. A medição dedicada Linux em host ocioso passou: 100/100 inícios, zero falhas, P95 0,2505 s e P99 0,269469 s, após CPU entre 95,29% e 96,08% ociosa por 60 s. O teste de contrato confirma políticas de restart nos três pacotes; execução nativa macOS/Windows e CI remota tripla ainda requerem gates de release. O BDD Rust confirmou lacuna/overlap DST e teto de recuperação. A suíte Playwright passou em 15 testes.
- Documentação afetada: configuração CI, guias de teste e operação, `CONTRIBUTING.md`, `AGENTS.md`, definição técnica e workflow state.
#### TASK-012 — Concluir instalação manual e documentação de uso

- Status de execução: guias multiplataforma revisados e atualizados; Linux instalado e validado, macOS/Windows preparados para reprodução nativa antes do release.
- Evidência Linux (2026-10-13): build release e UI passaram; daemon e CLI instalados em `~/.local`, serviço systemd habilitado, `/health` saudável e schedules recuperados após restart supervisionado. O instalador Windows também foi revisado para parar a tarefa antes de substituir binário/assets; os procedimentos Windows e macOS ainda não foram executados nativamente.

- Descrição: fornecer instruções reproduzíveis para dependências, compilação, registro/início automático, acesso à UI, uso da API, upgrade, diagnóstico, parada e desinstalação nos três sistemas operacionais.
- Fontes: REQ-001–REQ-006; JRN-001–JRN-015; DEC-003–DEC-004, DEC-012, DEC-016, DEC-021, DEC-032, DEC-036.
- Dependências: TASK-001, TASK-006–TASK-011, TASK-013–TASK-015.
- Subtarefas: registrar prerequisites C/Rust/Node; passos de build e service por SO; exemplos de uso da CLI para criação, manutenção e execução; exemplos de API/OpenAPI; limites de segurança localhost; configuração de SQLite/histórico; atualizar documentação raiz e navegação.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: BDD-JRN-001-*, BDD-JRN-002-*, BDD-JRN-003-*, BDD-JRN-004-*, BDD-JRN-005-*, BDD-JRN-006-*, BDD-JRN-007-*, BDD-JRN-008-*, BDD-JRN-009-*, BDD-JRN-010-*, BDD-JRN-011-*, BDD-JRN-012-*, BDD-JRN-013-*, BDD-JRN-014-*, BDD-JRN-015-*
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: processo e comandos de build/instalação implementados nos ambientes alvo; endpoints e UI estáveis.
- Definição de concluído: procedimentos executados/revisados em Windows/macOS/Linux; CLI instalada e seus comandos de criação/gerenciamento exemplificados; `README.md`, `CONTRIBUTING.md`, `AGENTS.md` e `.gitignore` refletem projeto real; `docs/README.md` aponta tutoriais/guias/referência existentes; documentação de comando distingue programa/args e shell explícito; nenhuma instrução de build inventada.
- Resultado esperado: desenvolvedor consegue instalar manualmente, iniciar o daemon, abrir localhost, criar/gerenciar schedules pela CLI e web, e consultar resultados em cada SO suportado.
- Documentação afetada: `README.md`, `CONTRIBUTING.md`, `AGENTS.md`, `.gitignore`, `docs/README.md`, docs how-to/reference a criar.
#### TASK-013 — Implementar CLI local de criação de schedules

- Status de execução: criação única/recorrente integrada e BDDs JRN-013 validados no Linux; CI tripla permanece no gate de release.

- Descrição: criar a base do cliente CLI multiplataforma e os comandos de criação de schedules únicos/recorrentes via API local do daemon, sem duplicar validação, persistência ou lógica de scheduling. Deve aceitar programa + args, horário/calendário, parâmetros JSON e políticas configuráveis suportadas na criação; apresentar erros de validação/conexão e ID aceito.
- Fontes: REQ-001; JRN-013; DEC-036; escopo registrado em `AGENTS.md`.
- Dependências: TASK-001, TASK-006, TASK-007.
- Subtarefas: definir interface de comandos/flags e ajuda; serializar entrada JSON sem interpolação em shell; chamar API por localhost; respeitar os mesmos limites e políticas do contrato; exibir ID/erro de forma acionável; incluir CLI no build e empacotamento manual dos três SOs.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: BDD-JRN-013-*
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: contrato de criação OpenAPI disponível e daemon/API locais executáveis; sintaxe cron padrão de cinco campos selecionada e registrada em DEC-037/TASK-002 conforme legibilidade, interoperabilidade e validação compartilhada entre plataformas.
- Definição de concluído: CLI cria schedule único/recorrente via API; schedule aceito aparece na CLI/web/API; entrada inválida ou daemon indisponível não é apresentada como sucesso; erros e uso são documentados; cenários BDD JRN-013 cobrem sucesso, dados inválidos e daemon indisponível nos três SOs.
- Resultado esperado: criação pelo terminal compõe com web/API e compartilha exatamente o contrato do daemon.
- Documentação afetada: guia CLI, `README.md`, `CONTRIBUTING.md`, OpenAPI se necessário, definição técnica e workflow state.
#### TASK-014 — Implementar consulta e manutenção de schedules pela CLI

- Status de execução: operações JRN-014 integradas e BDDs validados no Linux, incluindo listagem, atualização, pausa/retomada, cancelamento e parâmetros; CI tripla permanece no gate de release.

- Descrição: adicionar comandos CLI para listar/consultar e editar configuração de schedule, atualizar parâmetros, pausar, retomar e cancelar, usando os mesmos endpoints e regras de validação do daemon.
- Fontes: REQ-001–REQ-002, REQ-004–REQ-005; JRN-014; DEC-036.
- Dependências: TASK-001, TASK-007, TASK-013.
- Subtarefas: definir comandos/flags e saída legível/máquina; implementar list/show/update/parameters/pause/resume/cancel; exibir estados atualizados e erros sem falso sucesso; cobrir disparo opcional por atualização de parâmetros.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: BDD-JRN-014-*
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: operações correspondentes no contrato OpenAPI e CLI base executável.
- Definição de concluído: as operações de schedule cobertas por JRN-014 funcionam pela CLI; estado resultante confere com API/web; alterações inválidas não persistem; BDD cobre listar, consultar, atualizar, pausar/retomar, cancelamento e gatilho por atualização.
- Resultado esperado: o desenvolvedor administra configuração e estado dos schedules pelo terminal.
- Documentação afetada: guia CLI, `README.md`, OpenAPI, definição técnica, jornadas e workflow state.
#### TASK-015 — Implementar consulta e controle de execuções pela CLI

- Status de execução: operações JRN-015 integradas e BDDs validados no Linux, incluindo execução, histórico/saída, cancelamento e retry; CI tripla permanece no gate de release.

- Descrição: adicionar comandos CLI para executar agora, consultar histórico/detalhes/saídas, cancelar execução ativa, repetir falhas e confirmar retry de execução interrompida.
- Fontes: REQ-001–REQ-002, REQ-006; JRN-015; DEC-011, DEC-017, DEC-031, DEC-036.
- Dependências: TASK-001, TASK-008, TASK-013.
- Subtarefas: definir comandos/flags e saída legível/máquina; implementar run/list-runs/show/cancel-run/retry; solicitar confirmação explícita para retry interrompido; apresentar exit code, stdout/stderr e truncamento; distinguir pedido de cancelamento do término.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: BDD-JRN-015-*
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: endpoints de ocorrências e estados OpenAPI implementados; CLI base disponível.
- Definição de concluído: operações de execução cobertas por JRN-015 funcionam via CLI e refletem estado API/web; confirmações e estados de cancelamento são inequívocos; BDD cobre run-now, consulta, saída, cancelamento, retry falho e retry interrompido.
- Resultado esperado: o desenvolvedor inspeciona e recupera execuções pelo terminal sem contornar as regras do daemon.
- Documentação afetada: guia CLI, `README.md`, OpenAPI, definição técnica, jornadas e workflow state.
