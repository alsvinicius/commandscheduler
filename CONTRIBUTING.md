# Contribuindo

O MVP usa um único limite de projeto: daemon e CLI em Rust na raiz; interface Vue 3 + TypeScript + Vite em `ui/`, compilada e servida pelo daemon na origem local.

## Ferramentas

- Rust stable, selecionado por `rust-toolchain.toml`, com `rustfmt` e `clippy`.
- Node.js compatível com Vite 8 e pnpm 11.11.0 (`packageManager` no `package.json`).
- Ferramentas de compilação C nos ambientes que compilam SQLx/SQLite.
- O daemon usa o diretório local de dados do usuário para SQLite; `COMMAND_SCHEDULER_DB_PATH` sobrescreve o caminho.
- Para auditoria de dependências Rust: `cargo install cargo-audit --version 0.22.2 --locked`.

## Comandos conhecidos

- `pnpm install`: instala dependências da UI usando o lockfile da raiz.
- `cargo audit` / `pnpm audit`: conferem vulnerabilidades conhecidas nos lockfiles, sem exceções.
- `python3 scripts/check_publication.py`: verifica metadados, links locais, migrações embutidas e a lista explícita de arquivos do pacote Cargo.
- `python3 scripts/export_third_party_notices.py`: regenera os avisos de terceiros após mudanças de dependências; `--check` verifica que estão atualizados.
- `make up` / `make down`: inicia/encerra daemon (porta 30002) e Vite (porta 30014) para desenvolvimento; logs e SQLite isolado ficam em `.tmp/dev/`.
- `make backend` / `make frontend`: inicia o daemon ou o Vite em primeiro plano.
- `pnpm ui:dev`: inicia Vite em `127.0.0.1:30014` e encaminha `/api` e `/health` ao daemon em `127.0.0.1:30002`.
- `pnpm ui:build`: verifica tipos com `vue-tsc` e compila a UI para `ui/dist`.
- `pnpm openapi:lint`: valida a especificação OpenAPI 3.1 versionada.
- `python3 -m unittest discover -s tests/bdd -v`: executa cenários de jornada API/CLI em daemon temporário.
- `python3 scripts/measure_punctuality.py`: mede P50/P95/P99 do início de 100 schedules em daemon e SQLite temporários; consulte o [guia](docs/engineering/guides/punctuality.md). A CI está configurada para executar nos três SOs; evidência dedicada de release requer host ocioso.
- `python3 scripts/verify_linux_supervisor.py`: em Linux com manager systemd de usuário ativo, comprova restart após falha e recuperação de schedule usando unit, porta e SQLite temporários; veja o [guia do serviço Linux](docs/engineering/guides/linux-service.md).
- `python3 scripts/measure_availability.py monitor --log "$HOME/.local/state/command-scheduler/availability.jsonl"`: coleta probes locais duráveis; a geração de relatório e as exclusões estão no [guia de disponibilidade](docs/engineering/guides/availability.md). Uma meta só pode ser declarada após observação completa da janela mensal.
- `cargo fmt --all -- --check`: verifica formatação Rust.
- `cargo clippy --all-targets --all-features -- -D warnings`: executa Clippy.
- `cargo build --bins`: compila daemon e CLI.
- `pnpm build`: compila UI e binários Rust.

Durante a implementação, validações locais podem ocorrer apenas em Linux conforme a orientação registrada no [plano](docs/planning/implementation-plan.md#regra-temporaria-de-validacao-durante-implementacao). O workflow CI está configurado para build e testes Rust, build da UI, lint OpenAPI, suítes BDD Python/Playwright e medidor de pontualidade em Linux, macOS e Windows. A suíte Python contém 32 testes de jornada, quatro testes de contrato CI, doze verificações do relatório de disponibilidade e 15 reproduções da revisão; 25 testes Rust, 17 testes de browser e o script de pontualidade vinculam verificações aos 100 cenários. Em 2026-10-13, `python3 tests/bdd/check_coverage.py` confirmou 100/100 cenários ligados a verificações executáveis. Em 2026-10-13, medição Linux após 60 s com CPU 95,29–96,08% ociosa concluiu 100/100 inícios sem falhas (P95 0,2505 s; P99 0,269469 s), atendendo à meta de pontualidade nesse ambiente.

Antes de alterar comportamento, consulte os requisitos, jornadas, definição técnica e tarefa correspondente em `docs/`. Registre decisões reversíveis de implementação e evidências no [estado do workflow](docs/product/workflow-state.md); mudanças de requisito precisam ser propagadas para artefatos dependentes.

A validação das correções de segurança, concorrência e execução está registrada no [relatório de 2026-10-13](docs/engineering/reviews/2026-10-13-remediation-validation.md). O teste nativo da duração ilimitada da tarefa Windows está em `tests/windows/task-settings.ps1` e é executado pela CI Windows.

O workflow `Security` audita Rust/JavaScript, confere os avisos e varre todo o histórico Git com Gitleaks. As actions são fixadas por commit e os workflows usam permissões de leitura. Consulte a [nova validação de publicação](docs/engineering/reviews/2026-10-13-open-source-readiness.md) antes de anunciar releases.

SQLite usa diretamente `sqlx-core` e `sqlx-sqlite` 0.8.6, sem os drivers opcionais MySQL/PostgreSQL. Os mapeamentos de linhas e as migrações embutidas estão explícitos no código. Ao adicionar uma migração, inclua-a na lista de `src/storage.rs`; preserve os arquivos já aplicados para não mudar seus checksums.
