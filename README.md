# Command Scheduler

Para iniciar o frontend Vite e o daemon Axum juntos pela Project Gallery, use `make start`/`make stop`. A instância local padrão usa UI `http://127.0.0.1:30014` e daemon/API `http://127.0.0.1:30002`. `COMMAND_SCHEDULER_API_PORT` e `COMMAND_SCHEDULER_WEB_PORT` recebem as portas atribuídas; o frontend encaminha `/api` e `/health` ao daemon.

Documentação: consulte [`docs/README.md`](docs/README.md). Com o daemon ativo, a referência navegável da API fica em `http://127.0.0.1:30002/docs/api` e o OpenAPI em `/openapi.yaml`.

Plataforma local para criar e gerenciar schedules e execuções pela CLI, web/API e permitir que aplicações locais criem e parametrizem execuções.

Licenciado sob a [licença MIT](LICENSE).

**MVP em desenvolvimento.** O código está publicado em [alsvinicius/commandscheduler](https://github.com/alsvinicius/commandscheduler).
Veja a [política de segurança](SECURITY.md), os [avisos de terceiros](THIRD_PARTY_NOTICES.md) e o [procedimento de publicação](docs/diataxis/how-to/publish-open-source.md).

**Status:** CLI, daemon Rust, API local, UI Vue e persistência SQLite estão implementados e validados em Linux. Os 100 cenários BDD estão ligados a verificações executáveis; o medidor de pontualidade ociosa roda fora da CI, e seu resultado, a observação mensal e a reprodução nativa de macOS/Windows permanecem gates de release. A CI está configurada para Linux, macOS e Windows.

## Requisitos locais

- Rust stable via rustup, com `rustfmt` e `clippy` (o `rust-toolchain.toml` seleciona esses componentes).
- Node.js compatível com Vite 8 (20.19+ ou 22.12+; Node 24 também funciona) e pnpm 11.11.0.
- Compilador C e ferramentas de build exigidas pelo SQLx/SQLite nos sistemas-alvo.

## Desenvolvimento

```sh
pnpm install
make up
```

A interface fica em `http://127.0.0.1:30014` e a API de desenvolvimento em `http://127.0.0.1:30002`, com banco isolado. `make down` encerra os processos iniciados e mantém os logs e o banco de desenvolvimento em `.tmp/dev/`.

Para iniciar cada processo em primeiro plano, use `make backend` ou `make frontend`. Também é possível iniciar manualmente, depois de compilar a UI ao menos uma vez:

```sh
pnpm ui:build
cargo run --bin command-scheduler
```

O daemon atende em `http://127.0.0.1:30002`; `/health` retorna `ok`. `COMMAND_SCHEDULER_ADDR` pode alterar o endereço/porta, desde que seja loopback, e `COMMAND_SCHEDULER_WEB_DIR` aponta para outro diretório de assets. O servidor Vite encaminha `/api` e `/health` para o daemon local.

O banco SQLite fica no diretório local de dados do usuário (`ProjectDirs` do sistema operacional). `COMMAND_SCHEDULER_DB_PATH` permite escolher um arquivo específico, por exemplo durante desenvolvimento ou validação. Migrações são aplicadas ao iniciar o daemon; histórico finalizado é expurgado após 90 dias.

## Build

```sh
pnpm install
pnpm build
```

`pnpm build` compila a UI e os dois binários Rust (`command-scheduler` e `command-scheduler-cli`). Para gerar apenas os binários: `cargo build --bins`. Para apenas a UI: `pnpm ui:build`.

A CLI é cliente HTTP da API local. Exemplos (com daemon ativo):

```sh
cargo run --bin command-scheduler-cli -- create /usr/bin/echo --arg hello --cron "0 9 * * 1-5"
cargo run --bin command-scheduler-cli -- schedules
cargo run --bin command-scheduler-cli -- pause <schedule-id>
cargo run --bin command-scheduler-cli -- run <schedule-id>
```

Use `--at RFC3339` em vez de `--cron` para schedule único. `--server` ou `COMMAND_SCHEDULER_URL` altera a URL base local; o padrão é `http://127.0.0.1:30002`. Veja o [guia da CLI](docs/engineering/guides/cli.md) ou `--help` para opções de parâmetros, retries e política de horários vencidos.

## Documentação

- [Índice de documentação](docs/README.md)
- [Plano do MVP](docs/planning/implementation-plan.md)
- [Estado do workflow](docs/product/workflow-state.md)
- [Como contribuir](CONTRIBUTING.md)
- [Operar o daemon no Linux](docs/engineering/guides/linux-service.md)
- [Início automático no macOS e Windows](docs/engineering/guides/multiplatform-service.md)
