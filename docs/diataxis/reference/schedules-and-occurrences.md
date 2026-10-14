# Referência de schedules e ocorrências

Esta página resume os conceitos e valores aceitos no uso do sistema. Os esquemas HTTP completos e respostas estão no [OpenAPI 3.1](../../engineering/api/openapi.yaml).

## Schedule

Um schedule contém executável (`program`), argumentos ordenados (`args`), horário (`timing`), política de horários vencidos (`missedPolicy`), parâmetros opcionais, política opcional de retries e gatilho opcional de atualização de parâmetros. O executável é iniciado diretamente pelo sistema operacional; não há shell implícito.

| Campo | Valores e comportamento |
|---|---|
| `timing.type=once` | `at` é um instante RFC 3339. Novo horário único deve estar no futuro. |
| `timing.type=cron` | `expression` tem cinco campos: minuto, hora, dia do mês, mês e dia da semana. Usa o fuso local do daemon. |
| `missedPolicy=skip` | Avança o cursor sem criar ocorrências para horários vencidos. |
| `missedPolicy=consolidate` | Representa horários vencidos por uma ocorrência com `missedOccurrences` (contagem e intervalo). |
| `missedPolicy=sequence` | Cria até 100 ocorrências individuais em ordem; o excedente é consolidado. Ocorrências do mesmo schedule executam uma por vez. |
| `retryPolicy.maxRetries` | Quantidade máxima de tentativas adicionais; padrão 3. |
| `retryPolicy.delaysSeconds` | Atrasos entre tentativas, em segundos; padrão `[60, 300, 900]`. Se houver mais retries que atrasos, reutiliza-se o último atraso configurado. |
| `parameterUpdateTrigger.enabled` | Habilita execução quando os parâmetros são atualizados; desabilitado por padrão. |
| `parameterUpdateTrigger.scheduleBehavior` | `replace` substitui ocorrência agendada pendente; `coexist` mantém ambas. |

Cron considera hora civil local. Horário local inexistente durante a mudança de relógio é ignorado; horário repetido ocorre uma vez. A gramática aceita cinco campos numéricos com `*`, lista, intervalo e passo. A combinação de dia do mês e dia da semana segue a regra OR tradicional quando ambos são restritos.

## Estados do schedule

| Estado | Significado |
|---|---|
| `active` | O scheduler avalia horários e pode criar ocorrências. |
| `paused` | Nenhum novo horário é avaliado até retomar; ao retomar, a política de horários vencidos é aplicada. |
| `cancelled` | Estado terminal. Novas ocorrências agendadas e retries automáticos são impedidos. Ocorrências já iniciadas continuam. |

Mudar efetivamente `timing` define o cursor de avaliação no instante da alteração. Outros PATCHes não reiniciam esse cursor. Uma ocorrência iniciada conserva a configuração que foi carregada ao começar.

## Ocorrências

Uma ocorrência é uma execução concreta ou um retry. `source` pode ser `schedule`, `manual`, `parameter_update`, `retry` ou `recovery`. `attempt` começa em 1; `retryOf` aponta para a ocorrência anterior quando se trata de retry.

| Estado | Significado |
|---|---|
| `queued` | Aguardando execução. |
| `running` | Processo iniciado. |
| `succeeded` | Processo terminou com código zero e a saída foi drenada. |
| `failed` | Falha ao iniciar, código diferente de zero ou erro ao concluir a saída. |
| `cancelled` | Processo cancelado a pedido. |
| `interrupted` | Daemon parou durante a execução; a ocorrência não é repetida automaticamente. |

Cada ocorrência retém até 1 MiB por stream. `stdoutTruncated` e `stderrTruncated` indicam saída incompleta. Listagens paginadas omitem as saídas; o detalhe da ocorrência as inclui. O histórico finalizado é mantido por até 90 dias.

Retry manual cria uma nova ocorrência e não altera a anterior. Ocorrências `interrupted` exigem confirmação explícita; retry manual em schedule cancelado é rejeitado.
