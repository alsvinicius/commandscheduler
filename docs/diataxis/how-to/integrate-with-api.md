# Como integrar uma aplicação pela API

Use a API local para criar schedules a partir de outra aplicação e depois consultar ocorrências e resultados. A descrição completa dos campos e respostas está no [OpenAPI 3.1](../../engineering/api/openapi.yaml), servido também pelo daemon em `/docs/api` e `/openapi.yaml`.

## Criar um schedule

Envie `POST /api/v1/schedules` com `program`, `args`, `timing` e `missedPolicy`. O daemon continua escutando em loopback. Para acesso por outro dispositivo do tailnet, o proxy local do Workspace encaminha a porta do projeto pela interface Tailscale; nesse caso, o `Host` pode ser `localhost`, um endereço numérico de loopback ou um IP Tailscale IPv4 (`100.64.0.0/10`). Domínios arbitrários continuam rejeitados. Chamadas de navegador devem usar origem HTTP com a mesma autoridade do `Host`; isso inclui o proxy Vite local em desenvolvimento.

```sh
curl --fail-with-body --silent --show-error \
  -H 'Content-Type: application/json' \
  -H 'Idempotency-Key: exemplo-1' \
  -d '{
    "name": "relatório diário",
    "program": "/usr/bin/python3",
    "args": ["/opt/jobs/report.py", "--format", "json"],
    "timing": {"type": "cron", "expression": "30 8 * * 1-5"},
    "missedPolicy": "consolidate",
    "parameters": {"tenant": "acme"}
  }' \
  http://127.0.0.1:4371/api/v1/schedules
```

Use um caminho de executável acessível ao usuário que executa o daemon. Cada elemento de `args` vira exatamente um argumento do processo. Para schedules únicos, `timing` tem formato `{"type":"once","at":"<RFC3339>"}` e novas programações devem estar no futuro; horários já vencidos retornam 400. O cron tem cinco campos e usa o fuso local da máquina.

Se a integração repetir uma criação após timeout, reutilize a mesma chave e o mesmo corpo. O replay continua retornando o recurso original mesmo depois do horário único vencer. A mesma chave com corpo diferente retorna conflito. A chave é opcional.

Alterar efetivamente `timing` por PATCH estabelece o início da nova programação no instante da alteração e não recupera horários anteriores. Edições de outros campos e o reenvio de um horário equivalente preservam o cursor de avaliação. Cancelar um schedule impede novos retries automáticos e marca os retries ainda na fila como cancelados; execuções já iniciadas continuam. Retry manual em schedule cancelado retorna 409, sem criar ocorrência.

## Acompanhar uma execução

Guarde o `id` do schedule retornado. Uma execução manual pode ser solicitada com:

```sh
curl --fail-with-body --silent --show-error \
  -X POST http://127.0.0.1:4371/api/v1/schedules/SCHEDULE_ID/runs
```

A resposta contém o identificador da ocorrência. Consulte `GET /api/v1/occurrences/OCCURRENCE_ID` até `status` ser terminal. A ocorrência contém `exitCode`, `stdout`, `stderr` e indicadores de saída truncada.

## Tratar erros

Erros seguem Problem Details e incluem um campo estável `code`. Use o status HTTP para a classe da resposta e `code` para lógica específica; não dependa do texto em `detail`.

Para todas as operações, limites de paginação, esquemas e erros possíveis, consulte o [contrato OpenAPI](../../engineering/api/openapi.yaml). O daemon não escuta em uma interface de rede; o acesso remoto depende do proxy Tailscale do Workspace e das regras de acesso do tailnet. A API não inclui autenticação própria.

## Consultar cockpit, agenda e histórico global

`GET /api/v1/overview?from=...&to=...` retorna contagens de schedules por estado, sucessos e falhas finalizados no intervalo e o próximo horário ativo. `from` e `to` são limites RFC 3339 no intervalo semiaberto `[from,to)`; envie instantes com offset explícito. A UI calcula esses limites no calendário local.

`GET /api/v1/agenda?from=...&to=...` retorna os próximos 500 horários de schedules ativos, selecionados globalmente, e até 500 execuções finalizadas mais recentes no intervalo. A resposta tem no máximo 1000 itens em ordem cronológica. Os dois limites são obrigatórios. Horários cron são calculados no fuso local do daemon.

`GET /api/v1/occurrences` lista o histórico global com paginação (`limit` até 100), e filtros opcionais `status`, `scheduleId`, `from` e `to`. O cursor é opaco: use o `nextCursor` recebido sem interpretá-lo. A resposta de lista omite `stdout` e `stderr` para evitar transferir saídas grandes; consulte `GET /api/v1/occurrences/{occurrenceId}` para obter o resultado completo. O daemon mantém até 90 dias de histórico.

A lista de schedules também aceita `status=active`, `status=paused` ou `status=cancelled`, além de `limit` e `cursor`.
Os cursores de schedules e ocorrências são opacos e baseados na chave de ordenação. Continue usando `nextCursor` sem interpretá-lo.
