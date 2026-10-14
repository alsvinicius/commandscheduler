# Como instalar e manter o daemon

O daemon precisa permanecer ativo para disparar schedules. Ele escuta apenas em loopback e mantém o banco no diretório local de dados do usuário. A instalação inclui daemon, CLI e assets compilados da UI.

## Compilar os componentes

No checkout do projeto, com Rust, Node.js e pnpm instalados conforme o [README](../../../README.md#requisitos-locais), execute:

```sh
pnpm install --frozen-lockfile
pnpm ui:build
cargo build --locked --release --bins
```

O daemon e a CLI ficam em `target/release/`; a interface compilada fica em `ui/dist/`. O pacote de binários Cargo não inclui os assets da UI. Se iniciar o daemon sem `ui/dist`, a API permanece disponível, mas não haverá interface web funcional.

## Escolher uma instalação persistente

Use o procedimento correspondente ao gerenciador de serviços do sistema:

- [Linux com systemd de usuário](../../engineering/guides/linux-service.md): inicia no login ou no boot com linger habilitado.
- [macOS com LaunchAgent](../../engineering/guides/multiplatform-service.md#macos--launchagent-do-usuário): inicia na sessão do usuário.
- [Windows com Tarefa Agendada](../../engineering/guides/multiplatform-service.md#windows--tarefa-agendada-do-usuário): inicia no logon do usuário.

Esses procedimentos também descrevem atualização, parada e remoção. O agendador não executa tarefas enquanto o computador ou daemon estiver indisponível; ao voltar, aplica-se a política de horários vencidos do schedule.

## Confirmar que está ativo

O daemon padrão atende em `http://127.0.0.1:4371`. Confira saúde, interface e referência da API:

```sh
curl --fail http://127.0.0.1:4371/health
```

Abra `http://127.0.0.1:4371/` no navegador. O contrato navegável fica em `/docs/api`, e o arquivo OpenAPI em `/openapi.yaml`. Para investigar uma falha de execução, siga [Como diagnosticar uma execução que falhou](diagnose-failed-run.md).

## Desenvolvimento

Para iniciar uma instância isolada para desenvolvimento no Linux, use [`make up`](start-dev-environment-linux.md). Ela usa a porta 4372 e um banco separado em `.tmp/dev/`; não é a instância padrão usada pela CLI.
