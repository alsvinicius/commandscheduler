# Jornadas pela CLI — Command Scheduler

- Status: ready
- Revisão: 3
- Histórico de revisões: [registro central](../../revision-history.md) — seção `product/journeys/cli.md`
- Atualizado: 2026-10-13
- Fontes e revisões: [PRD r17](../prd.md), [catálogo de requisitos r1](../requirements.md), [definição técnica r48](../../engineering/technical-definition.md)
- Artefatos relacionados: [índice de jornadas](../user-journeys.md), [estado do workflow](../workflow-state.md)
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.

### JRN-013 — Criar um schedule pelo terminal

- Canal: CLI
- Requisitos: REQ-001
- Ator, gatilho e pré-requisitos: desenvolvedor individual; precisa migrar um comando agendado; daemon local instalado e disponível.
- Resultado: schedule criado pela linha de comando, com identificador consultável e gerenciável pela CLI, UI e API.
- Inclusão no MVP e justificativa: incluída; o contexto do repositório exige criação via terminal, além da integração por aplicações externas.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Executa o cliente CLI com programa, argumentos, horário e configurações opcionais do schedule. | CLI valida os valores e apresenta erros acionáveis sem criar schedule inválido. | CLI é cliente local da API do daemon; criação aceita os mesmos campos, limites e políticas da API. | Desenvolvedor consegue criar schedules únicos ou recorrentes sem usar a UI. |
| 2 | Confirma a submissão e guarda o identificador retornado. | CLI confirma somente após a criação ser aceita e exibe ID/consulta. | Schedule fica disponível para manutenção completa pela CLI, web e API; falha de conexão não é apresentada como sucesso. | Schedule criado aparece nos três canais com configuração equivalente. |

#### Caminhos de exceção e recuperação

- Dados inválidos ou JSON de parâmetros acima de 64 KiB são rejeitados com indicação do campo/limite e sem schedule ativo.
- Se o daemon estiver indisponível ou a resposta se perder, CLI mostra falha/resultado desconhecido e não afirma sucesso.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-013-01 — Criação por CLI**
  - Dado que o daemon local está disponível e argumentos/horário são válidos
  - Quando o desenvolvedor executar o comando de criação
  - Então recebe ID e consegue localizar o schedule pela CLI, web/API.
  - Rastreabilidade: JRN-013; REQ-001
- **BDD-JRN-013-02 — Criação inválida**
  - Dado que os dados estão inválidos
  - Quando o desenvolvedor executar a CLI
  - Então recebe erro acionável e nenhum schedule ativo é criado.
  - Rastreabilidade: JRN-013; REQ-001
- **BDD-JRN-013-03 — Daemon indisponível**
  - Dado que o daemon local não responde
  - Quando o desenvolvedor executar a CLI
  - Então recebe indicação de falha sem falsa confirmação.
  - Rastreabilidade: JRN-013; REQ-001

### JRN-014 — Consultar e manter schedules pela CLI

- Canal: CLI
- Requisitos: REQ-001, REQ-002, REQ-004, REQ-005
- Ator, gatilho e pré-requisitos: desenvolvedor individual; quer inspecionar ou ajustar um schedule existente; daemon local disponível e schedule identificado.
- Resultado: schedule consultado ou atualizado pela linha de comando, refletindo o mesmo estado disponível na web/API.
- Inclusão no MVP e justificativa: incluída; gerenciamento via terminal é requisito explícito do MVP e evita exigir UI para manutenção cotidiana.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Lista schedules ou consulta um schedule por identificador. | CLI apresenta configuração, estado e próximo horário, ou informa que o schedule não existe. | CLI consulta a API local e respeita paginação/filtros definidos no contrato. | Desenvolvedor identifica schedules e estado sem abrir navegador. |
| 2 | Altera configuração, agenda, parâmetros ou política; pausa, retoma ou cancela schedule. | CLI apresenta estado atualizado ou erro sem declarar mudança não confirmada. | Validação e persistência são as mesmas da API; atualização de parâmetros respeita início da execução e pode disparar execução se o gatilho estiver ativo. | As operações de manutenção disponíveis na API podem ser realizadas via CLI. |

#### Caminhos de exceção e recuperação

- Identificador inexistente, configuração inválida ou parâmetros acima de 64 KiB resultam em erro acionável sem alteração parcial.
- Se a execução já começou, parâmetros daquela execução não podem ser alterados; atualizações posteriores seguem a política do schedule.
- Se o daemon estiver indisponível, a CLI indica falha e não apresenta estado local como atualizado.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-014-01 — Listar e consultar**
  - Dado que há schedules
  - Quando o desenvolvedor listar ou consultar por ID
  - Então a CLI exibe os mesmos dados de estado retornados pela API.
  - Rastreabilidade: JRN-014; REQ-001, REQ-002, REQ-004, REQ-005
- **BDD-JRN-014-02 — Atualizar schedule**
  - Dado que há um schedule ativo
  - Quando o desenvolvedor alterar sua configuração pela CLI
  - Então a mudança é persistida e visível pela web/API.
  - Rastreabilidade: JRN-014; REQ-001, REQ-002, REQ-004, REQ-005
- **BDD-JRN-014-03 — Controlar estado**
  - Dado que há um schedule pausável
  - Quando o desenvolvedor pausar e depois retomar pela CLI
  - Então cada estado confirmado aparece na CLI e na API.
  - Rastreabilidade: JRN-014; REQ-001, REQ-002, REQ-004, REQ-005
- **BDD-JRN-014-04 — Cancelar schedule**
  - Dado que há um schedule ativo
  - Quando o desenvolvedor cancelar pela CLI
  - Então o estado cancelado é confirmado e não são criadas novas ocorrências agendadas.
  - Rastreabilidade: JRN-014; REQ-001, REQ-002, REQ-004, REQ-005
- **BDD-JRN-014-05 — Atualizar parâmetros com gatilho**
  - Dado que o gatilho está ativo
  - Quando o desenvolvedor atualizar parâmetros válidos pela CLI
  - Então uma execução é disparada conforme a configuração do schedule.
  - Rastreabilidade: JRN-014; REQ-001, REQ-002, REQ-004, REQ-005
- **BDD-JRN-014-06 — Entrada inválida**
  - Dado que a alteração é inválida
  - Quando o desenvolvedor tentar aplicá-la
  - Então recebe erro e a configuração vigente permanece inalterada.
  - Rastreabilidade: JRN-014; REQ-001, REQ-002, REQ-004, REQ-005

### JRN-015 — Controlar execuções e consultar resultados pela CLI

- Canal: CLI
- Requisitos: REQ-001, REQ-002, REQ-006
- Ator, gatilho e pré-requisitos: desenvolvedor individual; quer disparar, acompanhar ou recuperar uma execução; daemon local disponível.
- Resultado: ocorrência/execução consultada ou controlada pela CLI com resultado alinhado à API.
- Inclusão no MVP e justificativa: incluída; controle de execução e recuperação fazem parte do gerenciamento CLI solicitado.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Dispara execução imediata ou lista ocorrências/histórico de um schedule. | CLI apresenta IDs, origem, estado, horários e resultado disponível. | Execução imediata não altera o schedule; concorrência permitida segue a regra do produto. | Desenvolvedor acompanha execuções sem acessar a UI. |
| 2 | Consulta detalhe/saídas, cancela execução ativa ou solicita retry de ocorrência falha/interrompida. | CLI apresenta código de saída, stdout/stderr e truncamento, ou estado/erro da ação. | Retry de falha segue política; execução interrompida exige confirmação explícita; cancelamento distingue solicitação de término efetivo. | Operações suportadas pela API de ocorrências são acessíveis na CLI com confirmação quando requerida. |

#### Caminhos de exceção e recuperação

- Execução sem falha não pode ser repetida como recuperação; erro informa estado e preserva resultado original.
- Saídas são apresentadas de modo seguro e respeitam limite/truncamento de 1 MiB por stream.
- Se o processo ainda não encerrou após cancelamento, CLI indica que o pedido foi aceito sem afirmar término concluído.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-015-01 — Executar agora e consultar**
  - Dado que há um schedule válido
  - Quando o desenvolvedor disparar agora e consultar o ID retornado
  - Então a ocorrência e seu resultado ficam visíveis pela CLI/API.
  - Rastreabilidade: JRN-015; REQ-001, REQ-002, REQ-006
- **BDD-JRN-015-02 — Consultar saída**
  - Dado que uma execução terminou
  - Quando o desenvolvedor consultar o detalhe
  - Então vê estado, código de saída e stdout/stderr com sinalização de truncamento.
  - Rastreabilidade: JRN-015; REQ-001, REQ-002, REQ-006
- **BDD-JRN-015-03 — Cancelar execução**
  - Dado que uma execução está em andamento
  - Quando o desenvolvedor pedir cancelamento
  - Então a CLI apresenta o estado do pedido sem confundir solicitação com parada concluída.
  - Rastreabilidade: JRN-015; REQ-001, REQ-002, REQ-006
- **BDD-JRN-015-04 — Repetir falha**
  - Dado que uma ocorrência falhou
  - Quando o desenvolvedor solicitar retry
  - Então uma nova tentativa fica consultável.
  - Rastreabilidade: JRN-015; REQ-001, REQ-002, REQ-006
- **BDD-JRN-015-05 — Confirmar retry interrompido**
  - Dado que uma ocorrência foi interrompida no reinício
  - Quando o desenvolvedor confirmar explicitamente o retry
  - Então uma nova tentativa começa; sem confirmação, não começa.
  - Rastreabilidade: JRN-015; REQ-001, REQ-002, REQ-006
