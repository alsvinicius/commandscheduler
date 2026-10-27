# Início do daemon no macOS e Windows

Estes procedimentos pressupõem compilação nativa no sistema de destino. Instale Rust stable com `rustfmt`/`clippy`, Node.js compatível com Vite 8 e pnpm 11.11.0, além do compilador C/linker disponível no sistema. No checkout do projeto, compile daemon, CLI e UI:

```sh
pnpm install --frozen-lockfile
pnpm ui:build
cargo build --locked --release --bins
```

Os binários ficam em `target/release/` (`.exe` no Windows) e os assets em `ui/dist/`. O daemon atende apenas loopback em `127.0.0.1:30002`; a UI fica em `/`, a saúde em `/health` e a referência da API em `/docs/api` (OpenAPI em `/openapi.yaml`). A API e a escolha de outro arquivo SQLite são descritas no [guia da CLI](cli.md) e no [README](../../../README.md). O workflow CI compila/testa nos três sistemas; a instalação e o comportamento do supervisor precisam de reprodução nativa antes do release.

## macOS — LaunchAgent do usuário

Com `command-scheduler` e a pasta `ui/` (com `index.html`) compilados para macOS:

```sh
mkdir -p "$HOME/.local/bin" "$HOME/.local/lib/command-scheduler/ui" "$HOME/Library/LaunchAgents" "$HOME/Library/Logs"
cp target/release/command-scheduler "$HOME/.local/lib/command-scheduler/"
cp target/release/command-scheduler-cli "$HOME/.local/bin/"
cp -a ui/dist/. "$HOME/.local/lib/command-scheduler/ui/"
python3 -c 'from pathlib import Path; import os; p=Path("packaging/launchd/com.command-scheduler.daemon.plist.in"); s=p.read_text().replace("@INSTALL_DIR@", os.path.expanduser("~/.local/lib/command-scheduler")).replace("@HOME_DIR@", str(Path.home())); Path.home().joinpath("Library/LaunchAgents/com.command-scheduler.daemon.plist").write_text(s)'
launchctl bootstrap "gui/$(id -u)" "$HOME/Library/LaunchAgents/com.command-scheduler.daemon.plist"
```

Se `~/.local/bin` não estiver no `PATH`, acrescente `export PATH="$HOME/.local/bin:$PATH"` ao arquivo de inicialização do shell. O LaunchAgent inicia na sessão do usuário, mantém o daemon ativo após fechar o navegador e reinicia falhas. Consulte com `launchctl print "gui/$(id -u)/com.command-scheduler.daemon"` e `curl http://127.0.0.1:30002/health`. Para atualizar, substitua binário/assets e faça `launchctl bootout` seguido de `bootstrap`. Para parar/remover, use `bootout`, remova o plist e, se desejar, apague `~/.local/lib/command-scheduler` e `~/.local/bin/command-scheduler-cli`. O banco SQLite no diretório local de dados do usuário permanece separado e só deve ser removido se seus dados puderem ser descartados. Logs ficam em `~/Library/Logs/command-scheduler.log`.

## Windows — Tarefa agendada do usuário

Na raiz do checkout, gere o pacote nativo e instale daemon, CLI e UI no mesmo diretório:

```powershell
pnpm install --frozen-lockfile
pnpm ui:build
cargo build --locked --release --bins
$package = Join-Path $PWD 'dist\command-scheduler'
New-Item -ItemType Directory -Force -Path (Join-Path $package 'ui') | Out-Null
Copy-Item target\release\command-scheduler.exe $package
Copy-Item target\release\command-scheduler-cli.exe $package
Copy-Item ui\dist\* (Join-Path $package 'ui') -Recurse -Force
```

No PowerShell da conta que usará o daemon, execute sem elevação para que os arquivos, a tarefa e o `PATH` de usuário pertençam a essa conta:

```powershell
.\packaging\windows\install-service.ps1 -PackageDirectory C:\caminho\do\pacote
```

O script instala daemon, CLI e assets em `%LOCALAPPDATA%\Programs\CommandScheduler`, registra uma tarefa no logon do usuário, inicia o daemon e configura até três reinícios após falha. A pasta é adicionada ao `PATH` do usuário; abra um novo terminal e use `command-scheduler-cli --help`. Consulte em Task Scheduler ou use `Get-ScheduledTask -TaskName 'Command Scheduler'`. Para atualizar, gere pacote novo e execute o mesmo script novamente. Para parar/remover:

```powershell
Stop-ScheduledTask -TaskName 'Command Scheduler'
Unregister-ScheduledTask -TaskName 'Command Scheduler' -Confirm:$false
$installDirectory = Join-Path $env:LOCALAPPDATA 'Programs\CommandScheduler'
$userPath = [Environment]::GetEnvironmentVariable('Path', 'User') -split ';' | Where-Object { $_ -and $_ -ne $installDirectory }
[Environment]::SetEnvironmentVariable('Path', ($userPath -join ';'), 'User')
Remove-Item "$env:LOCALAPPDATA\Programs\CommandScheduler" -Recurse -Force
```

A base SQLite no diretório local de dados do usuário permanece separada dos binários e não é removida pelo instalador.

O agendamento do Windows começa quando o usuário entra na sessão. A validação do comportamento do supervisor, permissões, encerramento e recuperação ainda requer reprodução em Windows antes do release.
