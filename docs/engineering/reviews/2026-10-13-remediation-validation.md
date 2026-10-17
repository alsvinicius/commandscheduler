# Validação das correções da revisão — 2026-10-13

Os 14 comentários têm tratamento no código. Os comportamentos exercitáveis em Linux passaram na validação. O ajuste de duração da tarefa Windows foi conferido no instalador e tem teste nativo configurado na CI; sua execução nativa permanece pendente, assim como a confirmação dos demais gates multiplataforma.

Esta cópia não contém metadados Git. A validação usou os arquivos atuais, binários compilados localmente e bancos/processos temporários, sem interferir no banco de uso do usuário.

## Falhas encontradas na primeira tentativa de correção

A suíte anterior passava, mas as reproduções adicionais encontraram seis falhas na API/operação e uma na UI:

- `Host` arbitrário era aceito quando `Origin` estava ausente.
- Um caminho simbólico permitia iniciar outro daemon sobre o mesmo banco.
- Um pai que terminava em dois segundos, deixando pipes herdados abertos, fazia o teste de comando levar 32 segundos.
- Cancelamento após a saída do pai retornava 409 porque o receptor de cancelamento já tinha sido descartado.
- Replay idempotente de um horário único já vencido retornava 400.
- A agenda havia sido reduzida a 500 itens no total, embora o contrato previsse 1000.
- A atualização automática da UI recolocava o histórico de A depois de selecionar B; o teste mostrou duas ocorrências no cartão B.

Essas falhas foram corrigidas nesta validação. A comparação de horários agora ocorre na mesma transação que atualiza o cursor; retries pendentes são cancelados junto com o schedule; a resposta de execução não anuncia um retry que não foi inserido. Criação manual/retry também verifica o estado do schedule na própria inserção, evitando aceitar uma ocorrência que ficaria presa na fila.

## Evidências por comentário

As reproduções da API/operação estão em [test_review_regressions.py](../../../tests/bdd/test_review_regressions.py); as de seleção/filtro estão em [journeys.spec.ts](../../../tests/web/journeys.spec.ts). O executor também tem dois testes Rust adicionais de prazo total e preservação de saída em [execution.rs](../../../src/execution.rs).

| # | Comentário original | Resultado e evidência |
|---|---|---|
| 1 | Host/origem confiáveis | 403 para domínio atacante com ou sem Origin, sem executar o programa. Origens locais correspondentes, inclusive porta do proxy Vite e IPv6, retornam 200. |
| 2 | Exclusividade antes de recuperação | Segunda instância rejeitada na mesma porta, em outra porta e por caminho simbólico. Ocorrência do daemon ativo permanece running. Bind também precede migrações, recuperação e scheduler. |
| 3 | Licença open source | LICENSE contém MIT; Cargo.toml usa SPDX MIT; README aponta para o texto. |
| 4 | Retries após cancelamento | Nenhum retry é criado depois de cancelar durante a execução. Retry já enfileirado fica cancelled e nunca inicia. Retry manual em schedule cancelado retorna 409. |
| 5 | Recuperação sequence | Três ocorrências recuperadas iniciam na ordem de scheduled_for, sem sobreposição. Outro schedule inicia antes de terminar a sequência, confirmando ausência de limite global. |
| 6 | Pipes herdados/descendentes | Prazo de 30 s inclui pai e drenagem; saída capturada é preservada e timedOut é true. Cancelamento continua válido depois da saída do pai e encerra o descendente. Grupos de processos Unix foram exercitados; Job Object Windows aguarda validação nativa. |
| 7 | Duração ilimitada da tarefa Windows | Instalador usa ExecutionTimeLimit igual a TimeSpan.Zero. [Teste nativo](../../../tests/windows/task-settings.ps1) lê o construtor real do instalador, registra uma tarefa temporária, verifica PT0S e a remove. Configurado na CI; não executado neste host Linux. |
| 8 | Cursor ao mudar timing | PATCH que muda para cron reinicia o cursor e não cria backlog anterior à alteração. Edição de nome e reenvio do mesmo horário preservam o cursor; comparação e escrita usam BEGIN IMMEDIATE. |
| 9 | Horário único anterior à criação | Novo horário vencido retorna 400. Criação futura e replay idempotente depois de vencer o horário retornam 201 e 200 para o mesmo recurso. |
| 10 | Seleção global da agenda | Cron por minuto não exclui o cron por hora da seleção dos próximos 500 horários. Até 500 finalizações são adicionadas, preservando 1000 itens no total. |
| 11 | Atualização concorrente de parâmetros | 50 PUTs concorrentes válidos retornam 200 e criam 50 gatilhos únicos. Reserva de escrita precede a leitura da configuração. |
| 12 | Respostas de seleções anteriores na UI | Resposta inicial atrasada de A é descartada ao selecionar B. Atualização automática pendente também é descartada após troca de seleção ou filtro. |
| 13 | Cursores pela chave de ordenação | Schedules e ambas as listas de ocorrências usam (created_at,id). Testes com timestamps iguais e inserção/remoção do filtro, separadamente, não repetem nem pulam itens. |
| 14 | Corrida no cancelamento da fila | UPDATE confere rows_affected e reavalia running; ausência de aplicação retorna conflito. Em 20 solicitações, toda resposta 202 chega a cancelled; 409 é admitido quando o estado já mudou. |

## Verificações executadas em Linux

| Verificação | Resultado |
|---|---|
| cargo test --all-targets | 25 testes passaram: 23 daemon e 2 CLI. |
| python3 -m unittest discover -s tests/bdd -v | 60 testes passaram, incluindo 14 reproduções da revisão. |
| pnpm bdd:web | 17 testes passaram, incluindo 2 reproduções novas de concorrência da UI. |
| cargo fmt --all -- --check | Passou. |
| cargo clippy --all-targets --all-features -- -D warnings | Passou. |
| cargo build --bins | Passou. |
| pnpm ui:build | Tipos e build passaram. O Vite mantém o aviso de tamanho dos chunks da referência de API. |
| pnpm openapi:lint | Passou. |
| python3 tests/bdd/check_coverage.py | 100/100 cenários BDD ligados a verificações executáveis. |

A paginação foi adicionalmente reexecutada depois de fortalecer o teste para separar inserção e remoção e usar timestamps iguais. Passou.

Uma medição de regressão com o binário debug atual iniciou e finalizou **100/100 ocorrências**, com zero falhas: P50 **0,144541 s**, P95 **0,257593 s**, P99 **0,286062 s**, máximo **0,306588 s**. Load average ao fim: **0,84 / 1,08 / 1,08**. A medição ocorreu durante a validação local; não constitui um novo ensaio dedicado de release em host ocioso. Resultado bruto em `.tmp/remediation-punctuality.json`.

## Limites da conclusão

Não foi executada a CI remota nem validação nativa macOS/Windows nesta sessão. A tarefa Windows e o Job Object estão implementados, mas dependem da execução dos testes nesses hosts para comprovação nativa. A validação Linux satisfaz a autorização de implementação registrada em AGENTS.md; os gates nativos e de disponibilidade mensal para release continuam vigentes.
