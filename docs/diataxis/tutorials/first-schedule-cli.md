# Criar e executar um schedule pela CLI

Neste tutorial você compilará o daemon e a CLI, iniciará o serviço local e usará a CLI para criar um schedule, iniciar uma execução e ler a saída. No final, terá um schedule futuro de demonstração que poderá cancelar.

## Compilar e iniciar o daemon

Este tutorial usa caminhos e comandos de Linux. Instale as ferramentas listadas em [Requisitos locais](../../../README.md#requisitos-locais). Na raiz do projeto, compile os binários e inicie o daemon em um terminal:

```sh
cargo build --locked --bins
cargo run --bin command-scheduler
```

Mantenha esse terminal aberto. Em outro terminal, confirme que o serviço responde:

```sh
curl --fail http://127.0.0.1:4371/health
```

## Criar e executar

Crie um schedule único no futuro. O horário não será alcançado durante o tutorial; a execução será iniciada manualmente:

```sh
target/debug/command-scheduler-cli create /usr/bin/printf \
  --arg 'Olá pela CLI\n' \
  --at 2099-01-01T00:00:00Z
```

A saída JSON inclui o `id`. Copie-o e inicie uma ocorrência agora:

```sh
target/debug/command-scheduler-cli run ID_DO_SCHEDULE
```

A resposta contém o identificador da ocorrência. Consulte-a até `status` ser `succeeded` ou terminal:

```sh
target/debug/command-scheduler-cli run-show ID_DA_OCORRENCIA
```

Você verá `stdout` com `Olá pela CLI`. O executável e os argumentos foram enviados separadamente, sem passar por um shell.

## Cancelar o schedule de demonstração

Como o horário de 2099 não deve permanecer ativo, cancele o schedule:

```sh
target/debug/command-scheduler-cli cancel ID_DO_SCHEDULE
```

O cancelamento impede novas ocorrências agendadas e retries automáticos. Para conhecer os demais comandos, consulte a [referência da CLI](../reference/cli.md). Para encerrar o daemon, use `Ctrl+C` no terminal em que ele está rodando.
