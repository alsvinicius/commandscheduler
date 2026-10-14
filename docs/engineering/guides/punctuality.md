# Medir a pontualidade de schedules

Esta medição funciona em Linux, macOS e Windows com uma instância isolada do daemon e um banco temporário. Ela cria 100 schedules de execução única espaçados por 250 ms, registra `scheduledFor` e `startedAt` das ocorrências, calcula p50/p95/p99 e encerra o daemon. Nenhum schedule é adicionado ao banco habitual. O verificador usa o binário release se disponível e, senão, o binário debug.

Compile antes:

```sh
pnpm ui:build
cargo build --locked --bins
python3 scripts/measure_punctuality.py
```

O comando requer Python 3.12, os binários e os assets compilados. A saída JSON inclui sistema operacional, quantidade iniciada/terminal, falhas, percentis e load average quando suportado. O processo retorna código diferente de zero se houver ocorrência ausente/falha, P95 > 5 s ou P99 > 30 s. GitHub Actions está configurado para executar o verificador após build e suítes em Linux, macOS e Windows. Para evidência dedicada de release, execute também em máquina ociosa e guarde a saída com data, host, versões e condições; um resultado vale apenas para aquela execução e não prova meta mensal ou capacidade sob carga.

Opções: `--samples` (mínimo 100, padrão 100) e `--spacing-ms` (mínimo 100, padrão 250). O intervalo maior reduz carga adicional durante a medição.

## Evidência coletada

Em 2026-10-13, uma execução Linux iniciou e concluiu 100/100 ocorrências: nenhuma falha/ausência; P50 0,125 s, P95 0,252 s, P99 0,268 s e máximo 0,271 s. Load average no fechamento: 0,48 / 1,87 / 2,01 (1/5/15 min). Como o histórico maior não demonstra máquina ociosa durante toda a coleta, o resultado é uma medição observada e não uma comprovação isolada do gate de máquina ociosa.

Nova execução Linux em 2026-10-13 concluiu 100/100 ocorrências: nenhuma falha/ausência; P50 0,133 s, P95 0,255 s, P99 0,274 s e máximo 0,283 s. Load average no fechamento: 1,57 / 1,81 / 2,13 (1/5/15 min). O host estava em uso interativo e a execução comprova o funcionamento do medidor, não o gate de ociosidade.
