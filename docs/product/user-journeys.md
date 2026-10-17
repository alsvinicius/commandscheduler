# Command Scheduler — Jornadas de usuário

- Status: ready
- Revisão: 34
- Histórico de revisões: [registro central](../revision-history.md) — seção `product/user-journeys.md`
- Atualizado: 2026-10-13
- Fontes e revisões: [PRD r19](prd.md), [requisitos r5](requirements.md), [definição técnica r61](../engineering/technical-definition.md)
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.
- Artefatos relacionados: [estado do workflow](workflow-state.md), [índice de documentação](../README.md)

## Personas e contexto

- **Desenvolvedor individual:** quer migrar cron e scripts espalhados para schedules locais que possa visualizar e controlar pela CLI e interface web.
- **Aplicação integradora local:** usa a API local para criar schedules, atualizar parâmetros e consultar/controlar execuções.

Os atores e objetivos refletem o PRD r17. Cada jornada identifica seu canal (CLI, web ou API) e um resultado útil, sem transformar cada clique em jornada isolada.

## Jornadas

Os detalhes e cenários BDD ficam em documentos por canal; este arquivo é o índice e a matriz de cobertura.

### Índice — Web

Detalhes e cenários: [jornadas web](journeys/web.md).

- JRN-001 — Criar schedule.
- JRN-004 — Consultar estado e resultado de execução.
- JRN-005 — Manter schedule.
- JRN-008 — Executar agora ou cancelar execução.
- JRN-009 — Recuperar execução com falha.
- JRN-016 — Acompanhar operação, agenda e histórico.

### Índice — API

Detalhes e cenários: [índice das jornadas da API](journeys/api-index.md).

- JRN-002 — Criar schedule.
- JRN-003 — Atualizar parâmetros para execução agendada.
- JRN-006 — Manter schedule.
- JRN-007 — Disparar execução após atualização de parâmetros.
- JRN-010 — Consultar execuções.
- JRN-011 — Executar agora ou cancelar execução.
- JRN-012 — Recuperar execução com falha.

### Índice — CLI

Detalhes e cenários: [jornadas da CLI](journeys/cli.md).

- JRN-013 — Criar schedule pelo terminal.
- JRN-014 — Consultar e manter schedules pela CLI.
- JRN-015 — Controlar execuções e consultar resultados pela CLI.

## Cobertura de requisitos

| Requisito | Jornadas ou classificação de restrição | Lacunas |
|---|---|---|
| REQ-001 — Gerenciar agendamentos de comandos | JRN-001, JRN-004, JRN-005, JRN-008, JRN-009, JRN-010, JRN-011, JRN-012, JRN-013–JRN-015 | Coberto por CLI/web/API; decisões comportamentais estão definidas nos artefatos fonte. |
| REQ-002 — Gerenciar por web, API e CLI | JRN-001, JRN-002, JRN-004, JRN-005, JRN-006, JRN-008–JRN-015 | Ciclo de gerenciamento coberto nos três canais; OpenAPI detalhado é entrega de implementação. |
| REQ-003 — Aceitar agendamentos de aplicações externas | JRN-002 | API REST/JSON e idempotência cobertas; contrato completo é entrega da tarefa OpenAPI. |
| REQ-004 — Aplicar parâmetros externos | JRN-002, JRN-003 | JSON válido até 64 KiB, qualquer tipo, está definido na definição técnica. |
| REQ-005 — Executar após atualização | JRN-007 | Gatilho pode substituir ou coexistir com o horário por schedule, conforme PRD. |
| REQ-006 — Repetir falhas | JRN-005, JRN-009, JRN-012 | Política padrão e configuração por schedule cobertas; detalhes do contrato ficam na tarefa OpenAPI. |
| REQ-007 — Testar comando na criação web | JRN-001 | Execução transitória e separação visível entre executável e argumentos. |
| REQ-008 — Configurar recorrência por presets | JRN-001 | Presets na UI web; API mantém contrato cron atual. |
| REQ-009 — Exibir cockpit operacional | JRN-016 | Agregados do período e saúde do daemon definidos na definição técnica. |
| REQ-010 — Consultar agenda global | JRN-016 | Agenda local dia/semana/mês; contrato de consulta definido para implementação. |
| REQ-011 — Filtrar schedules e histórico extenso | JRN-016 | Filtros e paginação global definidos para implementação. |

## Decisões confirmadas e encerramento de itens

### Confirmado

- Atores e objetivos foram derivados do PRD r17: desenvolvedor individual e aplicação integradora local.
- As regras detalhadas de produto estão no [catálogo de requisitos](requirements.md); as decisões técnicas estão na [definição técnica](../engineering/technical-definition.md) e em seus registros por tema.
- As 16 jornadas e seus 100 cenários BDD estão nos arquivos por canal, com IDs estáveis e rastreabilidade para requisitos.

### Itens sem decisão pendente

- Limite de volume/carga e conformidade formal de acessibilidade estão fora das metas do MVP; os requisitos básicos de acessibilidade permanecem vigentes.
- A métrica de schedules migrados será observada de forma descritiva no MVP, sem meta numérica ou período. Não se deve declarar sucesso contra um alvo inexistente; reavaliar a meta após observar os resultados do MVP.
- Paginação e códigos HTTP são escolhas de implementação do contrato OpenAPI dentro das restrições técnicas; devem ser registrados em TASK-002, sem checkpoint adicional de produto.
