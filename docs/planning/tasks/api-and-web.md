# Tarefas de implementação — API e interface web

- Status: ready
- Revisão: 35
- Histórico de revisões: [registro central](../../revision-history.md) — seção `planning/tasks/api-and-web.md`
- Atualizado: 2026-10-13
- Fontes e revisões: [PRD r19](../../product/prd.md), [requisitos r5](../../product/requirements.md), [índice de jornadas r34](../../product/user-journeys.md), [definição técnica r61](../../engineering/technical-definition.md), [plano r59](../implementation-plan.md)
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.
- Artefatos relacionados: [estado do workflow](../../product/workflow-state.md), [índice de documentação](../../README.md)

#### TASK-007 — Implementar API local para schedule, parâmetros e controles

- Status de execução: API implementada e BDDs relacionados validados no Linux. O metadado público da recuperação consolidada usa `missedOccurrences` em API/OpenAPI/UI; a suíte confirmou a faixa e as variáveis recebidas pelo processo. Playwright exercita nove cenários web. A suíte Python também cobre criação inválida pela CLI (BDD-JRN-013-02) e reinício com recuperação pela política persistida (BDD-JRN-001-03). CI tripla permanece no gate de release.

- Descrição: implementar recursos OpenAPI para criar/listar/consultar/editar/pausar/retomar/cancelar schedules, atualizar parâmetros, disparar execução imediata e aplicar `Idempotency-Key`, validando origem de browser e bind em localhost.
- Fontes: REQ-002–REQ-005; JRN-002–JRN-003, JRN-006–JRN-007; DEC-002, DEC-020, DEC-023–DEC-028, DEC-032.
- Dependências: TASK-002–TASK-006.
- Subtarefas: bind localhost; respostas e erros conforme OpenAPI; limite de JSON ≤64 KiB e qualquer tipo JSON válido; persistência atômica de atualização; dispatch de gatilho imediato por atualização; cabeçalho idempotente opcional.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: BDD-JRN-002-*, BDD-JRN-003-*, BDD-JRN-006-*, BDD-JRN-007-*
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: esquema OpenAPI revisado; política CORS/origem e recursos de schedule definidos; componentes scheduler/executor acessíveis.
- Definição de concluído: endpoints e validação correspondem ao OpenAPI; clientes locais sem autenticação funcionam; chamadas de navegador de origem externa rejeitadas; payload inválido/grande não substitui parâmetros vigentes; BDD de cada jornada API correspondente passa; API não escuta interfaces de rede externas.
- Resultado esperado: aplicações locais conseguem criar/atualizar/controlar schedules e reconhecer cada resultado/conflict.
- Documentação afetada: OpenAPI, `docs/engineering/technical-definition.md`, guia de referência da API.
#### TASK-008 — Implementar API de ocorrências, histórico e recuperação

- Status de execução: API de ocorrências, recuperação e BDDs relacionados validados no Linux; CI tripla permanece no gate de release.

- Descrição: expor consulta de ocorrências, resultado/código de saída, saídas capturadas e truncamento; executar/cancelar ocorrências, retry manual e confirmação para execução interrompida.
- Fontes: REQ-001, REQ-002, REQ-006; JRN-004, JRN-008–JRN-012; DEC-011, DEC-013, DEC-017, DEC-019, DEC-031–DEC-032.
- Dependências: TASK-002–TASK-007.
- Subtarefas: listar/consultar ocorrência e histórico; cancelamento de execução separado do cancelamento de schedule; retry manual/automático; representação de stdout/stderr e flag truncada; expurgo por retenção.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: BDD-JRN-004-*, BDD-JRN-008-*, BDD-JRN-009-*, BDD-JRN-010-*, BDD-JRN-011-*, BDD-JRN-012-*
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: estados e respostas OpenAPI; executor e política de retenção implementados ou contratados.
- Definição de concluído: todas as jornadas API de consulta/controle/recuperação têm comportamento BDD; retry de ocorrência interrompida exige confirmação explícita; payload de ocorrência inclui `missedOccurrences` quando aplicável e metadados/saída dentro dos limites; OpenAPI atualizado.
- Resultado esperado: cliente pode consultar e controlar cada tentativa sem confundir schedule, ocorrência e processo.
- Documentação afetada: OpenAPI, guia da API, jornadas e definição técnica.
#### TASK-009 — Construir interface web para criar e manter schedules

- Status de execução: fluxo integrado; nove testes Playwright passaram no Linux, incluindo rótulos acessíveis e teclado. Validação BDD multiplataforma permanece no gate de release.

- Descrição: implementar fluxos web de criar e editar schedule, recorrência, política de vencidas, retries, parâmetros, gatilho por atualização, pausa/retomada/cancelamento e comandos `program`+`args`.
- Fontes: REQ-001–REQ-002, REQ-004–REQ-006; JRN-001, JRN-005; DEC-015, DEC-023, DEC-028, DEC-034.
- Dependências: TASK-001, TASK-007.
- Subtarefas: lista e formulário; validação de JSON/tamanho; formulário de calendário; estados de schedule; validação acessível; interação com API local.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: BDD-JRN-001-*, BDD-JRN-005-*
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: API de gerenciamento funcional; decisões de campos/validação acessíveis pela OpenAPI.
- Definição de concluído: usuário cria e mantém schedule pela UI; erros não são apresentados como sucesso; qualquer JSON válido até 64 KiB pode ser configurado; operações são possíveis com teclado e controles têm rótulos acessíveis; BDD JRN-001/JRN-005 passa.
- Resultado esperado: fluxo web completo para migrar e manter um schedule sem editar arquivos de cron.
- Documentação afetada: guia de uso local a criar, `README.md`, jornadas se fluxo observável divergir.
#### TASK-010 — Construir interface web de execução e recuperação

- Status de execução: fluxo integrado; nove testes Playwright passaram no Linux, incluindo histórico, saída, cancelamento e retry. Validação BDD multiplataforma permanece no gate de release.

- Descrição: exibir estado/histórico, resultado e streams de saída; permitir executar agora, cancelar execução, retry manual, configurar retry e confirmar retry de processo interrompido.
- Fontes: REQ-001–REQ-002, REQ-006; JRN-004, JRN-008–JRN-009; DEC-011, DEC-013, DEC-017, DEC-031, DEC-034.
- Dependências: TASK-008, TASK-009.
- Subtarefas: histórico detalhado; renderização segura do output e indicador de truncamento; ações condicionadas ao estado; polling/atualização sem assumir sucesso; acessibilidade básica.
- Entradas e limites para o agente: use as fontes e IDs acima como referência canônica; siga o [contrato de execução autônoma do plano](../implementation-plan.md#execucao-autonoma-do-plano).
- Cenários BDD: BDD-JRN-004-*, BDD-JRN-008-*, BDD-JRN-009-*
- Validação: use os comandos documentados em `CONTRIBUTING.md` e registre os comandos e evidências executados para a tarefa; não presuma comandos além dos arquivos de configuração e documentação vigentes.
- Escalonamento: use as condições de parada do [contrato do plano](../implementation-plan.md#execucao-autonoma-do-plano) para conflitos de fonte ou mudanças fora dos limites; prossiga com tarefas independentes.

- Definição de pronto: API de ocorrência pronta; estados e resultados têm representação OpenAPI.
- Definição de concluído: todas as jornadas web de consulta/controle/recuperação passam seus BDDs; saída truncada é identificada; ações distinguem pedido de cancelamento e parada concluída; UI mantém teclado e rótulos acessíveis.
- Resultado esperado: desenvolvedor acompanha e recupera execuções sem acessar logs dispersos ou cron diretamente.
- Documentação afetada: guia de uso local, README, jornadas e referência API.

#### TASK-016 — Testar comando no formulário de criação

- Status de execução: teste transitório funcional em Linux; 15 cenários Playwright passaram, incluindo BDD-JRN-001-15/16. CI tripla permanece no gate de release.
- Descrição: permitir que o usuário execute o programa preparado no formulário, com seus argumentos e parâmetros, e veja `stdout`, `stderr`, código de saída, truncamento ou timeout sem criar schedule nem ocorrência.
- Fontes: REQ-007; JRN-001, BDD-JRN-001-15; DEC-040.
- Dependências: TASK-007, TASK-009.
- Subtarefas: adicionar rota local transitória; executar processo diretamente sem shell; capturar saída com limite de 1 MiB por stream; encerrar após 30 segundos; apresentar aviso de efeitos locais e o resultado no formulário; separar visualmente Executável e Argumentos; detectar padrões comuns de linha completa no campo errado sem rejeitar genericamente caminhos com espaços; atualizar OpenAPI e documentação Diátaxis/skill.
- Entradas e limites para o agente: o processo executa com permissões do daemon e pode produzir efeitos reais; o teste não cria schedule ou ocorrência; programa e argumentos são validados pela API.
- Cenários BDD: BDD-JRN-001-15, BDD-JRN-001-16.
- Validação: compilar backend e interface, validar OpenAPI; não presumir comandos além dos documentados em `CONTRIBUTING.md`.
- Escalonamento: se o resultado não puder ser separado de uma ocorrência persistida ou se timeout/efeitos divergirem do requisito, parar e registrar a divergência.
- Definição de pronto: comportamento, timeout e limites definidos em REQ-007/DEC-040; endpoint documentado em OpenAPI.
- Definição de concluído: botão executa o comando e mostra o resultado; teste encerrado em 30 segundos; nenhuma linha de schedule/ocorrência é criada; efeito colateral comunicado; campos e validação reduzem a confusão entre executável e linha de comando; documentação de uso e skill atualizadas.
- Resultado esperado: usuário identifica problemas no executável ou argumentos antes de salvar o schedule.
- Documentação afetada: OpenAPI, guia de execução de comandos e skill Command Scheduler.

#### TASK-017 — Expor consultas globais para cockpit, agenda e histórico

- Status de execução: implementada e validada em Linux; endpoints agregados, agenda, filtros e paginação estão no OpenAPI 1.1.0 e cobertos por teste Rust/Playwright. A lista global omite saídas grandes e carrega detalhes sob demanda.

- Descrição: adicionar consultas somente leitura para agregados do cockpit, próximas instâncias de schedules ativos no intervalo solicitado e histórico global paginado; adicionar filtro de estado à lista paginada de schedules. Não alterar formatos de criação/edição de schedule nem persistir agregados redundantes.
- Fontes: REQ-009–REQ-011; JRN-016, BDD-JRN-016-01–02, BDD-JRN-016-05–07; DEC-042.
- Dependências: TASK-007 e TASK-008 (capacidades da API local e ocorrências existentes).
- Subtarefas: agregados totais de schedules por estado; contagens `succeeded`/`failed` por `finishedAt` na janela local; próximo horário de schedule ativo; gerar instâncias de recorrência futuras para o intervalo da agenda usando o parser e semântica existentes; consulta global de ocorrências com filtro por período/estado/schedule; cursores e ordenação estáveis; erros/vazios tipados e exemplos OpenAPI.
- Entradas e limites para o agente: leia OpenAPI e handlers atuais antes de escolher rotas; reusar os limites de página (padrão 50, máximo 100) e convenções cursor/erro existentes; não transferir todos os schedules ou ocorrências para a UI; manter API em localhost e mesmas proteções de origem. Limites de data usam fuso local. Métricas contam cada ocorrência/retry e somente estados finais `succeeded`/`failed`; histórico respeita 90 dias.
- Cenários BDD: BDD-JRN-016-01, BDD-JRN-016-02, BDD-JRN-016-05, BDD-JRN-016-06, BDD-JRN-016-07.
- Validação: usar os comandos backend, BDD e OpenAPI documentados em `CONTRIBUTING.md`; acrescentar testes de contrato e consulta para cada filtro, limite de página, período vazio e fuso local; validação de desenvolvimento em Linux, com CI tripla preservada como gate de release.
- Escalonamento: parar e reportar se implementação exigir alteração do contrato de schedule, persistência redundante, retenção diferente de 90 dias ou semântica de contagem divergente; escolha de rotas e DTOs permanece com o agente dentro do OpenAPI.
- Definição de pronto: REQ-009–REQ-011, DEC-042 e semântica de recorrência/tempo existentes disponíveis; OpenAPI atual e estrutura SQL consultados.
- Definição de concluído: queries agregam no backend sem carregar histórico integral; listagem de schedules e ocorrências é filtrável/paginada; próximas instâncias respeitam cron e DST atuais; cenários correspondentes automatizados e evidenciados; OpenAPI e documentação de referência atualizados.
- Resultado esperado: a UI recebe resumos e páginas globais representativas sem custo de transferir todas as execuções.
- Documentação afetada: OpenAPI, definição técnica, guia de uso/API Diátaxis, skill Command Scheduler se o contrato/documentação de integração mudar e estado do workflow.

#### TASK-018 — Construir presets de recorrência e cockpit web

- Status de execução: implementada e validada em Linux; title/cockpit, presets cron, agenda, filtros e históricos progressivos passam nos testes de navegador. BDD-JRN-016-01–07 e BDD-JRN-001-17/18 estão ligados a verificações; CI tripla permanece gate de release.

- Descrição: substituir o hero promocional por cockpit operacional; implementar título `commandscheduler`, presets de recorrência e cron avançado em criação/edição, agenda global dia/semana/mês, filtros de schedules e histórico progressivamente paginado usando TASK-017.
- Fontes: REQ-008–REQ-011; JRN-001, JRN-016 e BDD-JRN-001-17/18, BDD-JRN-016-01–07; DEC-041–DEC-042.
- Dependências: TASK-009, TASK-010, TASK-016 e TASK-017.
- Subtarefas: seletor de recorrência “a cada N minutos” (1/2/3/4/5/6/10/12/15/20/30), “a cada N horas” (1/2/3/4/6/8/12), horário diário, dias da semana/horário e dia do mês/horário; cron avançado; resumo legível para revisão; cockpit e seletor de janela; controles de agenda; filtros por estado; histórico global e por schedule com carregamento progressivo; estados de daemon indisponível/sem dados; título HTML; controles acessíveis e layout responsivo.
- Entradas e limites para o agente: usar fuso local e semântica cron existentes; presets mantêm API `timing` atual; “inativo” corresponde a pausado, cancelado permanece separado; período KPI é hoje/7/30/90 dias e números contam execuções por `finishedAt`; manter os requisitos básicos de teclado/rótulos. Consultas usam TASK-017, sem paginação client-side inventada.
- Cenários BDD: BDD-JRN-001-17/18 e BDD-JRN-016-01–07.
- Validação: usar UI build, Playwright/BDD e OpenAPI checks indicados em `CONTRIBUTING.md`; cobrir interação de presets, conversão cron, navegação da agenda, filtros, paginação e estados vazios/erro. Validar localmente em Linux; manter CI BDD nos três sistemas como gate de release.
- Escalonamento: parar e reportar se presets não puderem ser expressos com a semântica cron existente sem alterar o contrato; mudanças no formato API, contagens ou retenção exigem nova decisão de produto.
- Definição de pronto: TASK-017 concluída; comportamento e semântica definidos em REQ-008–REQ-011/DEC-041–DEC-042.
- Definição de concluído: cockpit substitui o hero, apresenta todas as métricas definidas e título exato; presets criam/editam horários compatíveis com a API; agenda e histórico filtram/paginam sem carregar tudo; todos os BDDs relacionados passam; Diátaxis e skill são revisadas e atualizadas quando afetadas; revisão e evidência registradas no estado.
- Resultado esperado: operador configura recorrências comuns sem cron e acompanha saúde, agenda e execuções com volume alto.
- Documentação afetada: jornadas web, guias Diátaxis de criação/consulta e troubleshooting, skill Command Scheduler quando instruções de integração/consumo forem afetadas, plano e estado do workflow.
