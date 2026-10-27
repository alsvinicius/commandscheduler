# Criar e executar seu primeiro schedule

Neste tutorial, você iniciará o ambiente local, criará um schedule para `/usr/bin/printf` e solicitará uma execução imediata. Ao final, verá o texto produzido pela execução.

## Pré-requisitos

- Linux com Rust, Node.js, pnpm e `make` instalados conforme os requisitos do [README](../../../README.md#requisitos-locais).
- `curl` e `jq` disponíveis no terminal.

## Iniciar o ambiente

Na raiz do repositório, instale as dependências da interface e suba o ambiente:

```sh
pnpm install
make up
```

O daemon de desenvolvimento estará em `http://127.0.0.1:30002`. Confirme que respondeu:

```sh
curl --fail http://127.0.0.1:30002/health
```

## Criar o schedule

Crie um schedule único distante no futuro. Ele não será disparado pelo relógio durante o tutorial; em seguida vamos pedir uma execução manual:

```sh
schedule_id=$(curl --fail --silent --show-error \
  -H 'Content-Type: application/json' \
  -H 'Idempotency-Key: diataxis-first-schedule' \
  -d '{"program":"/usr/bin/printf","args":["Olá, Command Scheduler\\n"],"timing":{"type":"once","at":"2099-01-01T00:00:00Z"},"missedPolicy":"skip"}' \
  http://127.0.0.1:30002/api/v1/schedules | jq -r .id)
printf 'Schedule: %s\n' "$schedule_id"
```

Você verá o identificador do schedule. `program` contém o executável e cada item de `args` é um argumento separado.

## Solicitar uma execução e consultar o resultado

Peça uma execução manual:

```sh
occurrence_id=$(curl --fail --silent --show-error \
  -X POST http://127.0.0.1:30002/api/v1/schedules/"$schedule_id"/runs | jq -r .id)
printf 'Execução: %s\n' "$occurrence_id"
```

Consulte o resultado até o estado deixar de ser `queued` ou `running`:

```sh
curl --fail --silent --show-error \
  http://127.0.0.1:30002/api/v1/occurrences/"$occurrence_id" | jq '{status, exitCode, stdout, stderr}'
```

O resultado esperado inclui `"status": "succeeded"` e `stdout` com `Olá, Command Scheduler`. Você criou um schedule e iniciou uma ocorrência sem manter a interface aberta.

## Encerrar

```sh
make down
```

O banco e os logs isolados do ambiente permanecem em `.tmp/dev/`.

Para conhecer opções da criação, consulte o [contrato OpenAPI](../../engineering/api/openapi.yaml). Para entender por que executável e argumentos são separados, veja [modelo de execução](../explanation/execution-model.md).
