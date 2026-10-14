# Medir a disponibilidade mensal

A meta de 99,5% é calculada somente sobre períodos em que o host está ligado e apto a executar schedules. O resultado de uma janela é:

```text
disponibilidade = segundos elegíveis com daemon saudável / total de segundos elegíveis × 100
```

Considere o daemon saudável quando `GET /health` responde HTTP 200 com o corpo `ok` dentro do timeout de 2 segundos. Amostre a cada 10 segundos por um observador independente do daemon. Registre em armazenamento persistente timestamp UTC, resultado e duração da amostra; configure o observador para iniciar automaticamente e continuar ativo enquanto o daemon for monitorado.

Inclua no denominador somente intervalos em que o host esteve ligado e acordado. A ferramenta Linux grava relógio UTC, `time.monotonic()` e `CLOCK_BOOTTIME`: o primeiro relógio monotônico não conta suspensão, enquanto `CLOCK_BOOTTIME` conta; a diferença permite excluir sono sem confundi-lo com indisponibilidade. Se UTC divergir de `CLOCK_BOOTTIME` em mais de dois segundos, classifica o intervalo como desconhecido, não como sono. A base dessa distinção está descrita na [documentação de relógio do Linux](https://www.man7.org/linux/man-pages/man2/clock_gettime.2.html) e na [documentação oficial de `time` do Python](https://docs.python.org/3/library/time.html). A ferramenta também detecta mudança de boot pelo identificador do sistema. Uma troca de boot deixa o intervalo como desconhecido por padrão: exclua desligamento/boot somente quando registros independentes permitirem delimitar o período. Não classifique uma lacuna não explicada como host indisponível nem como host apto. Lacunas por falha do próprio observador tornam a janela incompleta e impedem alegação de SLO até recuperar evidência suficiente.

Uma amostra sem resposta válida conta como indisponibilidade quando as amostras adjacentes confirmam que o host permaneceu ligado e acordado. Para o relatório mensal, some os intervalos elegíveis entre probes adjacentes, atribua cada intervalo ao resultado da probe final e publique também tempo elegível, tempo saudável, tempo indisponível, períodos excluídos, lacunas desconhecidas e completude da observação. Preserve os registros brutos para auditoria.

O observador deve rodar separado do daemon para que uma falha seguida de reinício seja registrada. Cada processo coletor usa um identificador; quando o supervisor reinicia o coletor, o intervalo entre processos fica desconhecido, mesmo se curto. A janela só sustenta uma afirmação mensal se cobrir o mês completo com lacunas desconhecidas explicadas; até existir essa observação, 99,5% permanece uma meta, não um resultado atingido. Aplicar o mesmo cálculo nos três sistemas e documentar as fontes nativas de boot/sono usadas pelo observador de cada plataforma.

## Coletor Linux

O script `scripts/measure_availability.py` grava probes JSONL e não inicia nem reinicia o daemon. No Linux, a unit systemd de usuário mantém o observador separado e o reinicia se falhar. Instale-o junto à unit:

O coletor deste workspace começou a registrar probes em 2026-10-13 15:31:55 UTC no caminho `~/.local/state/command-scheduler/availability.jsonl`. Isso inicia observação contínua; não representa um mês completo nem permite declarar a meta de 99,5% atingida.

```sh
install -Dm755 scripts/measure_availability.py "$HOME/.local/bin/command-scheduler-availability"
install -Dm644 packaging/systemd/command-scheduler-availability.service "$HOME/.config/systemd/user/command-scheduler-availability.service"
install -d "$HOME/.local/state/command-scheduler"
systemctl --user daemon-reload
systemctl --user enable --now command-scheduler-availability.service
```

O daemon e o observador devem usar o mesmo usuário e manager systemd. Para manter serviços de usuário ativos sem sessão, siga a instrução de linger do [guia do serviço Linux](linux-service.md). Cada amostra é gravada e sincronizada em disco antes da próxima. `--interval-seconds` define a cadência (padrão 10 s) e `--timeout-seconds` o timeout de saúde (padrão 2 s). Após a janela escolhida, gere o relatório usando datas ISO-8601 com fuso:

Habilite o observador antes do início do mês e mantenha-o ativo até haver uma amostra posterior ao fim da janela; isso fornece probes para delimitar as duas bordas.

```sh
python3 scripts/measure_availability.py report \
  --log "$HOME/.local/state/command-scheduler/availability.jsonl" \
  --from 2026-10-01T00:00:00Z --to 2026-11-01T00:00:00Z
```

O relatório só pode marcar `target_met` para uma janela completa de mês-calendário UTC. Para excluir um intervalo de máquina comprovadamente desligada, acrescente `--known-offline INICIO FIM EVIDÊNCIA`; repita a opção para cada intervalo. Informe uma referência auditável, como o arquivo de log do sistema que delimita boot/shutdown. Sem essa evidência, transições de boot e lacunas ficam desconhecidas. O relatório retorna código 2 se a janela tiver intervalos desconhecidos ou nenhuma amostra elegível; uma janela curta pode ser completa, mas nunca declara a meta mensal atingida.

Exemplo para uma pausa comprovada pelos registros do sistema:

```sh
python3 scripts/measure_availability.py report \
  --log "$HOME/.local/state/command-scheduler/availability.jsonl" \
  --from 2026-10-01T00:00:00Z --to 2026-11-01T00:00:00Z \
  --known-offline 2026-10-12T22:00:00Z 2026-10-13T06:00:00Z 'journalctl --list-boots; boot journal records'
```

Para encerrar a coleta: `systemctl --user disable --now command-scheduler-availability.service`. O log fica preservado para relatório e auditoria.
