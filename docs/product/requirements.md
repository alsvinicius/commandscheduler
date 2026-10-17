# Command Scheduler — Catálogo de requisitos

- Status: ready
- Revisão: 5
- Histórico de revisões: [registro central](../revision-history.md) — seção `product/requirements.md`
- Atualizado: 2026-10-13
- Fontes e revisões: PRD r19, escolhas do usuário sobre cockpit, agenda, presets de recorrência e histórico, confirmação de intervalos alinhados ao relógio local, e esclarecimento anterior sobre separação entre executável e argumentos
- Artefatos relacionados: [PRD](prd.md), [jornadas](user-journeys.md), [índice](../README.md)
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.

## Requisitos

### REQ-001 — Gerenciar agendamentos de comandos

- Necessidade e comportamento desejado: usuários devem conseguir definir schedules por horário e gerenciar, manter e controlar a execução de comandos executados na máquina.
- Prioridade e justificativa: central para a ideia declarada.
- Critérios de aceitação: o desenvolvedor consegue registrar programa e lista de argumentos para execução única ou recorrência por calendário flexível pela CLI; sintaxe de shell exige invocar o shell explicitamente. Schedules e configurações permanecem após reinício da máquina e a plataforma os dispara conforme configurado. Para cada schedule, o usuário escolhe pular ocorrências vencidas, consolidá-las em uma execução ao retornar ou executá-las em sequência. No modo sequencial, até 100 ocorrências por schedule ficam individuais e as excedentes são consolidadas com parâmetros atuais, quantidade e intervalo cobertos. Pela CLI, web e API é possível consultar e alterar schedules, pausar/retomar, cancelar, disparar agora e configurar repetição de falha; também é possível consultar/controlar ocorrências, resultados e saídas, conforme suporte da API. Histórico fica disponível por 90 dias.
- Fonte: pedido inicial do usuário e escopo registrado em `AGENTS.md`.

### REQ-007 — Testar um comando antes de salvar pela interface web

- Necessidade e comportamento desejado: o usuário deve conseguir verificar o comando que está preparando para um schedule e inspecionar sua saída antes de salvar.
- Prioridade e justificativa: reduz erros de configuração e torna claros problemas de executável e argumentos antes da criação.
- Critérios de aceitação: na criação web, o campo Executável identifica que aceita somente caminho/nome do programa, e o campo Argumentos explica o formato de array JSON e mostra um exemplo. Um padrão provável de linha de comando completa no campo Executável exibe orientação e impede testar ou salvar; a heurística não rejeita genericamente caminhos com espaços. O usuário pode executar `program` e `args` atuais com os parâmetros informados, ver `stdout`, `stderr`, código de saída e truncamento; o processo real roda com permissões do daemon, sem shell implícito, pode produzir efeitos no sistema e é encerrado após 30 segundos. O teste não cria schedule nem ocorrência/histórico. Falha ao iniciar é mostrada em `stderr`.
- Fonte: confirmação do usuário ao solicitar o botão de teste, 2026-10-13.

### REQ-002 — Gerenciar por web, API e CLI

- Necessidade e comportamento desejado: operadores devem gerenciar, manter e controlar agendamentos pela interface web, API e CLI local.
- Prioridade e justificativa: as três formas de interação são requisitos do MVP.
- Critérios de aceitação: o desenvolvedor consegue administrar schedules e execuções pela interface web e CLI; clientes locais conseguem realizar operações equivalentes pela API. A CLI chama a API local e oferece operações de consulta e manutenção de schedules, atualização de parâmetros, controle de execução e inspeção de resultados/saídas.
- Fonte: pedido inicial do usuário e confirmação de CLI de gerenciamento, 2026-10-13.

### REQ-003 — Aceitar agendamentos de aplicações externas

- Necessidade e comportamento desejado: aplicações externas podem solicitar a criação de agendamentos.
- Prioridade e justificativa: necessária para integrar consumidores além do operador humano.
- Critérios de aceitação: uma aplicação externa na máquina local consegue submeter um agendamento pela API HTTP REST/JSON sem autenticação e receber um resultado que permita saber se foi aceito ou por que falhou. A solicitação contém programa e argumentos e pode enviar `Idempotency-Key` opcional: mesma chave e conteúdo retornam o resultado original; mesma chave com conteúdo diferente é conflito. Chamadas de navegador são aceitas somente a partir da origem local da UI.
- Fonte: pedido inicial do usuário.

### REQ-004 — Aplicar parâmetros externos antes da execução

- Necessidade e comportamento desejado: aplicações externas conseguem definir parâmetros ao criar um agendamento e atualizá-los antes de cada execução.
- Prioridade e justificativa: explicitamente solicitado e distingue o fluxo de integração.
- Critérios de aceitação: valores fornecidos na criação ficam associados ao agendamento; a aplicação externa consegue atualizá-los até o início de cada execução. Se uma ocorrência chegar sem atualização, ela usa os últimos parâmetros disponíveis. O processo recebe qualquer valor JSON válido de até 64 KiB em `COMMAND_SCHEDULER_PARAMS_JSON`, sem substituição no texto do comando; payload inválido ou acima do limite é rejeitado e preserva os parâmetros vigentes.
- Fonte: pedido inicial do usuário.

### REQ-005 — Disparar execução após atualização de parâmetros

- Necessidade e comportamento desejado: permitir configurar um gatilho para executar o schedule quando os parâmetros forem atualizados.
- Prioridade e justificativa: explicitamente solicitado para permitir execução orientada pela preparação dos parâmetros.
- Critérios de aceitação: para um agendamento com esse gatilho habilitado, cada atualização de parâmetros dispara uma execução imediatamente. Por agendamento, o operador escolhe se esse gatilho substitui ou convive com o horário.
- Fonte: resposta de descoberta do usuário, 2026-10-12.

### REQ-006 — Repetir execuções com falha

- Necessidade e comportamento desejado: permitir que o operador repita uma execução com falha manualmente e, opcionalmente, configure repetição automática.
- Prioridade e justificativa: controle de execução solicitado como parte do produto.
- Critérios de aceitação: o operador pode repetir manualmente uma ocorrência com falha; pode configurar uma política automática de repetição. O padrão para falha normal é até três retries em 1, 5 e 15 minutos, configurável por schedule. Se uma execução for interrompida por reinício do serviço/máquina, nova tentativa exige confirmação manual.
- Fonte: resposta de descoberta do usuário, 2026-10-12.

### REQ-008 — Configurar recorrências por presets na interface web

- Necessidade e comportamento desejado: facilitar a configuração de recorrências comuns sem exigir que a pessoa escreva cron, preservando cron para regras avançadas.
- Prioridade e justificativa: melhora a criação e edição de schedules ao tornar a frequência compreensível antes de salvar.
- Critérios de aceitação: nos formulários web de criação e edição, a pessoa pode selecionar “a cada N minutos” para N em 1, 2, 3, 4, 5, 6, 10, 12, 15, 20 ou 30; “a cada N horas” para N em 1, 2, 3, 4, 6, 8 ou 12; execução diária em um horário, dias da semana e horário, ou dia do mês e horário. Pode trocar para cron avançado e revisar a expressão. Intervalos são alinhados aos limites do relógio/calendário local; as opções N dividem uma hora ou dia para preservar a cadência na virada. As frequências usam a semântica existente do cron; não há novo formato público de recorrência na API.
- Fonte: pedido do usuário para escolher intervalos de maneira mais amigável; confirmação de presets com cron avançado e de intervalos alinhados ao relógio local em 2026-10-13.

### REQ-009 — Exibir o estado operacional em um cockpit

- Necessidade e comportamento desejado: permitir que o desenvolvedor entenda rapidamente a saúde da plataforma, o próximo schedule e o estado geral dos agendamentos e execuções.
- Prioridade e justificativa: substitui conteúdo de apresentação por informação de operação acionável.
- Critérios de aceitação: a tela inicial apresenta o estado de saúde do daemon, o próximo schedule ativo, contagens de schedules ativos/pausados/cancelados e contagens de execuções bem-sucedidas e falhas. O período inicial é hoje, com seleção entre hoje e os últimos 7, 30 ou 90 dias para contagens de execução. Cada ocorrência de retry conta como execução independente; somente ocorrências `succeeded` são bem-sucedidas e `failed` compõe falhas, pela data de conclusão (`finishedAt`) no fuso local. O título da página é `commandscheduler`.
- Fonte: pedido do usuário por cockpit focado em indicadores e estado da plataforma/agendamentos; escolhas de indicadores e período confirmadas em 2026-10-13.

### REQ-010 — Consultar agenda global por período

- Necessidade e comportamento desejado: visualizar schedules e execuções organizados em um horizonte temporal útil.
- Prioridade e justificativa: permite reconhecer carga e atividade sem abrir cada schedule individualmente.
- Critérios de aceitação: a interface oferece agenda global navegável por dia, semana e mês, no fuso local; períodos futuros mostram as próximas instâncias dos schedules ativos e períodos passados mostram execuções registradas na retenção. Itens permitem abrir o schedule relacionado. A agenda inclui estados vazios e períodos sem ocorrências.
- Fonte: pedido do usuário por visão dia/semana/mês; preferência confirmada por agenda global com filtros e detalhe do schedule em 2026-10-13.

### REQ-011 — Filtrar schedules e navegar pelo histórico extenso

- Necessidade e comportamento desejado: encontrar schedules por estado e consultar históricos grandes sem uma lista longa e difícil de percorrer.
- Prioridade e justificativa: a visualização atual de “recentes” não escala para centenas de execuções.
- Critérios de aceitação: a lista global filtra schedules ativos, pausados/inativos e cancelados, preservando as categorias distintas. O histórico pode ser consultado globalmente ou no detalhe de um schedule, filtrado por período e estado da execução, e carregado em páginas com navegação progressiva. Consultas respeitam a retenção de 90 dias. A API permite buscar schedules por estado e ocorrências globalmente paginadas sem carregar o histórico inteiro.
- Fonte: pedido do usuário por filtros de estado e melhoria para centenas de execuções; escolhas de agenda global e detalhe por schedule confirmadas em 2026-10-13.
