# Decisões técnicas — Agendamento e execução

- Estado: ready
- Revisão: 4
- Histórico de revisões: [registro central](../../revision-history.md) — seção `engineering/decisions/scheduling-and-execution.md`
- Atualizado: 2026-10-13
- Fontes e revisões: [PRD r17](../../product/prd.md), [requisitos r1](../../product/requirements.md), [jornadas r28](../../product/user-journeys.md)
- Artefatos relacionados: [definição técnica](../technical-definition.md), [estado do workflow](../../product/workflow-state.md)
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.

### DEC-007 — Persistir schedules e configurar recuperação por schedule

- Estado: confirmada
- Fontes: REQ-001; JRN-001, JRN-002, JRN-005, JRN-006
- Decisão e alternativas: schedules/configurações sobrevivem ao reinício da máquina; comportamento para ocorrências vencidas durante indisponibilidade é configurável em cada schedule.
- Justificativa e consequências: mantém os horários sob controle do usuário; opções exatas de recuperação, tratamento de execução interrompida e armazenamento durável precisam ser especificados.
- Evidência: respostas do usuário durante a entrevista técnica e PRD r7.
- Implementação no MVP e extensão futura: persistência e as três políticas por schedule no MVP; excedentes sequenciais consolidados após 100 ocorrências.
### DEC-008 — Disponibilizar políticas para ocorrências vencidas

- Estado: confirmada
- Fontes: REQ-001; JRN-001, JRN-002, JRN-005, JRN-006
- Decisão e alternativas: por schedule, permitir pular todas as ocorrências vencidas, consolidá-las em uma execução ao retornar ou executá-las em sequência; no modo sequencial, acima de 100 ocorrências vencidas, consolidar as excedentes em uma execução.
- Justificativa e consequências: usuário quer escolher o comportamento conforme cada tarefa; consolidação automática acima de 100 evita fila excessiva. Execução consolidada informa quantidade e intervalo cobertos para que o comando/aplicação possa decidir como processar o atraso.
- Evidência: resposta do usuário durante a entrevista técnica e PRD r7.
- Implementação no MVP e extensão futura: as três opções selecionáveis no MVP; modo sequencial mantém até 100 ocorrências por schedule e consolida excedentes com parâmetros recentes e metadados da faixa coberta.
### DEC-010 — Usar os parâmetros mais recentes ao recuperar ocorrências vencidas

- Estado: confirmada
- Fontes: REQ-001, REQ-004; JRN-001, JRN-002, JRN-005, JRN-006
- Decisão e alternativas: tanto a execução consolidada como as execuções sequenciais usam os parâmetros mais recentes disponíveis quando a recuperação começar; não se mantém snapshot histórico de parâmetros por ocorrência perdida. Execução consolidada inclui quantidade e intervalo das ocorrências perdidas.
- Justificativa e consequências: decisão explícita do usuário. Aplicações integradoras podem atualizar parâmetros antes da retomada; ocorrências sequenciais não reproduzem estados históricos intermediários.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-13.
- Implementação no MVP e extensão futura: aplicável às três políticas configuráveis de ocorrência vencida no MVP.
### DEC-011 — Exigir confirmação para retry de comandos interrompidos

- Estado: confirmada
- Fontes: REQ-001, REQ-006; JRN-009, JRN-012
- Decisão e alternativas: comando identificado como em andamento após reinício será tratado como interrompido e requer confirmação manual antes de uma nova tentativa; não se tenta retomar o processo nem repetir automaticamente.
- Justificativa e consequências: decisão explícita do usuário para evitar repetir efeitos externos parciais. O runtime deve distinguir retry normal de recuperação de processo interrompido e expor ação de confirmação.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-13.
- Implementação no MVP e extensão futura: confirmação manual para retentativa de interrupção; retry manual e automático configurável seguem disponíveis para falhas normais segundo REQ-006.
### DEC-013 — Reter histórico de execuções por 90 dias

- Estado: confirmada
- Fontes: REQ-001, REQ-006; JRN-004, JRN-009, JRN-010, JRN-012
- Decisão e alternativas: manter histórico de execuções por 90 dias; depois disso pode ser removido.
- Justificativa e consequências: decisão explícita do usuário; reduz crescimento indefinido do banco local, limitando consulta retrospectiva ao período de retenção.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-13.
- Implementação no MVP e extensão futura: retenção de 90 dias no MVP.
### DEC-014 — Consolidar automaticamente ocorrências sequenciais excedentes

- Estado: confirmada
- Fontes: REQ-001; JRN-001, JRN-002, JRN-005, JRN-006
- Decisão e alternativas: em política sequencial, se o número de ocorrências vencidas exceder o limite, consolidar o excedente em uma execução.
- Justificativa e consequências: escolha explícita do usuário para evitar uma fila potencialmente grande; consolidar excedentes protege contra backlog ilimitado. A semântica do payload consolidado ainda precisa ser definida.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-13.
- Implementação no MVP e extensão futura: limite de 100 ocorrências sequenciais por schedule no MVP; consolidar excedentes em uma execução.
### DEC-017 — Configurar retry automático padrão para falhas normais

- Estado: confirmada
- Fontes: REQ-006; JRN-005, JRN-009, JRN-012
- Decisão e alternativas: padrão de 3 tentativas de retry além da execução inicial, nos intervalos de 1, 5 e 15 minutos; configuração pode ser alterada por schedule.
- Justificativa e consequências: decisão explícita do usuário baseada na recomendação; aplica-se a falhas normais, não a execução interrompida por reinício, que exige confirmação manual.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-13.
- Implementação no MVP e extensão futura: retry padrão por schedule com três tentativas; detalhes de configuração de desligamento do retry ficam na API/UI.
### DEC-018 — Tratar horários inexistentes e repetidos no fuso local

- Estado: confirmada
- Fontes: REQ-001; JRN-001, JRN-002, JRN-005, JRN-006
- Decisão e alternativas: pular horário local inexistente durante avanço do relógio e executar uma única vez o horário local repetido durante recuo.
- Justificativa e consequências: decisão explícita do usuário; evita duas execuções em horário local repetido e não desloca horários inexistentes para outro horário.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-13.
- Implementação no MVP e extensão futura: aplicar a regra às recorrências no fuso da máquina.

### DEC-037 — Usar cron padrão de cinco campos para recorrências

- Estado: confirmada como escolha técnica da TASK-002, dentro da autonomia do plano.
- Fontes: REQ-001; JRN-001, JRN-002; DEC-018; TASK-002.
- Decisão e alternativas: recorrências usam expressão cron padrão de cinco campos (minuto, hora, dia do mês, mês, dia da semana), interpretada no fuso local atual da máquina. Campos aceitam números e curingas, listas, intervalos e passos; dia da semana usa 0–7 (domingo pode ser 0 ou 7). Quando dia do mês e dia da semana são ambos restritos, a ocorrência segue a semântica cron tradicional OR. Não há campo de segundos nem aliases proprietários.
- Justificativa e consequências: cinco campos são concisos e reconhecidos amplamente por agendadores Unix e bibliotecas; excluir segundos reduz diferenças de dialeto e mantém precisão alinhada às metas de pontualidade. Um único parser Rust e os mesmos vetores de contrato/BDD validam as expressões em Linux durante implementação; o parser e a regra de calendário não variam por sistema operacional. A regra de horário local inexistente/repetido permanece a de DEC-018.
- Implementação no MVP e extensão futura: validar a sintaxe completa no daemon antes de persistir; rejeitar expressões inválidas. A decisão fica registrada no OpenAPI para consumidores CLI, UI e integrações.
### DEC-019 — Expor metadados de consolidação ao processo e à API

- Estado: confirmada
- Fontes: REQ-001, REQ-004; JRN-001, JRN-002, JRN-003, JRN-010
- Decisão e alternativas: o contexto da execução consolidada expõe quantidade em `COMMAND_SCHEDULER_MISSED_COUNT`, início em `COMMAND_SCHEDULER_MISSED_FROM` e fim em `COMMAND_SCHEDULER_MISSED_THROUGH`; timestamps usam RFC 3339. A API retorna campos estruturados equivalentes, com nomes JSON ainda a definir.
- Justificativa e consequências: decisão explícita do usuário. Evita alterar os argumentos originais do comando e permite a aplicações integradoras consultar o mesmo contexto.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-12.
- Implementação no MVP e extensão futura: nomes e formato das variáveis de ambiente definidos; nomes JSON no contrato API ainda precisam ser escolhidos.
### DEC-023 — Passar parâmetros JSON por variável de ambiente

- Estado: confirmada
- Fontes: REQ-004; JRN-002, JRN-003, JRN-007
- Decisão e alternativas: qualquer valor JSON válido, sem schema obrigatório nem exigência de objeto na raiz, é entregue ao processo por `COMMAND_SCHEDULER_PARAMS_JSON`; o scheduler não substitui placeholders no texto do comando. Limite: 64 KiB por schedule na criação e nas atualizações.
- Justificativa e consequências: escolha confirmada pelo usuário para evitar interpolação de valores no comando; consumidor do comando precisa ler e interpretar JSON. JSON inválido ou acima do limite é rejeitado sem alterar os parâmetros aceitos anteriormente.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-12.
- Implementação no MVP e extensão futura: qualquer valor JSON válido até 64 KiB na criação e atualização, disponível em `COMMAND_SCHEDULER_PARAMS_JSON` a cada execução.
### DEC-026 — Representar comando como executável e argumentos separados

- Estado: confirmada
- Fontes: REQ-001, REQ-004; JRN-001–JRN-003, JRN-007
- Decisão e alternativas: representar execução como programa mais lista de argumentos, sem interpretar implicitamente uma string por shell. Quando sintaxe de shell for necessária, registrar o shell explicitamente como programa e passar script/comando como argumento.
- Justificativa e consequências: separa dados de controle e argumentos, reduz ambiguidades de parsing entre plataformas e mantém valores JSON fora de interpolação textual. Comandos em pipeline exigirão execução explícita do shell desejado.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-12.
- Implementação no MVP e extensão futura: formato `program` + `args` na API/UI; shell explícito permanece possível.
### DEC-029 — Aplicar recuperação configurada a todo atraso de ocorrência

- Estado: confirmada
- Fontes: REQ-001; JRN-001, JRN-002, JRN-005, JRN-006
- Decisão e alternativas: atraso causado por suspensão da máquina ou atraso temporário do serviço segue a política de ocorrências vencidas do schedule (pular, consolidar ou executar em sequência), mesmo sem reinício do daemon.
- Justificativa e consequências: unifica semântica de recuperação e evita comportamento distinto conforme a causa do atraso.
- Evidência: resposta do usuário durante a definição técnica, 2026-10-12.
- Implementação no MVP e extensão futura: avaliar ocorrências vencidas e aplicar a política vigente de cada schedule.
### DEC-030 — Não impor limite global de concorrência no MVP

- Estado: confirmada
- Fontes: REQ-001, REQ-005; JRN-001, JRN-005, JRN-007–JRN-012
- Decisão e alternativas: não haverá limite global de execuções simultâneas no MVP; execuções concorrentes, inclusive do mesmo schedule, permanecem permitidas.
- Justificativa e consequências: escolha explícita do usuário; consumo de recursos do host pode crescer com schedules e gatilhos simultâneos.
- Evidência: resposta do usuário durante a definição técnica, 2026-10-12.
- Implementação no MVP e extensão futura: sem cota global aplicada; limites futuros podem ser adicionados por decisão posterior.
### DEC-031 — Capturar e reter stdout/stderr com limite

- Estado: confirmada
- Fontes: REQ-001, REQ-006; JRN-004, JRN-009–JRN-012
- Decisão e alternativas: capturar `stdout` e `stderr` de cada execução, reter ambos por 90 dias e limitar cada stream a 1 MiB, truncando bytes excedentes.
- Justificativa e consequências: fornece diagnóstico no histórico sem permitir crescimento ilimitado por uma única execução; UI/API deve indicar claramente truncamento.
- Evidência: resposta do usuário durante a definição técnica, 2026-10-12.
- Implementação no MVP e extensão futura: saída capturada por stream, até 1 MiB cada, junto ao histórico de 90 dias.

### DEC-039 — Aplicar a política salva após retomar um schedule

- Estado: confirmada
- Fontes: REQ-001; JRN-005, JRN-006; DEC-007, DEC-008
- Decisão e alternativas: horários que vencerem enquanto o schedule estiver pausado serão tratados ao retomar conforme a política de ocorrências vencidas configurada no próprio schedule. O cursor de avaliação não avança durante a pausa.
- Justificativa e consequências: decisão explícita do usuário em 2026-10-13 (“Aplicar a política salva”). A retomada pode gerar execução, consolidação ou pular ocorrências conforme `missedPolicy`; schedules pausados não disparam durante a pausa.
- Evidência: resposta do usuário no workflow `/mvp`, 2026-10-13.
- Implementação no MVP e extensão futura: avaliar desde o cursor persistido ao retomar e aplicar `skip`, `consolidate` ou `sequence`, como em outras indisponibilidades.
