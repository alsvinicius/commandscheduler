# Referência da CLI

O binário `command-scheduler-cli` é um cliente HTTP do daemon local. O endereço padrão é `http://127.0.0.1:4371`.

## Opções globais

| Opção | Descrição |
|---|---|
| `--server URL` | URL base do daemon; deve usar loopback. |
| `COMMAND_SCHEDULER_URL` | Alternativa por variável de ambiente para `--server`. |
| `--help` | Lista comandos ou opções do comando selecionado. |
| `--version` | Exibe a versão. |

## Comandos

| Comando | Operação |
|---|---|
| `create PROGRAM` | Cria schedule único ou recorrente. Exige `--at RFC3339` ou `--cron EXPR`. |
| `schedules` | Lista schedules. |
| `show ID` | Consulta schedule. |
| `update ID JSON` | Atualiza configuração usando objeto JSON compatível com PATCH. |
| `parameters ID JSON` | Substitui parâmetros por qualquer valor JSON. |
| `pause ID`, `resume ID`, `cancel ID` | Altera estado do schedule. |
| `run ID` | Inicia execução manual imediata. |
| `runs ID` | Lista ocorrências do schedule. |
| `run-show ID` | Consulta estado, resultado e saídas da ocorrência. |
| `cancel-run ID` | Solicita cancelamento da ocorrência ativa. |
| `retry ID` | Repete ocorrência com falha; ocorrência interrompida exige `--confirm`. |

## Opções de `create`

| Opção | Padrão | Significado |
|---|---:|---|
| `--arg VALUE` | nenhum | Argumento individual; pode ser repetido e preserva a ordem. |
| `--at RFC3339` | nenhum | Horário de schedule único; conflita com `--cron`. |
| `--cron EXPR` | nenhum | Expressão cron de cinco campos; conflita com `--at`. |
| `--name VALUE` | nenhum | Nome opcional. |
| `--parameters JSON` | `null` | Parâmetros serializados como JSON. |
| `--missed-policy VALUE` | `skip` | Tratamento dos horários vencidos: `skip`, `consolidate` ou `sequence`. |
| `--max-retries N` | `3` | Número de novas tentativas para falhas normais. |
| `--retry-delays LIST` | `60,300,900` | Atrasos em segundos para tentativas. |
| `--trigger-on-parameters` | desativado | Solicita execução quando os parâmetros são atualizados. |
| `--trigger-behavior VALUE` | `coexist` | Política `replace` ou `coexist` para o disparo por parâmetros. |

A CLI imprime respostas JSON. Horários cron usam o fuso local do daemon. Programa e argumentos são enviados separadamente; consulte [modelo de execução](../explanation/execution-model.md).
