# Operar o daemon no Linux com systemd

Este guia usa instalação por usuário em `~/.local`. Compile a UI e os binários no checkout:

```sh
pnpm install --frozen-lockfile
pnpm ui:build
cargo build --locked --release --bins
mkdir -p ~/.local/bin ~/.local/share/command-scheduler/ui ~/.config/systemd/user
cp target/release/command-scheduler ~/.local/bin/
cp target/release/command-scheduler-cli ~/.local/bin/
cp -a ui/dist/. ~/.local/share/command-scheduler/ui/
cp packaging/systemd/command-scheduler.service ~/.config/systemd/user/
systemctl --user daemon-reload
systemctl --user enable --now command-scheduler.service
```

O daemon atende apenas `127.0.0.1:4371`. Abra `http://127.0.0.1:4371`; a referência OpenAPI fica em `/docs/api`. O banco segue o diretório local de dados do usuário; `COMMAND_SCHEDULER_DB_PATH` pode definir outro caminho antes de iniciar o serviço. Atualize os arquivos compilados e execute `systemctl --user restart command-scheduler.service` após uma nova versão.

Confira a operação com `systemctl --user status command-scheduler.service`, `journalctl --user -u command-scheduler.service` e `curl http://127.0.0.1:4371/health`. Para iniciar serviços de usuário no boot sem sessão interativa, habilite o linger uma vez com `loginctl enable-linger "$USER"` (pode exigir autorização do administrador da máquina).

Use `command-scheduler-cli --help` para consultar os comandos. A CLI usa a API local em `127.0.0.1:4371`; os exemplos de criação, manutenção e execução estão no [guia da CLI](cli.md). Se `~/.local/bin` não estiver no `PATH`, acrescente `export PATH="$HOME/.local/bin:$PATH"` ao arquivo de inicialização do shell.

Para atualizar, copie os binários/assets novos e execute `systemctl --user restart command-scheduler.service`. Para parar: `systemctl --user disable --now command-scheduler.service`. Para remover, apague a unit em `~/.config/systemd/user/command-scheduler.service`, recarregue o manager e remova os binários/assets se não forem mais necessários. A base SQLite permanece no diretório de dados local até ser removida pelo usuário.

Se o coletor de disponibilidade também estiver instalado, encerre-o antes de remover o daemon com `systemctl --user disable --now command-scheduler-availability.service`; o log JSONL fica preservado em `~/.local/state/command-scheduler/availability.jsonl`.

## Verificar restart e recuperação

Com `cargo build --locked --release --bins` e `pnpm ui:build` concluídos, execute `python3 scripts/verify_linux_supervisor.py`. O verificador inicia uma unit temporária com banco e porta próprios, mata o processo do daemon e confirma que systemd reinicia o serviço e executa um schedule persistido. A unit e os arquivos temporários são removidos ao final; a instalação do usuário não é alterada.
