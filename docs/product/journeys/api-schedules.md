# Jornadas da API — schedules e parâmetros

- Status: ready
- Revisão: 3
- Histórico de revisões: [registro central](../../revision-history.md) — seção `product/journeys/api-schedules.md`
- Atualizado: 2026-10-13
- Fontes e revisões: [PRD r17](../prd.md), [requisitos r1](../requirements.md), [definição técnica r48](../../engineering/technical-definition.md)
- Artefatos relacionados: [índice de jornadas](../user-journeys.md), [estado do workflow](../workflow-state.md)
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.

### JRN-002 — Criar um schedule pela API local

- Canal: API
- Requisitos: REQ-002, REQ-003, REQ-004
- Ator, gatilho e pré-requisitos: aplicação integradora na mesma máquina; precisa registrar um comando e seus dados iniciais; API local disponível.
- Resultado: schedule criado e resultado identificável pela aplicação integradora.
- Inclusão no MVP e justificativa: incluída; permite que sistemas locais deleguem o agendamento à plataforma.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Envia comando, programação, parâmetros iniciais e política para horários vencidos à API. | A API valida a solicitação e informa se foi aceita ou rejeitada. | No MVP, API sem autenticação, acessível somente por localhost; a política de ocorrência vencida é configurável por schedule. | Aplicação local consegue criar schedule sem autenticação, definir política para indisponibilidade e distinguir sucesso de rejeição. |
| 2 | Em caso de rejeição, corrige os dados e envia novamente. | A API informa os erros identificáveis e o resultado da nova solicitação. | Dados inválidos não deixam schedule ativo. | A aplicação pode corrigir o pedido e confirmar a criação válida. |

#### Caminhos de exceção e recuperação

- Se não houver resposta por indisponibilidade, a aplicação não considera a criação confirmada e pode consultar ou tentar novamente.
- Chamadas de fora da máquina não são aceitas no MVP.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-002-01 — Criação aceita**
  - Dado que a API local está disponível e a solicitação é válida
  - Quando a aplicação enviar comando, programação e parâmetros
  - Então recebe confirmação e identificação consultável do schedule.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-02 — Criação idempotente**
  - Dado que a aplicação envia `Idempotency-Key` opcional
  - Quando repetir a mesma criação com a mesma chave e conteúdo
  - Então a API retorna o schedule originalmente criado sem duplicá-lo.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-03 — Referência OpenAPI**
  - Dado que o desenvolvedor abre a referência da API na UI local
  - Quando consultar a especificação
  - Então encontra OpenAPI 3.1 com schemas, erros e exemplos correspondentes à API implementada.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-04 — Aceitar qualquer JSON**
  - Dado que a aplicação envia objeto, array, string, número, booleano ou null válido de até 64 KiB
  - Quando criar o schedule
  - Então os parâmetros são aceitos e disponibilizados no processo.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-05 — Parâmetros inválidos ou grandes**
  - Dado que a aplicação envia JSON inválido ou payload acima de 64 KiB
  - Quando criar o schedule
  - Então a API rejeita sem criar schedule ativo.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-06 — Conflito de chave**
  - Dado que a aplicação envia uma chave já usada com conteúdo diferente
  - Quando tentar criar outro schedule
  - Então a API rejeita a solicitação como conflito.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-07 — Parâmetros JSON no processo**
  - Dado que o schedule tem parâmetros JSON
  - Quando a ocorrência começar
  - Então o comando recebe o objeto em `COMMAND_SCHEDULER_PARAMS_JSON` sem interpolação no comando.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-08 — Política configurada na criação**
  - Dado que a solicitação inclui uma política válida para horários vencidos
  - Quando a aplicação criar o schedule
  - Então a API confirma e persiste a política escolhida.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-09 — Solicitação rejeitada**
  - Dado que a solicitação contém dados inválidos
  - Quando for enviada
  - Então recebe erro identificável e nenhum schedule ativo é criado.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-10 — API indisponível**
  - Dado que não há resposta da API
  - Quando a aplicação aguardar o resultado
  - Então não considera a criação confirmada.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-11 — Consolidar excedentes**
  - Dado que o schedule usa política sequencial e acumulou mais de 100 ocorrências
  - Quando o serviço retornar
  - Então até 100 ocorrências ficam enfileiradas individualmente e o excedente é consolidado em uma execução usando os parâmetros mais recentes.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-12 — Metadados consolidados**
  - Dado que uma execução consolida ocorrências vencidas
  - Quando o processo começar
  - Então recebe quantidade e limites temporais por variáveis de ambiente e a API retorna os mesmos dados estruturados.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-13 — Comando estruturado**
  - Dado que a aplicação envia um programa e lista de argumentos
  - Quando a API criar o schedule
  - Então persiste e executa esses campos separadamente sem shell implícito.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-14 — Origem web não autorizada**
  - Dado que uma página de outra origem tenta chamar a API localhost pelo navegador
  - Quando a requisição chegar
  - Então ela é rejeitada sem CORS amplo.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-15 — BDD multiplataforma**
  - Dado que há cenários BDD vinculados às jornadas
  - Quando a CI executar a suíte
  - Então os cenários automatizados rodam em Windows, macOS e Linux.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-16 — Horário inexistente ou repetido**
  - Dado que uma recorrência coincide com avanço ou recuo do relógio local
  - Quando a API consultar o schedule ou o serviço calcular a ocorrência
  - Então horário inexistente é pulado e o horário repetido gera somente uma execução.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004
- **BDD-JRN-002-17 — Atraso sem reinício**
  - Dado que uma ocorrência vence durante suspensão/atraso temporário sem reinício do daemon
  - Quando o processamento retomar
  - Então a API e scheduler aplicam a política configurada no schedule.
  - Rastreabilidade: JRN-002; REQ-002, REQ-003, REQ-004

### JRN-003 — Atualizar parâmetros para uma execução agendada

- Canal: API
- Requisitos: REQ-004
- Ator, gatilho e pré-requisitos: aplicação integradora local; schedule existente; novos valores disponíveis antes do começo da próxima execução.
- Resultado: parâmetros vigentes atualizados para a próxima execução elegível.
- Inclusão no MVP e justificativa: incluída; atende à necessidade de preparar dados por aplicação externa.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Envia novos parâmetros ao schedule pela API. | A API confirma a atualização ou informa a rejeição. | Atualizações são aceitas até o começo da execução. | A aplicação recebe confirmação clara do resultado da atualização. |
| 2 | Aguarda a próxima ocorrência por horário quando não há gatilho imediato configurado. | A plataforma inicia a execução no horário previsto. | Se não houver atualização nova, são usados os últimos parâmetros disponíveis. | A ocorrência usa os parâmetros mais recentes aceitos antes de seu início. |

#### Caminhos de exceção e recuperação

- Atualização rejeitada não substitui os valores vigentes.
- Atualização recebida após o início não altera a execução em curso; pode valer para execução posterior.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-003-01 — Parâmetros atualizados**
  - Dado que existe schedule sem gatilho imediato e ainda não começou a próxima execução
  - Quando a aplicação atualizar os parâmetros
  - Então recebe confirmação e a próxima execução usa os novos valores.
  - Rastreabilidade: JRN-003; REQ-004
- **BDD-JRN-003-02 — Atualização tardia**
  - Dado que a execução já começou
  - Quando chegar uma atualização de parâmetros
  - Então ela não altera a execução em andamento.
  - Rastreabilidade: JRN-003; REQ-004
- **BDD-JRN-003-03 — Atualização rejeitada**
  - Dado que a atualização não pode ser aceita
  - Quando a aplicação a enviar
  - Então recebe rejeição e os parâmetros vigentes permanecem iguais.
  - Rastreabilidade: JRN-003; REQ-004
- **BDD-JRN-003-04 — JSON e limite na atualização**
  - Dado que a aplicação atualiza parâmetros com qualquer JSON válido até 64 KiB
  - Quando enviar a atualização
  - Então ela é aceita; JSON inválido ou acima do limite é rejeitado e os parâmetros anteriores permanecem vigentes.
  - Rastreabilidade: JRN-003; REQ-004

### JRN-006 — Manter um schedule pela API local

- Canal: API
- Requisitos: REQ-002
- Ator, gatilho e pré-requisitos: aplicação integradora local; schedule existente; API local disponível.
- Resultado: aplicação consulta ou altera o schedule sem depender da interface web.
- Inclusão no MVP e justificativa: incluída; gerenciamento externo deve ser possível por API, além da criação.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Consulta ou altera comando, horário, parâmetros, gatilho por atualização, política de repetição automática ou política para horários vencidos. | A API retorna dados atuais, confirmação ou motivos de rejeição. | As ações de gerenciamento disponíveis pela API acompanham a capacidade suportada pelo produto na interface. Alterações de parâmetros respeitam o início da execução; política de indisponibilidade é por schedule. | A aplicação local consegue consultar e alterar esses dados do schedule pela API. |
| 2 | Pausa, retoma ou cancela o schedule. | A API confirma a mudança de estado ou informa a rejeição. | Schedule pausado não dispara por horário; cancelado não gera novos disparos. | A aplicação consegue pausar/retomar e cancelar, consultando o estado final. |

#### Caminhos de exceção e recuperação

- Se a solicitação falhar ou não houver resposta, a aplicação não assume que a mudança ocorreu; pode consultar o estado e tentar novamente.
- Estado ou ação inválida é informado sem confirmar uma transição não realizada.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-006-01 — Alterar schedule pela API**
  - Dado que a aplicação local identifica um schedule existente
  - Quando consultar ou enviar alteração válida
  - Então a API retorna estado atual ou confirmação da configuração atualizada.
  - Rastreabilidade: JRN-006; REQ-002
- **BDD-JRN-006-02 — Pausar, retomar e cancelar**
  - Dado que o schedule está ativo
  - Quando a aplicação solicitar uma dessas transições
  - Então a API confirma o novo estado, não há disparos por horário durante a pausa e, ao retomar, horários vencidos seguem a política salva.
  - Rastreabilidade: JRN-006; REQ-002
- **BDD-JRN-006-03 — Falha sem confirmação**
  - Dado que a solicitação não recebe resposta conclusiva
  - Quando a aplicação consultar novamente
  - Então pode determinar o estado atual antes de repetir a ação.
  - Rastreabilidade: JRN-006; REQ-002
- **BDD-JRN-006-04 — Política de indisponibilidade**
  - Dado que o schedule existe
  - Quando a aplicação atualizar a política de horários vencidos
  - Então a API confirma e retorna o valor configurado.
  - Rastreabilidade: JRN-006; REQ-002
- **BDD-JRN-006-05 — Opções de recuperação**
  - Dado que a aplicação altera a política de um schedule
  - Quando envia pular, consolidar em uma execução ou executar todas em sequência
  - Então a API confirma e retorna a opção aplicada.
  - Rastreabilidade: JRN-006; REQ-002

- **BDD-JRN-006-06 — Recuperar horários vencidos ao retomar**
  - Dado que horários venceram enquanto o schedule estava pausado
  - Quando a aplicação retomar o schedule
  - Então esses horários são tratados pela política de recuperação salva nele.
  - Rastreabilidade: JRN-006; REQ-002; DEC-039

### JRN-007 — Disparar uma execução após atualização de parâmetros

- Canal: API
- Requisitos: REQ-005
- Ator, gatilho e pré-requisitos: aplicação integradora local; schedule existente com gatilho por atualização habilitado; novos parâmetros disponíveis.
- Resultado: uma execução começa imediatamente com os parâmetros atualizados.
- Inclusão no MVP e justificativa: incluída; permite que uma aplicação dispare trabalho ao concluir a preparação dos parâmetros.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Atualiza os parâmetros pela API. | A API confirma que os parâmetros foram aceitos. | Cada atualização aceita dispara uma execução quando esse gatilho está ativo. | A atualização aceita inicia execução sem esperar o próximo horário. |
| 2 | Consulta o estado da execução iniciada. | A plataforma apresenta estado/resultados disponíveis. | O gatilho pode, por schedule, substituir ou coexistir com o horário; execuções do mesmo schedule podem ocorrer em paralelo. | A aplicação pode distinguir a atualização aceita e a execução que ela disparou. |

#### Caminhos de exceção e recuperação

- Atualização rejeitada não inicia execução.
- Uma execução já iniciada mantém os valores com que começou; atualização posterior pode iniciar outra execução se o gatilho continuar ativo.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-007-01 — Gatilho ativo**
  - Dado que o schedule está configurado para executar após atualização
  - Quando a API aceitar novos parâmetros
  - Então uma execução começa imediatamente com esses valores.
  - Rastreabilidade: JRN-007; REQ-005
- **BDD-JRN-007-02 — Gatilho inativo**
  - Dado que o gatilho por atualização está desabilitado
  - Quando a API aceitar novos parâmetros
  - Então não inicia execução por essa atualização e os valores ficam disponíveis para o próximo horário.
  - Rastreabilidade: JRN-007; REQ-005
- **BDD-JRN-007-03 — Parâmetros rejeitados**
  - Dado que a API rejeita a atualização
  - Quando o gatilho estiver ativo
  - Então nenhuma execução é iniciada por essa solicitação.
  - Rastreabilidade: JRN-007; REQ-005
