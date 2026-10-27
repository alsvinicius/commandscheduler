# Guia da CLI

A CLI é cliente do daemon local; mantenha o daemon ativo antes dos comandos. O padrão é `http://127.0.0.1:30002`. Configure outro endereço de loopback com `--server URL` ou `COMMAND_SCHEDULER_URL`.

## Criar schedules

```sh
command-scheduler-cli create /usr/bin/echo --arg "olá" --cron "0 9 * * 1-5" \
  --name "relatório" --parameters '{"destino":"local"}' \
  --missed-policy consolidate --max-retries 3 --retry-delays 60,300,900
```

Para schedule único, use `--at 2026-10-13T14:30:00-03:00` no lugar de `--cron`. Passe cada argumento como um `--arg`; a CLI envia programa e argumentos separados, sem interpolação por shell. `--parameters` aceita JSON. Políticas de horários vencidos: `skip`, `consolidate` ou `sequence`.

Para iniciar execução após atualização de parâmetros, acrescente `--trigger-on-parameters`; `--trigger-behavior` aceita `replace` ou `coexist`.

## Consultar e manter

```sh
command-scheduler-cli schedules
command-scheduler-cli show <schedule-id>
command-scheduler-cli update <schedule-id> '{"missedPolicy":"sequence"}'
command-scheduler-cli parameters <schedule-id> '{"versao":2}'
command-scheduler-cli pause <schedule-id>
command-scheduler-cli resume <schedule-id>
command-scheduler-cli cancel <schedule-id>
```

`resume` aplica a política salva aos horários vencidos durante a pausa. A CLI confirma ações apenas depois da resposta da API.

## Execuções

```sh
command-scheduler-cli run <schedule-id>
command-scheduler-cli runs <schedule-id>
command-scheduler-cli run-show <occurrence-id>
command-scheduler-cli cancel-run <occurrence-id>
command-scheduler-cli retry <occurrence-id>
command-scheduler-cli retry <interrupted-occurrence-id> --confirm
```

O detalhe apresenta status, código de saída e saídas capturadas. Retry de execução interrompida exige `--confirm`; isso cria uma nova tentativa e preserva a ocorrência original.

Para a lista completa de argumentos, execute `command-scheduler-cli --help` e `command-scheduler-cli <comando> --help`.
