# Estado do workflow de produto

- Produto / repositório: Command Scheduler / [alsvinicius/commandscheduler](https://github.com/alsvinicius/commandscheduler)
- Atualizado: 2026-10-16
- Revisão: 71
- Histórico de revisões: [registro central](../revision-history.md) — seção `product/workflow-state.md`
- Idioma deste conjunto: português, idioma das entrevistas registradas; novos workflows seguem o idioma usado pelo usuário ativo.
- Passo atual: `/mvp` em andamento; TASK-016 (BDD), TASK-017 e TASK-018 implementadas e verificadas em Linux.
- Escopo solicitado nesta revisão: implementar a ampliação aprovada em TASK-017/018 e completar os BDDs pendentes de TASK-016; validação de implementação somente em Linux.
- Atividade atual: TASK-016, TASK-017 e TASK-018 concluídas com build, API, docs e BDDs Linux; próximos trabalhos são a janela mensal TASK-011 e os passos Linux ainda possíveis de TASK-012. Gates de release em macOS/Windows permanecem preservados.
- Última ação concluída: o repositório público `alsvinicius/commandscheduler` foi confirmado e o vínculo Git local foi restaurado na branch `main`, acompanhando `origin/main`. README e procedimento de publicação foram alinhados ao repositório existente. A revisão open source de 2026-10-13 permanece como registro histórico; a CI remota e a execução nativa macOS/Windows continuam pendentes.
- Próxima ação registrada: manter o coletor TASK-011 ativo até completar observação mensal; seguir com TASK-012 usando Linux onde aplicável; manter os gates nativos macOS/Windows e CI tripla para release.
- Próxima ação recomendada: continuar `/mvp` com observação TASK-011 e validações Linux disponíveis de TASK-012; depois fazer revisão dos critérios de release, sem declarar release pronto antes dos gates multiplataforma.

## Checkpoint

- PRD r19, requisitos r5, jornadas r34, definição técnica r61 e plano r59 incluem REQ-008–REQ-011; pacote API/web r35 registra TASK-016/017/018 concluídas em Linux.
- As 16 jornadas têm 100 cenários BDD identificados como `BDD-JRN-NNN-NN`, estruturados em Dado/Quando/Então e ligados a requisitos; 100/100 estão ligados a verificações executáveis após build/BDD Linux; CI de Windows/macOS/Linux continua como gate de release.
- As jornadas permanecem em quatro documentos detalhados (Web, API schedules, API execuções e CLI), mais o índice do canal API. As tarefas permanecem em três pacotes com entradas, limites de autonomia, cenários BDD, validação e condições de escalonamento; TASK-017/018 estendem o pacote API/web.
- Decisões DEC-001–DEC-036 foram movidas para três registros temáticos sem alterar IDs; DEC-037 escolhe cron de cinco campos, DEC-038 registra SQLite e DEC-039 define a política na retomada após pausa. DEC-041 registra presets por relógio/calendário local, confirmados pelo usuário; DEC-042 define consultas globais para o cockpit. DEC-027 usa o nome público `missedOccurrences`, conforme o DTO e a UI. A métrica de migração é observacional, sem meta numérica ou período no MVP; volume/carga e conformidade formal de acessibilidade não são metas do MVP.
- TASK-001 concluída com build/link e smoke em Linux; `cargo check` cruzado também passou para Windows GNU e macOS x86_64. Linkagem/execução nativa desses alvos e cenários BDD continuam pendentes para o gate de release, sem bloquear desenvolvimento Linux.

## Artefatos

| Artefato | Caminho | Revisão | Status | Fontes | Bloqueios |
|---|---|---:|---|---|---|
| PRD | [product/prd.md](prd.md) | 19 | ready | descoberta e escolhas do usuário; AGENTS.md | nenhum |
| Requisitos | [product/requirements.md](requirements.md) | 5 | ready | PRD r19; escolhas do usuário | nenhum |
| Índice de jornadas | [product/user-journeys.md](user-journeys.md) | 34 | ready | PRD r19; requisitos r5; definição técnica r61 | CI tripla permanece gate de release |
| Jornadas web | [product/journeys/web.md](journeys/web.md) | 9 | ready | PRD r19; requisitos r5; definição técnica r61 | CI tripla permanece gate de release |
| Índice de jornadas API | [product/journeys/api-index.md](journeys/api-index.md) | 1 | ready | PRD r17; requisitos r1 | nenhum |
| Jornadas API — schedules | [product/journeys/api-schedules.md](journeys/api-schedules.md) | 3 | ready | PRD r17; requisitos r1; definição técnica r55 | nenhum |
| Jornadas API — execuções | [product/journeys/api-executions.md](journeys/api-executions.md) | 2 | ready | PRD r17; requisitos r1; definição técnica r55 | nenhum |
| Jornadas CLI | [product/journeys/cli.md](journeys/cli.md) | 3 | ready | PRD r17; requisitos r1; definição técnica r55 | nenhum |
| Definição técnica | [engineering/technical-definition.md](../engineering/technical-definition.md) | 61 | ready | PRD r19; requisitos r5; jornadas r34; DEC-041–DEC-042 | nenhum |
| Decisões técnicas | [runtime e entrega](../engineering/decisions/runtime-and-delivery.md), [agendamento e execução](../engineering/decisions/scheduling-and-execution.md), [interfaces e qualidade](../engineering/decisions/interfaces-and-quality.md) | 3; 4; 8 | ready | PRD r19; requisitos r5; jornadas r33; DEC-037–DEC-042 | nenhum |
| Plano de implementação | [planning/implementation-plan.md](../planning/implementation-plan.md) | 59 | ready | PRD r19; requisitos r5; jornadas r34; definição técnica r61 | gates multiplataforma antes do release |
| Pacotes de tarefas | [base e runtime](../planning/tasks/foundation.md), [API e web](../planning/tasks/api-and-web.md), [verificação, docs e CLI](../planning/tasks/verification-docs-and-cli.md) | base e runtime r38; API/web r35; verificação/docs/CLI r49 | TASK-001–TASK-010 e TASK-013–TASK-018 implementadas/validadas em Linux; TASK-011 observação mensal em curso; TASK-012 validações nativas pendentes | PRD r19; requisitos r5; jornadas r34; definição técnica r61; plano r59 | gates multiplataforma antes do release |

## Decisões e encerramentos

- Fase 1 entrega o ciclo local por CLI, web e API, com execução e recuperação nos três sistemas operacionais.
- Agentes podem selecionar escolhas reversíveis de implementação dentro dos critérios documentados; decisões de produto, escopo, segurança, compatibilidade, custo material e ações irreversíveis escalam ao usuário.
- Build, formatação e lint locais têm comandos definidos em `CONTRIBUTING.md`; durante a implementação, validações locais podem ocorrer somente em Linux. GitHub Actions está configurado para os três SOs e inclui a suíte Python, os testes Rust, 17 testes de browser e o medidor de pontualidade; a execução remota ainda não foi comprovada nesta cópia sem metadados Git. Dos 100 cenários BDD agora registrados, 100/100 estão ligados a verificações executáveis e passaram no desenvolvimento Linux; execução da matriz CI tripla não foi comprovada nesta cópia. A medição Linux dedicada em host ocioso passou com 100/100 inícios, zero falhas, P95 0,2505 s e P99 0,269469 s após CPU entre 95,29% e 96,08% ociosa em 60 s. O coletor Linux foi validado com amostras saudáveis, falhas, suspensão, ajuste de relógio, reinício e lacunas; a unit systemd foi validada. O coletor mensal está ativo desde 2026-10-13 15:31:55 UTC, mas ainda não há janela completa. Gates nativos macOS/Windows e CI BDD tripla permanecem para release.
- Validação somente Linux está autorizada durante implementação; compatibilidade nativa e cenários BDD nos três sistemas continuam obrigatórios antes do release.
- O conjunto existente permanece em português por derivar de entrevistas de produto conduzidas nesse idioma. A skill agora exige que novos workflows sigam o idioma usado pelo usuário ativo.
- Métrica delimitada: schedules migrados serão observados sem meta ou janela no MVP; revisar a necessidade de alvo depois de observar os resultados. O impacto e a fonte estão registrados no PRD.

## Atualizações downstream pendentes

A semântica de retomada aplica a política salva aos horários vencidos (DEC-039). Serviço e coletor systemd foram instalados e validados no Linux; a observação mensal continua aberta e a medição de pontualidade ociosa passou no Linux. A reprodução nativa macOS/Windows e a execução comprovada de todos os BDDs na matriz tripla permanecem gates de release, sem bloquear implementação Linux.

## Próximo passo

Continuar `/mvp` com observação TASK-011 e validações Linux disponíveis de TASK-012. A execução nativa Windows/macOS e a matriz CI BDD tripla permanecem gates para revisão de release.
