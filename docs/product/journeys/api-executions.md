# Jornadas da API — execuções e recuperação

- Status: ready
- Revisão: 2
- Histórico de revisões: [registro central](../../revision-history.md) — seção `product/journeys/api-executions.md`
- Atualizado: 2026-10-13
- Fontes e revisões: [PRD r17](../prd.md), [requisitos r1](../requirements.md), [definição técnica r48](../../engineering/technical-definition.md)
- Artefatos relacionados: [índice de jornadas](../user-journeys.md), [estado do workflow](../workflow-state.md)
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.

### JRN-010 — Consultar execuções pela API local

- Canal: API
- Requisitos: REQ-001, REQ-002
- Ator, gatilho e pré-requisitos: aplicação integradora local; schedule criado; precisa verificar o estado ou resultado de uma ocorrência.
- Resultado: cliente local conhece o estado e resultado disponíveis pela API.
- Inclusão no MVP e justificativa: incluída; a API deve permitir controle de execução além do cadastro.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Consulta pela API as execuções do schedule ou uma ocorrência específica. | A API retorna os estados e resultados disponíveis ou indica que não há ocorrências. | Histórico de execuções retido por 90 dias. | A aplicação consegue distinguir execuções em andamento, concluídas e falhas dentro do período de retenção. |

#### Caminhos de exceção e recuperação

- Se a API não conseguir consultar estado, retorna falha observável; a aplicação não interpreta falha como execução concluída.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-010-01 — Consulta de resultado**
  - Dado que o schedule possui ocorrências
  - Quando a aplicação consultar pela API local
  - Então recebe estados/resultados disponíveis.
  - Rastreabilidade: JRN-010; REQ-001, REQ-002
- **BDD-JRN-010-02 — Sem ocorrências**
  - Dado que ainda não há ocorrências
  - Quando a aplicação consultar
  - Então recebe resposta que indica ausência de resultados.
  - Rastreabilidade: JRN-010; REQ-001, REQ-002
- **BDD-JRN-010-03 — Saídas e retenção**
  - Dado que uma ocorrência produziu `stdout` e `stderr`
  - Quando a aplicação consultar dentro de 90 dias
  - Então recebe as saídas capturadas até 1 MiB cada e indicação de truncamento se aplicável.
  - Rastreabilidade: JRN-010; REQ-001, REQ-002

### JRN-011 — Executar agora ou cancelar execução pela API

- Canal: API
- Requisitos: REQ-001, REQ-002
- Ator, gatilho e pré-requisitos: aplicação integradora local; schedule existente ou ocorrência ativa; precisa iniciar ou cancelar uma execução pela API.
- Resultado: execução manual iniciada ou pedido de cancelamento submetido e consultável.
- Inclusão no MVP e justificativa: incluída; mantém equivalência de controle entre interface e API.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Solicita executar agora ou cancelar ocorrência ativa pela API local. | A API confirma o início ou pedido aceito, ou informa por que a ação não foi aplicada. | Execuções manuais podem coexistir; cancelar ocorrência não cancela o schedule. | A aplicação consegue solicitar execução imediata e cancelamento de uma ocorrência em andamento. |
| 2 | Consulta o estado da ocorrência pela API. | A API mostra se iniciou, terminou ou ainda pode estar em andamento. | Pedido de cancelamento não confirma que o processo já parou. | A aplicação distingue solicitação aceita de cancelamento concluído. |

#### Caminhos de exceção e recuperação

- Sem execução ativa, o cancelamento não é aplicado e o estado atual é retornado.
- Se não for possível iniciar agora, a API informa falha sem indicar execução iniciada.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-011-01 — Execução imediata**
  - Dado que o schedule pode executar
  - Quando a aplicação solicitar execução agora
  - Então a API confirma início ou retorna o estado observável da execução.
  - Rastreabilidade: JRN-011; REQ-001, REQ-002
- **BDD-JRN-011-02 — Concorrência sem limite global**
  - Dado que várias execuções foram aceitas e há recursos do sistema
  - Quando a API receber outra solicitação válida
  - Então o scheduler não rejeita apenas por um limite global configurado no produto.
  - Rastreabilidade: JRN-011; REQ-001, REQ-002
- **BDD-JRN-011-03 — Cancelamento solicitado**
  - Dado que há ocorrência ativa
  - Quando a aplicação solicitar cancelamento
  - Então a API confirma o pedido e permite consultar se ela continua.
  - Rastreabilidade: JRN-011; REQ-001, REQ-002
- **BDD-JRN-011-04 — Sem execução ativa**
  - Dado que não há ocorrência ativa
  - Quando a aplicação solicitar cancelamento
  - Então recebe estado atual sem uma falsa confirmação de cancelamento.
  - Rastreabilidade: JRN-011; REQ-001, REQ-002

### JRN-012 — Recuperar execução com falha pela API

- Canal: API
- Requisitos: REQ-001, REQ-002, REQ-006
- Ator, gatilho e pré-requisitos: aplicação integradora local; ocorrência falhou ou política automática está configurada.
- Resultado: nova tentativa iniciada conforme pedido ou política, com seu estado consultável.
- Inclusão no MVP e justificativa: incluída; a API deve oferecer os mesmos controles de recuperação que a interface.

| Etapa | Intenção/entrada/ação do usuário | Resposta observável | Regras de negócio | Critérios de aceitação |
|---|---|---|---|---|
| 1 | Configura política automática ou solicita repetição manual pela API. | A API confirma política salva ou informa que nova tentativa começou/rejeitou. | Repetição manual e automática configurável são suportadas; padrão são três retries além da execução inicial, em 1, 5 e 15 minutos. | A aplicação consegue configurar repetição automática e solicitar repetição de falha. |
| 2 | Consulta o resultado da nova tentativa pela API. | A API retorna o estado disponível da tentativa. | Execuções concorrentes são permitidas. | A aplicação consegue distinguir a tentativa original e a nova tentativa. |

#### Caminhos de exceção e recuperação

- Ocorrência que não falhou não pode ser repetida como recuperação; a API informa o estado atual.
- Se a repetição não puder começar, a API informa falha observável e preserva o resultado original.

#### Cenários BDD (Dado/Quando/Então)

- **BDD-JRN-012-01 — Repetição manual**
  - Dado que uma ocorrência terminou com falha
  - Quando a aplicação solicitar repetição pela API
  - Então uma nova tentativa começa e fica consultável.
  - Rastreabilidade: JRN-012; REQ-001, REQ-002, REQ-006
- **BDD-JRN-012-02 — Repetição automática**
  - Dado que existe política automática configurada
  - Quando a ocorrência falhar
  - Então a política determina a nova tentativa e seu estado fica disponível.
  - Rastreabilidade: JRN-012; REQ-001, REQ-002, REQ-006
- **BDD-JRN-012-03 — Ocorrência sem falha**
  - Dado que uma ocorrência não falhou
  - Quando a aplicação solicitar repetição de falha
  - Então recebe rejeição/estado e a execução original permanece inalterada.
  - Rastreabilidade: JRN-012; REQ-001, REQ-002, REQ-006
- **BDD-JRN-012-04 — Execução interrompida**
  - Dado que o serviço reiniciou e encontrou uma execução que estava em andamento
  - Quando a aplicação solicitar repetição
  - Então a API informa estado interrompido e exige confirmação explícita antes da nova tentativa.
  - Rastreabilidade: JRN-012; REQ-001, REQ-002, REQ-006
- **BDD-JRN-012-05 — Retry automático padrão**
  - Dado que uma falha normal ocorreu e o schedule mantém política padrão
  - Quando as tentativas forem aplicadas
  - Então há até três retries nos intervalos de 1, 5 e 15 minutos, exceto se uma tentativa tiver sucesso ou o schedule for alterado/cancelado.
  - Rastreabilidade: JRN-012; REQ-001, REQ-002, REQ-006
- **BDD-JRN-012-06 — Saída truncada**
  - Dado que stdout ou stderr ultrapassa 1 MiB
  - Quando a aplicação consultar a ocorrência
  - Então o conteúdo retornado está limitado por stream e marcado como truncado.
  - Rastreabilidade: JRN-012; REQ-001, REQ-002, REQ-006
