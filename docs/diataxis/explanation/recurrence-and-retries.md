# Recorrência, recuperação e retries

O scheduler transforma horários em ocorrências persistidas. Essa distinção permite consultar separadamente a intenção recorrente (schedule) e cada execução, incluindo resultados, tentativas e horários vencidos.

## Relógio e recorrência

Uma programação cron tem cinco campos e é calculada no fuso local do daemon. Por isso, mudar o fuso da máquina pode mudar os instantes correspondentes à mesma expressão. Em transições de horário de verão, um horário civil inexistente é pulado; um horário que aparece duas vezes é executado uma única vez.

Horários únicos identificam um instante RFC 3339 e precisam estar no futuro quando criados ou efetivamente reprogramados. Ao alterar `timing`, o cursor é reposicionado no momento da mudança. Isso evita criar retroativamente execuções para períodos em que a nova regra ainda não estava configurada.

## Horários vencidos

O daemon pode ficar desligado ou um schedule pode ficar pausado. Ao avaliar o período vencido, `missedPolicy` define como representá-lo:

- `skip` move o cursor adiante sem executar o passado.
- `consolidate` cria uma ocorrência com a contagem e os limites do intervalo vencido. O processo pode ler esses metadados em `COMMAND_SCHEDULER_MISSED_COUNT`, `COMMAND_SCHEDULER_MISSED_FROM` e `COMMAND_SCHEDULER_MISSED_THROUGH`.
- `sequence` preserva horários individuais até o limite de 100. Horários adicionais são representados por uma ocorrência consolidada. O scheduler executa ocorrências desse mesmo schedule em ordem e uma por vez; schedules diferentes continuam independentes.

Essas políticas descrevem a recuperação de horários agendados. Uma execução manual, uma execução causada por atualização de parâmetros e um retry são ocorrências distintas e não são horários vencidos.

## Falhas e retries

Uma falha pode gerar retry automático conforme `retryPolicy`. O padrão permite até três tentativas adicionais após a inicial, com atrasos de 60, 300 e 900 segundos. As ocorrências são persistidas, então o histórico mostra cada tentativa separadamente.

Parar o daemon durante uma execução marca a ocorrência como `interrupted`. O sistema não decide repeti-la automaticamente porque não sabe se os efeitos locais ocorreram antes da interrupção. O operador pode solicitar retry com confirmação, que cria uma ocorrência vinculada à anterior.

Cancelar o schedule impede novos retries automáticos e cancela os retries que ainda estão na fila. Uma execução já iniciada continua. Cancelar uma ocorrência ativa solicita o encerramento da árvore de processos e altera seu estado quando o executor confirma a conclusão.

Para procedimentos de administração, consulte [Como administrar schedules e execuções](../how-to/manage-schedules-and-runs.md); os formatos e estados estão na [referência de schedules e ocorrências](../reference/schedules-and-occurrences.md).
