# Command Scheduler — Plano de implementação

- Status: ready
- Revisão: 59
- Histórico de revisões: [registro central](../revision-history.md) — seção `planning/implementation-plan.md`
- Atualizado: 2026-10-13
- Fontes e revisões: [PRD r19](../product/prd.md), [requisitos r5](../product/requirements.md), [jornadas r34](../product/user-journeys.md), [definição técnica r61](../engineering/technical-definition.md)
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.
- Artefatos relacionados: [estado do workflow](../product/workflow-state.md), [índice da documentação](../README.md)

## Resultado do MVP e limite de release

Entregar a um desenvolvedor individual um serviço local instalável manualmente que continue ativo sem o navegador, preserve schedules em SQLite e permita criar e gerenciar schedules/execuções por CLI, UI web e API local. Aplicações locais devem conseguir criar schedules idempotentes, atualizar parâmetros JSON e observar execuções, retries, saídas e recuperação.

O MVP inclui REQ-001–REQ-011 e as jornadas JRN-001–JRN-016. A CLI local cria e gerencia schedules/execuções por meio da API do daemon; web/API também oferecem gerenciamento completo. A interface web testa comandos antes de criar schedules e oferece presets de recorrência, cockpit operacional, agenda e histórico filtrável/paginado. Deve suportar Windows, macOS e Linux nas versões com suporte de segurança do fornecedor no lançamento. O serviço usa permissões do usuário local; comandos são `program` + `args`, sem shell implícito. A API é REST/JSON em localhost, sem autenticação, com chamadas de navegador limitadas à origem da UI.

Release depende de: criação funcional pela CLI e API; gerenciamento de schedules e execuções via CLI, web/API, incluindo atualização, pausa/retomada/cancelamento, execução imediata, consulta de histórico/resultados/saídas e recuperação de falhas; persistência e recuperação após reinício; políticas de ocorrência vencida e de retry conforme schedule; contrato OpenAPI 3.1 navegável; capturas `stdout`/`stderr` limitadas e indicadas quando truncadas; instalação e início automático documentados para os três sistemas; cenários BDD ligados às jornadas executados em CI para Windows/macOS/Linux; verificação da pontualidade P95 ≤ 5 s e P99 ≤ 30 s em máquina ociosa; reinício/recuperação demonstrados e método de medição da meta de disponibilidade mensal de 99,5% registrado.

## Regra temporária de validação durante implementação

Por orientação explícita do usuário (2026-10-13), tarefas de implementação podem ser verificadas usando apenas Linux por enquanto; a falta de hosts Windows/macOS não bloqueia o avanço entre tarefas. Essa exceção vale para a validação de desenvolvimento e não remove nem satisfaz antecipadamente os critérios de release acima: antes do release, ainda será necessário comprovar compatibilidade nativa e executar os cenários BDD em CI para Windows, macOS e Linux, além de reproduzir instalação/uso nos sistemas previstos.

A meta de disponibilidade se aplica ao período em que a máquina está ligada e apta a executar schedules. O percentual mensal é uma meta operacional; não deve ser alegado como atingido sem observação mensurada. Não foi definida carga de referência para pontualidade ou volume.

## Escopo adiado e priorização

- Execução remota/distribuída, acesso pela rede e autenticação ficam fora do MVP; o produto é local e a API aceita clientes somente em localhost.
- Instaladores gráficos/automáticos, distribuição de binários prontos e atualização automática ficam adiados; a instalação do MVP é manual, com documentação por sistema operacional.
- Volume/capacidade, alvo numérico e período para schedules migrados, conformidade formal WCAG, backup/restore guiado e metas de privacidade/regulatórias não fazem parte das decisões do MVP. A adoção será observada descritivamente; revisar eventual alvo após obter resultados. Essas ausências não bloqueiam o release definido.
- Sem limite global de concorrência no MVP. Execuções simultâneas podem elevar consumo local; a UI/API devem refletir execuções independentes e não rejeitar por cota global do produto.
- A especificação OpenAPI deve ser completa em schemas, erros e exemplos. TASK-002 selecionou paginação cursor/limit (padrão 50, máximo 100), códigos HTTP documentados no contrato e cron padrão de cinco campos no fuso local (DEC-037).

## Execução autônoma do plano

Cada tarefa é um pacote executável por agente de código dentro do escopo aprovado. O agente deve ler as fontes e IDs, cumprir as restrições, automatizar os cenários BDD associados, executar ou descobrir os comandos de validação do projeto, registrar evidências e atualizar documentação e estado. Pode tomar decisões reversíveis de implementação dentro dos critérios da tarefa. Deve escalar mudanças de produto, escopo, segurança, compatibilidade, custo material ou ações irreversíveis. Na ausência de comandos conhecidos, primeiro inspeciona arquivos de configuração, README e CI; não inventa comandos.

## Fases

### Fase 1 — MVP

Pacotes de tarefas detalhados:

- [Base e runtime](tasks/foundation.md): TASK-001–TASK-006.
- [API e interface web](tasks/api-and-web.md): TASK-007–TASK-010 e TASK-016–TASK-018.
- [Verificação, documentação e CLI](tasks/verification-docs-and-cli.md): TASK-011–TASK-015.
- TASK-016 (teste transitório de comando na UI) incluída como extensão confirmada do MVP; detalhada em [API e web](tasks/api-and-web.md).

- Valor entregue e IDs incluídos: fluxo completo para desenvolvedor local e aplicações integradoras, cobrindo REQ-001–REQ-011 e JRN-001–JRN-016.
- Dependências: direção técnica descrita em DEC-001–DEC-042 conforme definição técnica r61. TASK-017 dependia das capacidades API/TASK-007–TASK-008; TASK-018 dependia da API/TASK-017 e dos fluxos web/TASK-009–TASK-010. Ambas estão concluídas em Linux. TASK-001–TASK-003 concluíram scaffold, contrato e armazenamento com validação Linux; as validações finais multiplataforma permanecem no gate de release. Capacidade e estimativas da equipe não foram fornecidas; não há datas ou esforço assumidos.
- Critérios de saída: release criteria no início deste plano; todos os cenários BDD rastreáveis às jornadas automatizados no CI em Windows, macOS e Linux; instruções reais de build/instalação reproduzidas por sistema operacional; configuração SQLite sobrevive a reinício e recupera ocorrências de acordo com política; documentação e arquivos obrigatórios atualizados.

### Fase 2 — Operação e expansão conforme demanda

- Valor entregue e IDs incluídos: reduzir custo de operação recorrente além dos controles do MVP; sem requisitos confirmados para execução remota, acesso em rede ou autenticação.
- Dependências: evidência do uso do MVP e nova decisão de produto para cada capacidade ampliada.
- Critérios de saída: cada extensão aprovada mantém compatibilidade com API local e comportamento persistente; plano é revisado antes de assumir requisitos multiusuário/distribuídos.
- Escopo candidato (não comprometido): backup/restore assistido, distribuição/atualização automática, controles de volume/carga, acesso remoto autenticado e execução distribuída. Não iniciar sem validação e requisitos próprios.

## Cobertura de escopo completo

| Requisito / jornada | Fase/tarefas, adiamento ou aposentadoria | Justificativa |
|---|---|---|
| REQ-001; JRN-001, JRN-004–JRN-005, JRN-008–JRN-015 | Fase 1: TASK-003–TASK-006, TASK-008–TASK-015 | Gerir, agendar, executar, consultar e recuperar localmente é o produto central; CLI cobre o ciclo de gestão. |
| REQ-002; JRN-001–JRN-016 | Fase 1: TASK-002, TASK-007–TASK-010, TASK-012–TASK-015, TASK-017–TASK-018 | UI web, API e CLI são canais de primeira classe para o ciclo de gestão local. |
| REQ-003; JRN-002 | Fase 1: TASK-002–TASK-003, TASK-007, TASK-012 | Integração API local com criação idempotente. |
| REQ-004; JRN-002–JRN-003 | Fase 1: TASK-002–TASK-005, TASK-007, TASK-009 | Aplicações devem parametrizar criação/execução com JSON atualizado. |
| REQ-005; JRN-007 | Fase 1: TASK-004–TASK-007 | Gatilho por atualização é diferencial central e exige execução imediata. |
| REQ-006; JRN-005, JRN-009, JRN-012 | Fase 1: TASK-002–TASK-005, TASK-008–TASK-010 | Retry manual e automático configurável são necessários para controle/recuperação. |
| REQ-007; JRN-001 | Fase 1: TASK-016 | Testar comando antes de salvar reduz erros de configuração e revela saídas do processo. |
| REQ-008; JRN-001 | Fase 1: TASK-018 | Presets de calendário reduzem o conhecimento de cron necessário sem remover a opção avançada. |
| REQ-009–REQ-011; JRN-016 | Fase 1: TASK-017–TASK-018 | Cockpit, agenda e consultas paginadas tornam a operação compreensível e utilizável com muitos schedules/execuções. |
| Execução remota/distribuída | Adiada, sem TASK no MVP | Fora do escopo confirmado; muda o limite local e modelo de segurança. |
| API por rede/autenticação | Adiada, sem TASK no MVP | MVP local sem autenticação restrito a localhost. |
| Instaladores/atualização automáticos | Adiados; documentar instalação manual em TASK-006/TASK-012 | Usuário confirmou instalação manual nos três sistemas. |
| Escala formal e limites de carga | Adiados; sem meta confirmada | Não há meta de volume/carga; medir pontualidade apenas em máquina ociosa. |
| Conformidade formal de acessibilidade | Adiada; requisitos básicos em TASK-009/TASK-010 | Decisão expressa é sem certificação, mantendo teclado e rótulos acessíveis. |

## Decisões confirmadas, premissas e itens encerrados

### Confirmado

- PRD r19, requisitos r5, jornadas r34 e definição técnica r61 estão reconciliados; REQ-001–REQ-011 e JRN-001–JRN-016 cobertos por tarefas da Fase 1.
- Stack e restrições de arquitetura/execução, política de recuperação, limites JSON/saída, OpenAPI e metas de qualidade estão na definição técnica; não foram inventados esforço nem prazos.
- Recorrências de intervalo seguem os limites do relógio/calendário local, não o tempo decorrido desde a criação. Os valores de N disponíveis dividem uma hora ou um dia, preservando a cadência ao atravessar a virada; os presets são convertidos para cron e não mudam o contrato público atual de schedules.

### Premissas de planejamento

- Comandos e versões de ferramentas são escolhas de implementação dentro dos limites das fontes; o agente deve descobri-los no repositório ou selecionar opções compatíveis, registrar a decisão e fornecer evidência. A sintaxe de calendário é selecionada em TASK-002 por critérios explícitos. Framework BDD/CI pode ser escolhido pelo agente conforme a stack e os recursos de CI disponíveis, registrando a justificativa e o comando executável.
- DEC-001 está selecionada para o MVP: um daemon Rust coordena scheduler, API, UI servida na mesma origem, armazenamento SQLite e processos filhos; DEC-036 seleciona CLI cliente da API com criação e gerenciamento. Nenhuma validação de arquitetura ou CLI está pendente para as tarefas correspondentes.
- A disponibilidade de 99,5% é objetivo mensal mensurado no período ligado/apto, não uma alegação antecipada de que foi atingida.
- A CLI é cliente da API local e compartilha seu contrato/validações; manutenção e controle pela CLI seguem as operações descritas em JRN-014–JRN-015.

### Restrições conhecidas, sem decisão pendente

- Decisões de escopo encerradas sem alvo quantitativo: volume/carga máxima, meta/período para schedules migrados e conformidade formal de acessibilidade. Pontualidade está definida para máquina ociosa sem carga de referência. Essas ausências limitam afirmações de capacidade/sucesso, mas não deixam decisão do MVP pendente.
- Não há decisões pendentes que impeçam a implementação do MVP. Orçamento, capacidade/experiência da equipe, datas e ambiente de desenvolvimento não foram informados; por isso o plano não estima esforço nem compromete cronograma.

## Estado de execução

TASK-001–TASK-003 concluídas; TASK-004–TASK-010 e TASK-013–TASK-018 implementadas e validadas em Linux. Todos os 100 cenários BDD estão ligados a verificações executáveis; suítes Rust, Python e Playwright passaram localmente em Linux. Validações locais durante desenvolvimento podem ocorrer somente em Linux; compatibilidade nativa e execução BDD em Windows/macOS/Linux permanecem gates de release. A observação mensal TASK-011 ainda não tem uma janela completa; TASK-012 ainda tem reprodução nativa pendente para macOS/Windows.

## Próxima etapa recomendada

Continuar `/mvp` com TASK-011 (manter coletor ativo até concluir a observação mensal) e TASK-012 (reprodução de instalação/operação), usando validações Linux durante implementação. A medição de pontualidade em host ocioso e TASK-006 foram validadas em Linux; preservar reprodução nativa em macOS/Windows e CI BDD tripla como gates de release.
