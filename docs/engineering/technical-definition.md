# Command Scheduler — Definição técnica

- Status: ready
- Revisão: 61
- Histórico de revisões: [registro central](../revision-history.md) — seção `engineering/technical-definition.md`
- Atualizado: 2026-10-13
- Fontes e revisões: [PRD r19](../product/prd.md), [requisitos r5](../product/requirements.md), [jornadas r34](../product/user-journeys.md)
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.
- Artefatos relacionados: [estado do workflow](../product/workflow-state.md), [índice de documentação](../README.md)

## Restrições e preferências

### Confirmado pelo produto

- Produto executa localmente na máquina do desenvolvedor; interface web e API local.
- API do MVP sem autenticação e acessível somente por localhost.
- Comandos rodam com as permissões do usuário local; execuções do mesmo schedule podem ocorrer em paralelo.
- Schedules únicos e recorrentes por calendário flexível; recorrências usam fuso da máquina.
- Parâmetros podem ser atualizados até o início da execução; atualização pode disparar execução imediata por opção do schedule.
- O scheduler deve operar como serviço/daemon do sistema iniciado automaticamente, sem depender da interface web aberta.
- O MVP deve suportar Windows, macOS e Linux.
- O usuário pretende aprender e construir com Rust usando apoio de IA; composição da equipe e experiência anterior não foram especificadas.
- UI confirmada em Vue 3 + TypeScript + Vite.
- A interface web local deve ser aberta no navegador em localhost.
- Schedules e configurações persistem após reinício da máquina; o tratamento de ocorrência vencida é configurável por schedule.
- Se houver ocorrências vencidas em modo consolidado ou sequencial, as execuções usam os parâmetros mais recentes disponíveis.
- Se o daemon/máquina reiniciar com um comando em andamento, uma nova execução exige confirmação manual; não há retry automático nesse caso.
- Histórico de execuções deve ser mantido por 90 dias.
- No modo sequencial, até 100 ocorrências vencidas por schedule são enfileiradas; excedentes são consolidadas em uma execução.
- Uma execução consolidada recebe os parâmetros mais recentes e metadados com a quantidade e o intervalo das ocorrências cobertas.
- O contexto de execução disponibiliza os metadados da consolidação ao comando executado e pela API.
- A quantidade e os limites temporais consolidados são expostos como variáveis de ambiente do processo e como campos da ocorrência na API.
- Parâmetros do schedule são disponibilizados ao processo como valor JSON em uma variável de ambiente dedicada; valores não são interpolados no texto do comando.
- Criação pela API aceita chave de idempotência opcional para que retries da mesma solicitação não criem schedules duplicados.
- Retry automático padrão para falhas normais: 3 tentativas com intervalos de 1, 5 e 15 minutos; configurável por schedule.
- Teste de comando web usa `POST /api/v1/commands/test`: execução real, sem shell, sob as permissões do daemon, com parâmetros na variável `COMMAND_SCHEDULER_PARAMS_JSON`, limite de 30 segundos e captura de até 1 MiB por stream. A resposta não persiste schedule nem ocorrência.
- Formulários web rotulam `program` como executável e `args` como array JSON de strings, dão exemplo e bloqueiam padrões comuns de linha de comando completa colada em `program`; caminhos com espaços não são rejeitados genericamente (DEC-040).
- A UI deve oferecer presets de recorrência em minutos/horas e calendário diário, semanal e mensal, mantendo cron avançado. Os intervalos usam limites do relógio/calendário local e são convertidos para cron de cinco campos; opções N de minuto dividem 60 e opções N de hora dividem 24 para manter cadência regular ao virar a hora/dia. Não alteram o modelo público da API (DEC-041).
- A tela inicial substitui o hero por cockpit operacional. O `<title>` do frontend é `commandscheduler`; o cockpit exibe saúde do daemon, próxima execução ativa, contagens por estado de schedule e execuções `succeeded`/`failed` concluídas dentro de hoje ou dos últimos 7/30/90 dias. Retrys contam como ocorrências individuais.
- A interface oferece agenda global por dia/semana/mês e filtros de estado para schedules. A agenda calcula instâncias futuras dos schedules ativos no intervalo; períodos passados mostram execuções persistidas. Histórico global ou por schedule permite filtro por período/estado e paginação; retenção segue os 90 dias definidos.
- Em recorrências locais, horários inexistentes por avanço do relógio são pulados; horários repetidos por recuo do relógio executam uma vez.
- A API não exige autenticação para clientes locais; chamadas originadas de páginas web são aceitas somente da origem local da UI e não há CORS amplo.
- O MVP suporta versões de Windows, macOS e Linux ainda cobertas pelo suporte de segurança do fornecedor na data do lançamento.
- Os cenários BDD das jornadas devem ser automatizados e executados em CI nos três sistemas operacionais.
- O daemon será instalado manualmente em cada sistema operacional; o repositório deve fornecer instruções de compilação e instalação.

### Itens explicitamente sem definição para o MVP

- Experiência/capacidade da equipe, orçamento e ambiente de desenvolvimento não foram especificados.
- Meta e janela da métrica de schedules migrados, volume/carga e conformidade formal de acessibilidade ficam explicitamente fora das decisões do MVP. Para a métrica, consultar a decisão de acompanhamento descritivo registrada no PRD r17; para os outros itens, consultar as restrições do PRD e suas fontes.

## Arquitetura e limites de projeto

### Limites conhecidos

- O repositório atual é um único limite de projeto; não há subprojetos frontend/backend/infra confirmados.
- Stack do MVP: runtime/API Rust com Axum + Tokio + SQLx/SQLite; interface Vue 3 + TypeScript + Vite. A aplicação permanece em um único limite de projeto até a estrutura de implementação ser criada.

### Arquitetura escolhida para o MVP

Um daemon Rust é o limite operacional local do MVP: mantém o scheduler, a API Axum/REST, o banco SQLite e o executor de processos; também serve os arquivos compilados da interface Vue na mesma origem local. Um CLI companion cria e gerencia schedules e execuções usando a API local, sem lógica de domínio própria. Tokio coordena I/O/timers; SQLx acessa SQLite. O repositório fornecerá instruções de compilação e instalação manual para Windows, macOS e Linux. Daemon e CLI pertencem ao mesmo projeto; subprojetos separados não são necessários até existir fronteira real de build/deploy.

## Dados, interfaces e integrações

- A API HTTP REST/JSON v1 local precisa criar, consultar, alterar, pausar/retomar/cancelar schedules, atualizar parâmetros, iniciar execução imediata, consultar/cancelar/repetir ocorrências e retornar estados/resultados observáveis conforme JRN-002, JRN-003, JRN-006, JRN-007 e JRN-010–JRN-012.
- A interface web deve expor criação, consulta, edição, pausa/retomada, cancelamento, execução imediata e recuperação de falhas conforme JRN-001, JRN-004–JRN-005 e JRN-008–JRN-009.
- Dados centrais incluem programa + args, configuração de horário, gatilhos, parâmetros vigentes, política de ocorrência vencida, retry e estado do schedule. Ocorrências guardam origem (horário, atualização, manual ou retry), horários previstos/início/fim, estado e resultado necessário ao controle. SQLite é o armazenamento local; registros de histórico de execução são retidos por 90 dias. Chave de idempotência opcional fica associada ao schedule até sua remoção.
- Corpo de criação: programa, lista de argumentos, schedule e política de recuperação, com parâmetros JSON opcionais em `parameters`; parâmetro atualizado é aplicado até o início da execução e repassado como `COMMAND_SCHEDULER_PARAMS_JSON`.
- Dados de recuperação consolidada aparecem na API em `missedOccurrences: { count, from, through }`; variáveis equivalentes são `COMMAND_SCHEDULER_MISSED_COUNT`, `COMMAND_SCHEDULER_MISSED_FROM` e `COMMAND_SCHEDULER_MISSED_THROUGH`. `from`/`through` e variáveis temporais usam RFC 3339.
- Contrato de API v1: [OpenAPI 3.1](api/openapi.yaml) define recursos, paginação cursor/limit, respostas, erros Problem Details, idempotência e payloads. A UI local publica a referência navegável em `/docs/api` e o documento canônico em `/openapi.yaml`.
- Cockpit, agenda e histórico usam `GET /overview`, `GET /agenda` e `GET /occurrences`, sem alterar criação/edição de schedules. `/overview` agrega schedules por estado e ocorrências `succeeded`/`failed` por `finishedAt` no intervalo RFC 3339 semiaberto `[from,to)`; a UI calcula os limites no calendário local e conta cada retry. `/agenda` combina instâncias futuras de schedules ativos com execuções finalizadas no período local solicitado e limita a resposta a 1000 itens. `/occurrences` filtra por período, estado e schedule, pagina até 100 itens em ordem estável de criação e omite stdout/stderr; detalhes e saídas são consultados sob demanda em `/occurrences/{occurrenceId}`. `/schedules` aceita filtro por estado com paginação. O armazenamento mantém índices para as consultas operacionais e o histórico continua limitado a 90 dias (DEC-042).
- Persistência definida em DEC-038: SQLite em diretório local de dados do usuário, sobrescrevível por `COMMAND_SCHEDULER_DB_PATH`; migrações SQLx embutidas são aplicadas no startup. Tempos são Unix em microssegundos no banco e RFC 3339 na API. SQLite usa WAL, chaves estrangeiras habilitadas e timeout de lock de 5 s. Expurgo diário remove ocorrências finalizadas há mais de 90 dias e preserva schedules.
- O cursor `last_evaluated_at` por schedule é persistido em microssegundos; a criação transacional de ocorrências recuperadas e o avanço do cursor usam comparação otimista para evitar duplicidade por varreduras concorrentes; schedules existentes recebem cursor inicial igual a `created_at` na migração aditiva v2. O daemon reavalia schedules ativos a cada 250 ms, preserva a janela de pontualidade de 5 s, enfileira timers e retries vencidos, executa sem limite global de concorrência e marca ocorrências em execução como interrompidas no startup. A implementação do cron usa parser com segundos/ano desabilitados, apenas algarismos e `* , - /`, sem aliases/extensões; candidatos civis são convertidos pelo fuso local para aplicar DEC-018. A recuperação limita a 100 ocorrências individuais e agrega excedentes com count/from/through. O executor inicia `program` diretamente com `args`, injeta parâmetros JSON e metadados de vencidas no ambiente, drena `stdout`/`stderr` limitando cada stream a 1 MiB e persiste código de saída/truncamento. Falha normal agenda retry segundo `maxRetries`/`delaysSeconds`; cancelamento termina o processo e não agenda retry. Recorrências usam cron padrão de cinco campos, conforme [DEC-037](decisions/scheduling-and-execution.md#dec-037--usar-cron-padrao-de-cinco-campos-para-recorrencias), no fuso local da máquina; semântica de horário inexistente/repetido segue DEC-018.
- O processo é iniciado diretamente como `program` com `args`, sem shell implícito; shell e script devem aparecer explicitamente na configuração. Comandos rodam sob o usuário local; execuções concorrentes do mesmo schedule permanecem permitidas.
- Não há integrações externas além dos clientes locais da API confirmadas.
- O cliente CLI local cria e gerencia schedules/execuções chamando a API REST/JSON do daemon; somente a API é fonte de validação e persistência.
- Operações CLI devem cobrir os recursos de gerenciamento da API: listar/consultar/editar, pausar/retomar/cancelar schedule, atualizar parâmetros, executar agora, consultar histórico/detalhes/saídas, cancelar execução, repetir falha e confirmar retry de execução interrompida.

## Segurança e requisitos de qualidade

- Restrição confirmada: API acessível somente por localhost, sem autenticação no MVP. Execuções usam privilégios do usuário local.
- Chamadas de páginas web de origens não autorizadas devem ser rejeitadas; clientes locais fora do navegador continuam sem autenticação conforme escopo confirmado.
- CLI e aplicações locais chamam a API por localhost sem autenticação; restrição de origem se aplica a chamadas feitas por navegador.
- Pontualidade: P95 ≤ 5 s e P99 ≤ 30 s em máquina ociosa, sem carga de referência formal. Não há limite global de concorrência no MVP.
- Disponibilidade: 99,5% mensal durante tempo ligado e apto a executar; períodos desligados/suspensos excluídos. Reinício automático do daemon e recuperação dos schedules persistidos são requisitos.
- Acessibilidade: sem conformidade WCAG formal no MVP; navegação por teclado e rótulos acessíveis são requisitos básicos.
- Não há metas formais de volume/carga, nem meta numérica ou janela para schedules migrados. Privacidade e requisitos regulatórios não foram especificados.

## Desenvolvimento e verificação

- Toolchain escolhida: Rust + Axum/Tokio/SQLx com SQLite embutido; UI Vue 3 + TypeScript + Vite. Build SQLx/SQLite requer C build tools, que devem constar nas instruções multiplataforma.
- Estratégia confirmada: automatizar cenários BDD ligados às jornadas e executá-los em CI em Windows, macOS e Linux. Complementar com unit/integration para persistência, parser de schedule, idempotência, processos, concorrência, captura/truncamento de saída, CLI, recuperação e pontualidade P95/P99 em máquina ociosa; provedor CI, framework e comandos de build/teste serão selecionados nas tarefas sem checkpoint de aprovação.
- A estrutura de implementação foi iniciada na raiz como um único projeto: Cargo compila daemon e CLI; pnpm mantém a UI Vue 3 + TypeScript + Vite como pacote do mesmo workspace e compila `ui/dist`, servido pelo daemon. `rust-toolchain.toml` seleciona Rust stable com rustfmt e Clippy; `rust-version = 1.85` define a versão mínima declarada. A toolchain Rust foi instalada temporariamente em `/tmp` para validação Linux; `cargo check` também passou nos alvos Windows GNU e macOS x86_64, sem linkagem dos executáveis.
- Orientação temporária do usuário (2026-10-13): durante a implementação, a validação pode ocorrer somente em Linux. Compatibilidade nativa Windows/macOS e CI BDD nos três sistemas permanecem critérios de release no [plano](../planning/implementation-plan.md#regra-temporaria-de-validacao-durante-implementacao).
- Comandos conhecidos de instalação, desenvolvimento, build, formatação e Clippy estão no `README.md` e `CONTRIBUTING.md`. CLI e `/health` existem como scaffolds; comportamento de produto ainda será implementado nas tarefas dependentes.
- Validação contratual do OpenAPI: `pnpm openapi:lint`; referência Scalar é empacotada localmente e carrega sob demanda, sem CDN. Validação do banco usa migração/reabertura, idempotência, limite/atomicidade JSON e retenção em testes Rust.

## Entrega e operações

- Instruções de compilação e instalação manual dos binários daemon e CLI nos três sistemas confirmadas; incluir dependências C, registro/início automático, status, parada, atualização, diagnóstico e desinstalação. Logs estruturados, backup/restore e recuperação do arquivo SQLite ainda serão detalhados na implementação. Disponibilidade mensal alvo de 99,5% durante máquina ligada/apta, com reinício automático do daemon e recuperação dos schedules persistidos.
- A solução é local no MVP; não existe hospedagem cloud ou pipeline de deployment confirmados.
- Manter as políticas de documentação no root; criar arquivos obrigatórios em novos limites de projeto somente quando esses subprojetos forem efetivamente definidos.

## Cobertura de escopo

| Requirement / journey | Approach / decisions | MVP or future | Gaps |
|---|---|---|---|
| REQ-001; JRN-001, JRN-004, JRN-005, JRN-008–JRN-015 | Daemon local, CLI/web/API para criação e gerenciamento, calendário, recuperação por schedule, SQLite e processo program+args | MVP | Pontualidade definida em máquina ociosa, sem alvo formal de carga; concorrência sem limite global. |
| REQ-002; JRN-001–JRN-015 | UI local, API REST/JSON v1 e CLI cliente da API, com origem web restrita à UI; OpenAPI 3.1 completo e navegável | MVP | Paginação e detalhes de códigos HTTP serão especificados no contrato. |
| REQ-003; JRN-002 | API sem auth para clientes locais; idempotência opcional no create | MVP | Sem lacuna bloqueante; esquema/versionamento no OpenAPI. |
| REQ-004; JRN-002–JRN-003 | Qualquer JSON válido até 64 KiB em `COMMAND_SCHEDULER_PARAMS_JSON`; valores não interpolados no comando | MVP | Sem lacuna bloqueante de formato ou tamanho. |
| REQ-005; JRN-007 | Disparar por cada atualização aceita quando configurado; concorrência permitida | MVP | Semântica de erros de payload segue validação da API. |
| REQ-006; JRN-005, JRN-009, JRN-012 | Retry normal: três tentativas adicionais em 1/5/15 min; execução interrompida exige confirmação | MVP | Configuração fina dos retries por schedule será descrita na implementação. |
| REQ-007; JRN-001 | Endpoint transitório de teste retorna saídas e código; sem schedule/histórico; timeout de 30 s e limite de saída de 1 MiB por stream | MVP | Processo pode produzir efeitos locais; a UI avisa antes da ação. |
| REQ-008; JRN-001 | Presets web mapeiam intervalos locais e calendários para cron; cron avançado continua disponível (DEC-041) | MVP | Contrato de recorrência da API permanece inalterado. |
| REQ-009; JRN-016 | Cockpit web consome estado e agregados paginados/filtrados (DEC-042) | MVP | Nenhuma agregação deve depender de carregar todo o histórico no cliente. |
| REQ-010; JRN-016 | Agenda global por período local com detalhe do schedule (DEC-042) | MVP | Consultas devem paginar; próximas ocorrências podem ser calculadas a partir das configurações recorrentes. |
| REQ-011; JRN-016 | Filtros por estado e histórico global/por schedule com paginação por cursor (DEC-042) | MVP | Retenção mantém-se em 90 dias. |
| Futuro: execução remota/distribuída | Nenhum desenho comprometido | Fora do MVP confirmado | Reavaliar se o usuário ampliar o escopo. |

## Decisões confirmadas, premissas e itens encerrados

### Confirmado

- Restrições de produto listadas nesta definição vêm do PRD r19, requisitos r5 e jornadas r33. DEC-001 foi selecionada como direção técnica no planejamento; isso registra uma escolha de implementação, não uma nova exigência de produto confirmada pelo usuário. Criação e gerenciamento via CLI estão registrados em `AGENTS.md` e detalhados em JRN-013–JRN-015.
- Um único projeto/repositório é o limite confirmado; nenhum novo subprojeto foi criado ou precisa ser criado para começar o planejamento.

### Premissas

- O serviço deve cumprir schedules com a interface fechada, conforme a decisão confirmada de execução como daemon.
- Recuperação de execução que estava em andamento no reinício exige confirmação manual, mesmo que o schedule tenha retry automático para falhas normais.

### Itens encerrados sem decisão pendente

- Volume máximo de schedules/execuções não é uma meta do MVP. A meta de pontualidade vale apenas para máquina ociosa, sem inferência de capacidade sob carga.
- Schedules migrados serão acompanhados como métrica descritiva sem meta numérica ou período durante o MVP; o PRD r19 registra impacto e ponto de reavaliação.
- Acessibilidade formal WCAG fica fora do MVP; navegação por teclado e rótulos acessíveis continuam exigidos.
- Paginação e códigos HTTP concretos são escolhas de implementação da tarefa TASK-002, limitadas ao contrato OpenAPI 3.1 e às jornadas. O agente registra as escolhas e exemplos no contrato.


## Retomada de schedules

Ao retomar um schedule, o daemon avalia os horários vencidos desde o cursor persistido e aplica a política `missedPolicy` configurada: pular, consolidar ou executar em sequência. Durante a pausa, o cursor não avança e nenhum disparo por horário ocorre (DEC-039).

O endpoint `POST /commands/test` executa o comando de forma transitória e aguarda sua conclusão ou o limite de 30 segundos. O teste não registra ocorrência nem aplica política de retry; o processo e suas saídas são descartados após a resposta.


## Ciclo de vida do daemon

No Unix, o daemon inicia encerramento gracioso ao receber SIGINT ou SIGTERM; o systemd local reinicia falha inesperada e a inicialização seguinte marca ocorrências `running` como `interrupted`, exigindo confirmação para retry (DEC-011). O unit de referência Linux usa `packaging/systemd/command-scheduler.service`; instruções em [guia systemd](guides/linux-service.md).


Evidência Linux (2026-10-13): o unit `packaging/systemd/command-scheduler.service` foi copiado para o caminho de usuário documentado, ativado pelo manager systemd e operou com o binário e assets nos caminhos finais. `/health`, UI, OpenAPI e API responderam; um schedule gravado em SQLite continuou consultável após `systemctl --user restart`. A validação usou drop-in temporário para isolar o banco; unit, drop-in, binário/assets copiados e dados temporários foram removidos ao final. `systemd-analyze verify` também foi executado antes da instalação; a execução nativa de LaunchAgent e do serviço/tarefa Windows permanece no gate de release.


A CLI aceita apenas URLs HTTP de `localhost`/IP loopback para a API e encerra requests após 30 segundos; falhas são apresentadas sem confirmação de sucesso.

### Revisão visual Instrument Design (2026-10-19)

A interface oferece Chalk, Graphite e Workshop com preferência local e Workshop como padrão. Papéis de superfície, tinta e estados estão no CSS da UI; o contrato da API não muda. Fontes: TASK-009/010/018 e REQ-008–011. Evidência e escopo em [revisão visual](instrument-design-update.md).
