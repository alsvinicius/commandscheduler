# Jornadas pela interface web — Command Scheduler

- Status: ready
- Revisão: 9
- Histórico de revisões: [registro central](../../revision-history.md) — seção `product/journeys/web.md`
- Atualizado: 2026-10-13
- Fontes e revisões: [PRD r19](../prd.md), [catálogo de requisitos r5](../requirements.md), [definição técnica r61](../../engineering/technical-definition.md)
- Artefatos relacionados: [índice de jornadas](../user-journeys.md), [estado do workflow](../workflow-state.md)
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.

### JRN-001 — Criar um schedule pela interface web

- Canal: Web
- Requisitos: REQ-001, REQ-002, REQ-007, REQ-008
- Ator, gatilho e pré-requisitos: desenvolvedor individual; quer migrar um comando de cron/script; plataforma disponível na máquina local.
- Resultado: schedule único ou recorrente criado e consultável pela interface.
- Inclusão no MVP e justificativa: incluída; fluxo principal para substituir agendamento disperso.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Escolhe criar schedule e informa comando, horário, parâmetros iniciais e política para horários vencidos durante indisponibilidade. | A interface apresenta a configuração para revisão e aponta dados inválidos ou ausentes. | Suporta execução única e recorrência por calendário flexível; na recorrência a pessoa pode escolher presets comuns ou cron avançado. Recorrência usa o fuso e o calendário da máquina; execução ocorre com permissões do usuário local. A política de ocorrência vencida é configurável por schedule. | O desenvolvedor consegue preparar um schedule único ou recorrente por preset/cron, escolher sua política de recuperação após indisponibilidade e corrigir erros antes de salvar. |
| 1a | Opcionalmente testa o comando antes de salvar. | A interface exibe stdout, stderr, código de saída e se a saída foi truncada; informa se o processo excedeu 30 segundos. | O teste inicia o processo real com os argumentos e parâmetros atuais, sem shell implícito. Pode produzir efeitos locais, não cria schedule nem ocorrência e termina o processo após 30 segundos. | O usuário consegue distinguir execução bem-sucedida, código não zero, falha ao iniciar e timeout sem alterar a lista ou histórico. |
| 1b | Informa o executável e os argumentos nos campos separados. | A UI explica que Executável recebe somente o caminho/nome e que Argumentos é um array JSON; sinaliza padrões comuns de linha completa colada no primeiro campo. | Caminhos válidos podem conter espaços, então a validação usa padrões prováveis, não rejeição genérica por espaço. | Uma linha como `/usr/bin/python3 -c ...` recebe orientação imediata e não pode ser testada ou salva até ser separada. |
| 2 | Confirma a criação. | A interface confirma e mostra o schedule na lista. | Sucesso só é apresentado após a criação ser aceita. | O schedule salvo pode ser consultado na interface. |

#### Caminhos de exceção e recuperação

- Configuração inválida não cria schedule ativo; os campos que precisam de correção ficam identificados.
- Se a plataforma não confirmar a criação, a interface mostra falha em vez de sucesso e permite nova tentativa.
- Falha ao iniciar ou código de saída diferente de zero no teste é mostrado como resultado do teste; não salva nem bloqueia a criação por si só.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-001-01 — Schedule válido**
  - Dado que o desenvolvedor informou um comando e um horário válido
  - Quando confirmar a criação
  - Então a interface confirma e exibe o schedule.
  - Rastreabilidade: JRN-001; REQ-001, REQ-002
- **BDD-JRN-001-02 — Schedule inválido**
  - Dado que falta um campo obrigatório ou a programação é inválida
  - Quando tentar salvar
  - Então a interface aponta a correção e não cria schedule ativo.
  - Rastreabilidade: JRN-001; REQ-001, REQ-002
- **BDD-JRN-001-03 — Schedule persistente**
  - Dado que o schedule foi salvo
  - Quando a máquina reiniciar e o serviço voltar
  - Então a configuração continua disponível e o comportamento de ocorrência vencida segue a política daquele schedule.
  - Rastreabilidade: JRN-001; REQ-001, REQ-002
- **BDD-JRN-001-04 — Pular vencidas**
  - Dado que o schedule usa a política de pular
  - Quando o serviço voltar após um ou mais horários
  - Então essas ocorrências não são executadas.
  - Rastreabilidade: JRN-001; REQ-001, REQ-002
- **BDD-JRN-001-05 — Consolidar vencidas**
  - Dado que o schedule usa a política de consolidar
  - Quando o serviço voltar após vários horários
  - Então uma execução consolidada é iniciada.
  - Rastreabilidade: JRN-001; REQ-001, REQ-002
- **BDD-JRN-001-06 — Executar vencidas em sequência**
  - Dado que o schedule usa a política de executar todas
  - Quando o serviço voltar após vários horários
  - Então as ocorrências vencidas são executadas em sequência.
  - Rastreabilidade: JRN-001; REQ-001, REQ-002
- **BDD-JRN-001-15 — Testar comando antes de salvar**
  - Dado que o usuário informou programa, argumentos e parâmetros
  - Quando executar o teste
  - Então a UI mostra as saídas e o código de saída, sem criar schedule ou ocorrência.
  - Rastreabilidade: JRN-001; REQ-007
- **BDD-JRN-001-16 — Separar executável e argumentos**
  - Dado que o usuário cola `/usr/bin/python3 -c ...` no campo Executável
  - Quando preencher ou tentar enviar o formulário
  - Então a interface explica como separar programa e argumentos e impede o teste e a criação até a correção.
  - Rastreabilidade: JRN-001; REQ-007
- **BDD-JRN-001-17 — Criar recorrência por preset**
  - Dado que o usuário escolhe um intervalo ou padrão diário, semanal ou mensal
  - Quando salvar o schedule
  - Então o schedule aparece com a frequência escolhida e é executado segundo o relógio e calendário local.
  - Rastreabilidade: JRN-001; REQ-008
- **BDD-JRN-001-18 — Usar cron avançado**
  - Dado que o usuário precisa de uma recorrência personalizada
  - Quando alternar para cron avançado e informar uma expressão válida
  - Então a interface salva e apresenta a expressão cron configurada.
  - Rastreabilidade: JRN-001; REQ-008
- **BDD-JRN-001-07 — Consolidar excedentes**
  - Dado que o schedule usa política sequencial e acumulou mais de 100 ocorrências
  - Quando o serviço retornar
  - Então até 100 ocorrências ficam enfileiradas individualmente e o excedente é consolidado em uma execução usando os parâmetros mais recentes.
  - Rastreabilidade: JRN-001; REQ-001, REQ-002
- **BDD-JRN-001-08 — Metadados consolidados**
  - Dado que uma execução consolida ocorrências vencidas
  - Quando o processo começar
  - Então recebe quantidade e limites temporais por variáveis de ambiente e a API retorna os mesmos dados estruturados.
  - Rastreabilidade: JRN-001; REQ-001, REQ-002
- **BDD-JRN-001-09 — Horário inexistente ou repetido**
  - Dado que uma recorrência coincide com avanço ou recuo do relógio local
  - Quando o serviço calcular a ocorrência
  - Então pula horário inexistente e agenda apenas uma execução no horário repetido.
  - Rastreabilidade: JRN-001; REQ-001, REQ-002
- **BDD-JRN-001-10 — Atraso sem reinício**
  - Dado que o horário de uma ocorrência passou enquanto a máquina estava suspensa ou o serviço atrasado
  - Quando o scheduler voltar a processar
  - Então aplica a política de vencidas configurada naquele schedule.
  - Rastreabilidade: JRN-001; REQ-001, REQ-002
- **BDD-JRN-001-11 — Parâmetros na UI**
  - Dado que o desenvolvedor informa qualquer JSON válido até 64 KiB
  - Quando salvar o schedule
  - Então a UI aceita; JSON inválido ou acima do limite é rejeitado sem criar schedule.
  - Rastreabilidade: JRN-001; REQ-001, REQ-002
- **BDD-JRN-001-12 — Acessibilidade básica**
  - Dado que o desenvolvedor usa apenas teclado ou leitor de tela
  - Quando percorrer e preencher o fluxo de criação
  - Então controles são alcançáveis por teclado e têm rótulos acessíveis.
  - Rastreabilidade: JRN-001; REQ-001, REQ-002
- **BDD-JRN-001-13 — Recuperação do daemon**
  - Dado que o daemon falha enquanto a máquina continua ligada e apta
  - Quando o supervisor reiniciar o serviço
  - Então schedules persistidos voltam a ser processados conforme suas políticas de ocorrências vencidas.
  - Rastreabilidade: JRN-001; REQ-001, REQ-002
- **BDD-JRN-001-14 — Pontualidade em máquina ociosa**
  - Dado que o ambiente de verificação está ocioso e mede inícios agendados
  - Quando a suíte calcular a distribuição do atraso
  - Então P95 é no máximo 5 s e P99 no máximo 30 s, sem inferir capacidade para uma carga não especificada.
  - Rastreabilidade: JRN-001; REQ-001, REQ-002

### JRN-016 — Acompanhar operação pela interface web

- Canal: Web
- Requisitos: REQ-009, REQ-010, REQ-011
- Ator, gatilho e pré-requisitos: desenvolvedor individual; abre a interface para entender o estado da plataforma, localizar atividade ou investigar execuções; daemon local disponível.
- Resultado: estado operacional e atividade relevante identificados, com acesso ao schedule e à execução relacionados.
- Inclusão no MVP e justificativa: incluída; dá visibilidade operacional e torna a interface utilizável com muitos schedules e execuções.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Abre a tela inicial e escolhe período dos indicadores. | Cockpit exibe saúde do daemon, próximo schedule ativo, schedules por estado e execuções bem-sucedidas/falhas no período. | Período padrão é hoje; opções são hoje e últimos 7, 30 e 90 dias. Limites de datas e `finishedAt` usam o fuso local. Retrys são ocorrências independentes. | O usuário distingue saúde do daemon, schedules ativos/pausados/cancelados e execuções `succeeded`/`failed`; a página identifica-se como `commandscheduler`. |
| 2 | Navega a agenda por dia, semana ou mês e seleciona um item. | Agenda global apresenta instâncias futuras de schedules ativos ou execuções históricas do período; item abre detalhe do schedule. | A agenda usa o fuso local e permite navegar entre períodos sem perder a visão global; histórico limita-se à retenção de 90 dias. | O usuário consegue localizar atividade por cada uma das três escalas sem carregar todo o histórico. |
| 3 | Filtra lista de schedules por estado e consulta execuções. | Lista mostra ativos, pausados/inativos ou cancelados; histórico permite filtros e navegação progressiva entre páginas. | Estado pausado é distinto de cancelado. Consultas de histórico respeitam retenção de 90 dias e podem ser globais ou por schedule. | Centenas de execuções podem ser consultadas sem carregar todas de uma vez; filtros de estado, período e schedule restringem os resultados. |

#### Caminhos de exceção e recuperação

- Sem schedules ativos, o cockpit informa que não há próxima execução; sem dados no período selecionado, indicadores mostram zero e a agenda/histórico exibe estado vazio.
- Se o daemon não responder, a saúde é apresentada como indisponível e os demais painéis não simulam valores atuais.
- Se uma página seguinte deixar de estar disponível, a interface mostra o erro e mantém filtros e resultados já carregados.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-016-01 — Cockpit e estado da plataforma**
  - Dado que o daemon e schedules estão disponíveis
  - Quando abrir a interface
  - Então vê saúde do daemon, próximo schedule e contagens de schedules e execuções.
  - Rastreabilidade: JRN-016; REQ-009
- **BDD-JRN-016-02 — Período dos indicadores**
  - Dado que há execuções em datas diferentes dentro do histórico retido
  - Quando selecionar hoje, 7, 30 ou 90 dias
  - Então as contagens de sucesso e falha refletem ocorrências concluídas (`finishedAt`) dentro da janela local escolhida, contando retries individualmente.
  - Rastreabilidade: JRN-016; REQ-009
- **BDD-JRN-016-03 — Agenda por escalas temporais**
  - Dado que há schedules e execuções em diferentes datas
  - Quando alternar entre dia, semana e mês
  - Então a agenda global mostra instâncias futuras dos schedules ativos ou execuções históricas no período local e permite abrir o schedule relacionado.
  - Rastreabilidade: JRN-016; REQ-010
- **BDD-JRN-016-04 — Filtrar schedules por estado**
  - Dado que existem schedules ativos, pausados e cancelados
  - Quando filtrar por estado
  - Então a lista contém somente os schedules daquela categoria e mantém pausados separados de cancelados.
  - Rastreabilidade: JRN-016; REQ-011
- **BDD-JRN-016-05 — Histórico paginado com filtros**
  - Dado que existem mais execuções que cabem em uma página
  - Quando filtrar por período, estado ou schedule e avançar páginas
  - Então cada página contém resultados correspondentes, sem exigir o carregamento do histórico completo.
  - Rastreabilidade: JRN-016; REQ-011
- **BDD-JRN-016-06 — Período sem atividade**
  - Dado que não há schedules ativos ou ocorrências no período selecionado
  - Quando abrir o cockpit ou a agenda
  - Então os indicadores mostram zero e a interface informa que não há próxima execução/atividade.
  - Rastreabilidade: JRN-016; REQ-009, REQ-010
- **BDD-JRN-016-07 — Daemon indisponível**
  - Dado que o daemon não responde
  - Quando abrir ou atualizar a interface
  - Então a saúde aparece como indisponível e a interface não apresenta os indicadores como se fossem dados atuais.
  - Rastreabilidade: JRN-016; REQ-009

### JRN-004 — Consultar o estado e resultado de uma execução

- Canal: Web
- Requisitos: REQ-001, REQ-002
- Ator, gatilho e pré-requisitos: desenvolvedor individual; quer saber se uma ocorrência está em andamento, terminou ou falhou.
- Resultado: estado/resultados disponíveis da execução conhecidos para decidir o próximo passo.
- Inclusão no MVP e justificativa: incluída; acompanhamento observável é necessário para controlar schedules e recuperar falhas.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Consulta as execuções do schedule pela interface web. | A interface mostra estado disponível, incluindo em andamento, concluída ou falha. | Histórico de execuções retido por 90 dias. | O desenvolvedor consegue distinguir execuções e resultados dentro do período de retenção. |
| 2 | Consulta uma ocorrência específica pela interface web. | A interface mostra o resultado conhecido e as ações disponíveis para aquele estado. | Registros fora dos 90 dias de retenção podem ter sido removidos. | O desenvolvedor consegue saber o resultado conhecido e escolher uma ação compatível com o estado. |

#### Caminhos de exceção e recuperação

- Se o estado não puder ser carregado, a interface comunica a indisponibilidade e permite consultar novamente.
- Se não houver ocorrências disponíveis, a interface informa que ainda não há resultados a mostrar.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-004-01 — Consulta com ocorrências**
  - Dado que o schedule tem execuções registradas
  - Quando o desenvolvedor consultar pela web
  - Então recebe o estado disponível de cada ocorrência.
  - Rastreabilidade: JRN-004; REQ-001, REQ-002
- **BDD-JRN-004-02 — Sem ocorrências**
  - Dado que o schedule ainda não executou
  - Quando o ator consultar
  - Então recebe uma resposta que indica ausência de resultados.
  - Rastreabilidade: JRN-004; REQ-001, REQ-002
- **BDD-JRN-004-03 — Falha ao consultar**
  - Dado que o estado não está disponível
  - Quando a consulta falhar
  - Então a interface informa a falha sem inventar um resultado.
  - Rastreabilidade: JRN-004; REQ-001, REQ-002
- **BDD-JRN-004-04 — Saídas e retenção**
  - Dado que uma ocorrência produziu `stdout` e `stderr`
  - Quando o desenvolvedor consultar dentro de 90 dias
  - Então as saídas capturadas estão disponíveis, cada stream limitado a 1 MiB, e qualquer truncamento fica indicado.
  - Rastreabilidade: JRN-004; REQ-001, REQ-002

### JRN-005 — Manter um schedule pela interface web

- Canal: Web
- Requisitos: REQ-001, REQ-002, REQ-006
- Ator, gatilho e pré-requisitos: desenvolvedor individual; schedule criado; quer ajustar seu ciclo de vida ou política de repetição.
- Resultado: configuração e estado do schedule refletem a ação confirmada.
- Inclusão no MVP e justificativa: incluída; mantém o controle operacional no mesmo lugar onde schedules são criados.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Consulta e edita comando, horário, parâmetros, gatilho por atualização, política de repetição automática ou política para horários vencidos. | A interface mostra os dados atuais, valida alterações e confirma quando são salvas. | Alterações de parâmetros respeitam o limite de início da execução; recorrência usa fuso da máquina. O gatilho pode substituir ou coexistir com horário; recuperação de ocorrência vencida é configurada por schedule. | O desenvolvedor consegue alterar essas configurações e ver o resultado salvo. |
| 2 | Pausa ou retoma o schedule. | A interface mostra o novo estado. | Enquanto pausado, não há novos disparos por horário; retomar reativa o schedule. | O estado pausado/ativo fica visível após cada ação. |
| 3 | Cancela o schedule. | A interface confirma cancelamento e mostra que o schedule não terá novos disparos. | Cancelar o schedule não implica, por si só, cancelar execução em andamento. | O schedule cancelado não gera novas ocorrências agendadas. |

#### Caminhos de exceção e recuperação

- Alteração inválida ou não confirmada não é mostrada como salva.
- A tentativa de cancelar um schedule já cancelado não cria um novo estado; a interface apresenta o estado atual.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-005-01 — Alterar schedule**
  - Dado que há um schedule ativo
  - Quando o desenvolvedor alterar dados válidos e salvar
  - Então a interface confirma a nova configuração.
  - Rastreabilidade: JRN-005; REQ-001, REQ-002, REQ-006
- **BDD-JRN-005-02 — Pausar e retomar**
  - Dado que o schedule está ativo
  - Quando o desenvolvedor pausá-lo e depois retomá-lo
  - Então cada novo estado fica visível, não há disparo por horário durante a pausa e horários vencidos são tratados pela política salva ao retomar.
  - Rastreabilidade: JRN-005; REQ-001, REQ-002, REQ-006
- **BDD-JRN-005-03 — Cancelar schedule**
  - Dado que o schedule está ativo
  - Quando o desenvolvedor cancelá-lo
  - Então novos disparos por horário deixam de ocorrer e a interface mostra o estado cancelado.
  - Rastreabilidade: JRN-005; REQ-001, REQ-002, REQ-006
- **BDD-JRN-005-04 — Política após indisponibilidade**
  - Dado que existe schedule recorrente
  - Quando o desenvolvedor alterar e salvar a política para horários vencidos
  - Então ela fica visível e é aplicada após a próxima indisponibilidade.
  - Rastreabilidade: JRN-005; REQ-001, REQ-002, REQ-006
- **BDD-JRN-005-05 — Selecionar política de recuperação**
  - Dado que há horários vencidos após indisponibilidade
  - Quando o usuário escolhe pular, consolidar em uma execução ou executar todas em sequência
  - Então a plataforma aplica a opção para aquele schedule.
  - Rastreabilidade: JRN-005; REQ-001, REQ-002, REQ-006
- **BDD-JRN-005-06 — Acessibilidade básica**
  - Dado que o desenvolvedor usa teclado ou leitor de tela
  - Quando navegar pela manutenção do schedule
  - Então os controles são acessíveis por teclado e têm rótulos acessíveis.
  - Rastreabilidade: JRN-005; REQ-001, REQ-002, REQ-006

### JRN-008 — Executar agora ou cancelar uma execução ativa

- Canal: Web
- Requisitos: REQ-001, REQ-002
- Ator, gatilho e pré-requisitos: desenvolvedor individual; schedule existente ou ocorrência ativa; precisa iniciar trabalho fora do próximo horário ou interromper uma ocorrência.
- Resultado: execução manual iniciada ou pedido de cancelamento submetido e seu estado conhecido.
- Inclusão no MVP e justificativa: incluída; ambos são controles expressamente requeridos, independentes do ciclo de vida do schedule.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Solicita executar agora ou cancelar uma ocorrência ativa pela interface web. | A interface confirma início ou pedido aceito, ou informa por que a ação não foi aplicada. | Execução manual pode coexistir com horários e outras execuções. Cancelar execução não cancela o schedule. | O desenvolvedor consegue executar agora e solicitar cancelamento de execução em andamento. |
| 2 | Consulta o estado após a ação pela interface web. | A interface mostra se a execução iniciou, terminou ou ainda pode estar em andamento. | Um pedido de cancelamento não equivale à confirmação de que o processo já parou. | O desenvolvedor consegue distinguir pedido aceito de cancelamento concluído. |

#### Caminhos de exceção e recuperação

- Sem execução ativa, cancelamento não é aplicado e a interface mostra o estado atual.
- Se não for possível iniciar agora, a interface informa falha sem indicar uma execução iniciada.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-008-01 — Executar agora**
  - Dado que o schedule pode executar
  - Quando o desenvolvedor pela web solicitar execução imediata
  - Então recebe confirmação de início ou estado observável de que ela foi iniciada.
  - Rastreabilidade: JRN-008; REQ-001, REQ-002
- **BDD-JRN-008-02 — Cancelar execução ativa**
  - Dado que uma ocorrência está ativa
  - Quando o ator solicitar cancelamento
  - Então recebe confirmação do pedido e pode consultar se a execução ainda continua.
  - Rastreabilidade: JRN-008; REQ-001, REQ-002
- **BDD-JRN-008-03 — Sem execução ativa**
  - Dado que não há ocorrência ativa
  - Quando solicitar cancelamento
  - Então a ação não é aplicada e o estado atual é informado.
  - Rastreabilidade: JRN-008; REQ-001, REQ-002

### JRN-009 — Recuperar uma execução com falha

- Canal: Web
- Requisitos: REQ-001, REQ-002, REQ-006
- Ator, gatilho e pré-requisitos: desenvolvedor individual; ocorrência falhou ou política automática está configurada.
- Resultado: uma nova tentativa é iniciada segundo escolha manual ou política automática, ou o usuário sabe que não haverá repetição.
- Inclusão no MVP e justificativa: incluída; recuperação de falhas foi explicitamente solicitada.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Configura política automática ou, após falha, solicita repetição pela interface web. | A interface confirma política salva ou mostra que nova tentativa começou. | Repetição pode ser manual ou automática configurável; padrão são três retries além da execução inicial, em 1, 5 e 15 minutos. | O desenvolvedor consegue configurar repetição automática e repetir manualmente uma ocorrência com falha. |
| 2 | Consulta a tentativa pela interface web. | Estado/resultado da tentativa fica disponível. | Execuções concorrentes são permitidas conforme o PRD. | O desenvolvedor consegue distinguir tentativa original e nova tentativa no estado apresentado, sem pressupor retenção além da definida. |

#### Caminhos de exceção e recuperação

- Ocorrência que não falhou não oferece repetição de falha; o estado atual é apresentado.
- Se uma nova tentativa não puder ser iniciada, a interface informa o motivo observável e preserva o resultado da tentativa original.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-009-01 — Repetição manual**
  - Dado que uma ocorrência terminou com falha
  - Quando o desenvolvedor pela web solicitar repetição
  - Então uma nova tentativa começa e pode ser consultada separadamente.
  - Rastreabilidade: JRN-009; REQ-001, REQ-002, REQ-006
- **BDD-JRN-009-02 — Repetição automática**
  - Dado que há política automática configurada
  - Quando uma ocorrência falhar
  - Então o produto aplica essa política e informa o estado da nova tentativa quando houver.
  - Rastreabilidade: JRN-009; REQ-001, REQ-002, REQ-006
- **BDD-JRN-009-03 — Repetição indisponível**
  - Dado que a ocorrência não falhou ou não pode ser repetida
  - Quando o ator solicitar nova tentativa
  - Então recebe o estado ou motivo e o resultado original é preservado.
  - Rastreabilidade: JRN-009; REQ-001, REQ-002, REQ-006
- **BDD-JRN-009-04 — Execução interrompida**
  - Dado que o serviço reiniciou e encontrou uma execução que estava em andamento
  - Quando o desenvolvedor solicitar repetição
  - Então a execução aparece como interrompida e a nova tentativa só começa após confirmação manual.
  - Rastreabilidade: JRN-009; REQ-001, REQ-002, REQ-006
- **BDD-JRN-009-05 — Retry automático padrão**
  - Dado que uma falha normal ocorreu e o schedule mantém política padrão
  - Quando as tentativas forem aplicadas
  - Então há até três retries nos intervalos de 1, 5 e 15 minutos, exceto se uma tentativa tiver sucesso ou o schedule for alterado/cancelado.
  - Rastreabilidade: JRN-009; REQ-001, REQ-002, REQ-006
