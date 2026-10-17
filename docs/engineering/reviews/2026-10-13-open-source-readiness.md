# Nova validação para publicação open source

Data: 2026-10-13. Destino informado: `https://github.com/alsvinicius/commandscheduler`.

## Conclusão

**Nenhum bloqueio local conhecido para abrir o código como MVP em desenvolvimento.**
Licença MIT, avisos de terceiros, política de segurança, metadados, limite de
arquivos publicáveis e verificações automatizadas estão presentes. As auditorias
de dependências e a varredura dos arquivos publicáveis terminaram limpas.

**Uma release multiplataforma ainda não está comprovada.** O repositório GitHub
será criado pelo usuário; a consulta à API GitHub retornou 404 nesta sessão,
portanto ele ainda não está disponível para esta conta. Esta cópia não tem
metadados Git. A execução remota da
matriz nativa e a varredura de um histórico Git real ainda não ocorreram. A
observação mensal de disponibilidade permanece pendente. Resultado limpo nas
verificações abaixo não é garantia de ausência de vulnerabilidades desconhecidas.

## Problemas encontrados e corrigidos nesta rodada

| Achado | Correção e reprodução |
|---|---|
| `@ai-sdk/provider-utils` 4.0.5, [GHSA-866g-f22w-33x8](https://github.com/advisories/GHSA-866g-f22w-33x8), transitivo de Scalar | Scalar atualizado para 1.74.0; override limitado às versões 4.x afetadas fixa 4.0.33. `pnpm audit` passou sem exceções. |
| `rsa` no lockfile, [RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html), trazido por drivers opcionais SQLx | Uso direto de `sqlx-core`/`sqlx-sqlite` 0.8.6 remove MySQL/PostgreSQL e RSA do lockfile. Mapeamentos e migrações explícitos. Banco criado pela implementação anterior reabriu com dados e checksums preservados. `cargo audit` passou sem ignorar alertas. |
| Atualizações concorrentes de parâmetros de schedule cancelado excediam o timeout HTTP | Rollback antes da consulta secundária libera reserva e conexão. Regressão verifica 20 respostas 409 e 20 respostas 404 concorrentes, sem timeout/500. |
| `cargo package` incluía `node_modules` e, com padrões não ancorados, arquivos de `.tmp` | Lista explícita e ancorada contém 21 arquivos. Pacote foi efetivamente gerado, extraído e recompilado pelo Cargo. |
| Gitleaks sinalizou chave fictícia de idempotência da documentação | Exemplo identificado como não secreto e simplificado; nova varredura passou sem suppressions. |
| Assets e componente de scaffold sem uso; chunks da referência de API acima de 500 kB | Scaffold removido, favicon próprio e divisão das bibliotecas em chunks menores. Build sem warnings; os 17 testes web passaram após a alteração. |

Os [14 comentários da revisão anterior](2026-10-13-remediation-validation.md)
continuam cobertos por reproduções. A suíte agora inclui também a regressão de
parâmetros rejeitados e dois contratos da CI de segurança.

## Verificações finais

| Verificação | Resultado |
|---|---|
| `cargo test --locked --all-targets` | 25 testes: 23 daemon + 2 CLI; nenhum ignorado em Linux. |
| `python3 -m unittest discover -s tests/bdd -v` | 63 testes passaram: 32 jornadas, 4 contratos CI, 12 disponibilidade, 15 regressões. |
| `pnpm bdd:web` | 17 testes passaram com o build final e a nova divisão de chunks. |
| fmt, Clippy com `-D warnings`, build debug e release | Passaram. |
| `cargo +1.85.0 check --locked --all-targets` | Passou na versão mínima anunciada. |
| Clippy cruzado para `x86_64-pc-windows-gnu`, todos os targets/features, `-D warnings` | Passou usando Zig para os objetos C. Não executa código Windows. |
| Instalação pnpm com lockfile congelado, tipos e build UI | Passaram; build UI sem warnings. |
| OpenAPI lint e rastreabilidade BDD | Passaram; 100/100 cenários ligados a verificações executáveis. |
| `cargo audit` | Zero vulnerabilidades; zero warnings; sem exceções. Base RustSec com 1.310 advisories, commit `6aed84668c23d7f535077a9fe918bf1fa0f6a5bc`. |
| `pnpm audit` | Zero vulnerabilidades em todas as severidades; sem exceções. |
| Gitleaks 8.30.1, binário com checksum verificado | Zero segredos detectados nos arquivos publicáveis. A varredura final inclui este relatório. |
| `scripts/check_publication.py` | Metadados, arquivos do pacote, todas as migrações embutidas e links locais conferidos. |
| `scripts/export_third_party_notices.py --check` | Textos de 513 versões de dependências conferidos; fontes externas fixadas por commit e checksum. |
| Avisos no daemon | `/third-party-notices.txt` retornou exatamente o arquivo distribuído pelo build. |
| Actionlint 1.7.7 | Os dois workflows passaram. Actions fixadas por SHA; token com leitura. |
| Reinício systemd em unit/porta/banco temporários | Um restart após SIGKILL, PID novo e schedule recuperado com ocorrência succeeded; limpeza realizada pelo verificador. |

O pacote Cargo contém binários/fontes, migrações, contrato e licenças; não instala
a UI. A distribuição web precisa também de `ui/dist`, conforme o guia de uso.
Os avisos incluem fontes upstream e atribuições; a licença própria não substitui
as condições das dependências.

## Pontualidade do binário final

Após 60 segundos de observação, a CPU ficou **87,60% e 88,50% ociosa** em duas
janelas de 30 segundos. O binário release atual iniciou e finalizou **100/100
ocorrências**, com zero ausências ou falhas: P50 **0,128350 s**, P95 **0,258917 s**,
P99 **0,282066 s**, máximo **0,282174 s**. Load average ao fim: **2,07 / 3,99 / 4,17**.
Esse resultado passa as metas de latência do medidor, mas descreve um host com
atividade; não foi classificado como novo ensaio dedicado de release em host ocioso.

## Após criar o GitHub

1. Enviar apenas os arquivos revisados e habilitar relatórios privados de segurança.
2. Obter sucesso dos workflows `CI` e `Security` no commit publicado. O workflow
   Security varre todo o histórico Git e repete as auditorias semanalmente.
3. Conferir os resultados nativos macOS/Windows, incluindo descendentes e registro
   da tarefa Windows com `PT0S`. Proteger merges com essas verificações.
4. Cumprir os gates de medição dedicada e janela mensal antes de anunciar as
   respectivas garantias de release. Publicar o código como MVP não comprova essas metas.

Os logs e resultados brutos desta rodada ficam em `.tmp/revalidation-*` e não
fazem parte dos arquivos publicáveis. O procedimento está no
[guia Diátaxis de publicação](../../diataxis/how-to/publish-open-source.md).
