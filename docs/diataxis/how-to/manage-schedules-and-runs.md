# Como administrar schedules e execuções

Use a página **Agendamentos** da interface web ou a CLI para alterar um schedule existente. A página **Execuções** reúne o histórico de todos os schedules, exibe o nome do schedule responsável e permite cancelar uma execução ativa ou solicitar retry. A página inicial apresenta horários previstos e ocorrências enfileiradas ou em andamento nas próximas 24 horas. A [referência da CLI](../reference/cli.md) lista os subcomandos; o [OpenAPI](../../engineering/api/openapi.yaml) define os códigos HTTP e os corpos completos.

## Alterar a configuração

Consulte o schedule antes de editá-lo:

```sh
command-scheduler-cli show SCHEDULE_ID
```

Envie ao comando `update` um objeto JSON com os campos que devem mudar. Exemplo para trocar somente a política de horários vencidos:

```sh
command-scheduler-cli update SCHEDULE_ID '{"missedPolicy":"sequence"}'
```

Uma alteração efetiva em `timing` reinicia o cursor no momento da alteração; os horários anteriores à nova programação não são recuperados. Reenviar o mesmo horário ou alterar outros campos preserva o cursor.

## Pausar, retomar ou cancelar

```sh
command-scheduler-cli pause SCHEDULE_ID
command-scheduler-cli resume SCHEDULE_ID
command-scheduler-cli cancel SCHEDULE_ID
```

Ao retomar, horários vencidos durante a pausa seguem a política `missedPolicy` configurada. Cancelar é terminal para o schedule: não é possível retomá-lo. Ocorrências já iniciadas continuam; retries automáticos futuros são impedidos e retries que ainda aguardavam na fila são cancelados.

## Atualizar parâmetros

O comando `parameters` substitui o valor JSON completo, que pode ser objeto, array, texto, número, booleano ou `null`:

```sh
command-scheduler-cli parameters SCHEDULE_ID '{"lote":42,"modo":"incremental"}'
```

Por padrão, a atualização só altera os parâmetros. Se o schedule tiver o gatilho de atualização habilitado, a API pode também iniciar uma ocorrência. `scheduleBehavior` define se essa execução substitui a ocorrência agendada pendente ou coexiste com ela. Consulte o schedule após a resposta para confirmar o gatilho e os parâmetros atuais.

## Acompanhar e cancelar uma ocorrência

Liste as ocorrências de um schedule e consulte a ocorrência pelo identificador:

```sh
command-scheduler-cli runs SCHEDULE_ID
command-scheduler-cli run-show OCCURRENCE_ID
```

Para solicitar execução imediata, use `run SCHEDULE_ID`; para cancelar uma execução ativa, use:

```sh
command-scheduler-cli cancel-run OCCURRENCE_ID
```

O cancelamento é um pedido para encerrar o processo e seus descendentes. A resposta aceita o pedido, mas não garante que o processo já terminou; consulte o estado novamente.

## Repetir uma ocorrência

Uma falha pode gerar retry automático conforme `retryPolicy`. Para iniciar uma repetição manual de uma falha, use:

```sh
command-scheduler-cli retry OCCURRENCE_ID
```

Retry cria uma nova ocorrência e preserva o histórico anterior. Uma ocorrência `interrupted` exige confirmação explícita com `--confirm`, pois a execução anterior pode ter produzido efeitos antes do daemon parar:

```sh
command-scheduler-cli retry OCCURRENCE_ID --confirm
```

Não solicite retry de uma ocorrência interrompida sem avaliar se é seguro executar o comando novamente.
