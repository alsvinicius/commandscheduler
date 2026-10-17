---
name: command-scheduler
description: Integra agentes e aplicações ao Command Scheduler local. Use sempre que o usuário pedir para criar, cadastrar, agendar, pausar, retomar, consultar ou cancelar um schedule, iniciar ou investigar uma execução, consumir a API, ou tirar dúvidas sobre uso e comportamento do Command Scheduler neste repositório. Consulte a documentação Diátaxis do projeto antes de responder ou agir.
---

# Command Scheduler

Ajude agentes a usar o Command Scheduler como cliente local, integrar aplicações pela API e responder dúvidas de uso com base na documentação Diátaxis.

## Manter a documentação alinhada

Ao alterar implementação ou contrato neste repositório, avalie se a mudança afeta comportamento, API, configuração, operação ou fluxo de uso. Se afetar, atualize na mesma tarefa as páginas pertinentes em `docs/diataxis/` e esta skill. Se o contrato HTTP mudar, atualize também `docs/engineering/api/openapi.yaml`. Mantenha a documentação Diátaxis focada em uso e entendimento do sistema; PRD e planejamento continuam nos seus próprios artefatos.

## Fontes de informação

1. Comece por [`docs/diataxis/README.md`](../../../docs/diataxis/README.md) e abra a página adequada à necessidade:
   - tutorial para aprender um fluxo completo;
   - how-to para concluir uma tarefa, incluindo [administrar schedules e execuções](../../../docs/diataxis/how-to/manage-schedules-and-runs.md) ou [instalar e operar o daemon](../../../docs/diataxis/how-to/install-and-operate.md);
   - referência para opções, configuração e contrato, incluindo [schedules e ocorrências](../../../docs/diataxis/reference/schedules-and-occurrences.md);
   - explicação para compreender arquitetura ou execução, incluindo o [mapa técnico do repositório](../../../docs/diataxis/explanation/technical-architecture-and-repository.md).
2. Para campos, rotas, códigos de erro e payloads da API, use [`docs/engineering/api/openapi.yaml`](../../../docs/engineering/api/openapi.yaml), contrato canônico apontado pela documentação Diátaxis.
3. Baseie respostas de uso nas páginas Diátaxis. Se a resposta não estiver coberta, diga o que a documentação não especifica; não deduza comportamento a partir de PRD, plano ou requisitos de produto.

## Entender a intenção antes de criar

Criar um schedule faz o daemon executar um processo local com as permissões do usuário que executa o daemon. Trate a criação como uma ação com efeito persistente.

- Crie, atualize ou cancele recursos somente quando o usuário pedir explicitamente essa ação. Perguntas como “como eu agendaria?” pedem orientação, não criação.
- Antes da criação, identifique o executável, a lista ordenada de argumentos, o horário único ou cron, o fuso relevante e os parâmetros necessários. Peça os detalhes que faltarem e mudariam materialmente o que será executado.
- Não invente comando, caminho de arquivo, horário, recorrência ou parâmetros. Se a tarefa estiver clara, use valores padrão documentados para campos opcionais e descreva-os no resumo.
- Revise o processo e os argumentos com o usuário quando houver ambiguidade sobre efeitos, como apagar ou sobrescrever dados, enviar conteúdo, ou iniciar ações privilegiadas.
- Parâmetros podem conter dados sensíveis. Não os repita em logs, exemplos ou respostas além do necessário para confirmar a ação.

## Conectar ao daemon local

1. Verifique se o daemon já responde em loopback. O endereço padrão é `http://127.0.0.1:4371`; um ambiente `make up` usa `http://127.0.0.1:4372` e banco de desenvolvimento em `.tmp/dev/`.
2. Se o endereço do daemon foi fornecido pelo usuário ou pela configuração do ambiente, use-o. Para a CLI, `--server URL` e `COMMAND_SCHEDULER_URL` selecionam a URL base.
3. Se não houver daemon ativo, não assuma que o banco de desenvolvimento é o destino pretendido. Informe que o serviço local está indisponível. Para experimentos explicitamente de desenvolvimento, o guia Diátaxis documenta `make up` e `make down` no Linux.
4. A API escuta em loopback. `Host` deve ser `localhost`, um endereço numérico de loopback ou um IP Tailscale IPv4 (`100.64.0.0/10`) quando a porta do projeto for encaminhada pelo proxy do Workspace. Clientes web exigem origem HTTP com a mesma autoridade, inclusive no proxy Vite local. Não falsifique `Origin` para contornar a proteção.
5. Use o prefixo `/api/v1` nas rotas. Para o endereço padrão, a base é `http://127.0.0.1:4371/api/v1`. Consulte OpenAPI em `/openapi.yaml` ou a referência navegável em `/docs/api`.

## Criar e administrar schedules

Escolha o cliente que melhor se adapta ao contexto:

- Prefira a CLI instalada quando disponível. Consulte `docs/diataxis/reference/cli.md` para comandos e opções.
- Use a API HTTP local para integrações, chamadas estruturadas ou quando a CLI não estiver disponível. Siga `docs/diataxis/how-to/integrate-with-api.md` e valide cada payload contra OpenAPI.

Na API, `POST /schedules` requer `program`, `args`, `timing` e `missedPolicy`. Envie a chamada a `{base}/schedules`, onde `{base}` termina em `/api/v1`. Uma criação segura para retry pode incluir um `Idempotency-Key` único por operação lógica; reutilize a mesma chave somente com o mesmo corpo. O contrato especifica os códigos de resposta.

Regras que afetam a configuração:

- `program` é o executável; cada elemento de `args` é passado como argumento separado, na ordem dada. O daemon não executa comandos por shell implicitamente.
- Na UI, o campo **Executável** recebe apenas o caminho/nome do programa; cada argumento fica em um item do array JSON **Argumentos**. Se um usuário colar uma linha completa (por exemplo, `python3 -c ...`) em Executável, explique como separá-la antes de testar ou salvar. A UI detecta padrões comuns, mas não interpreta toda sintaxe de shell.
- A UI permite testar o comando antes de salvar via `POST /api/v1/commands/test`. Isso inicia o processo real, com efeitos locais possíveis; não cria schedule/ocorrência, limita a execução a 30 segundos e a saída a 1 MiB por stream. Só acione o teste quando o usuário pedir explicitamente para executar/verificar o comando; não teste por padrão ao criar schedules.
- Horário único usa `{"type":"once","at":"<RFC3339>"}` e novas programações devem estar no futuro; replay idempotente continua válido após vencer o horário. Recorrência usa `{"type":"cron","expression":"<cinco campos>"}` no fuso local do daemon.
- `missedPolicy` aceita `skip`, `consolidate` ou `sequence`; escolha conforme a intenção do usuário e explique consequências quando relevante. Horários únicos devem estar no futuro na criação. Alterar efetivamente `timing` reinicia o cursor de avaliação no instante da mudança. `sequence` processa ocorrências do mesmo schedule em ordem e uma por vez; cancelar o schedule impede retries automáticos.
- `parameters` aceita qualquer valor JSON dentro dos limites do contrato. Atualizações de parâmetros podem disparar execução somente quando o schedule habilita essa opção.
- Caminho executável, PATH, diretório de trabalho e ambiente do daemon podem diferir do terminal interativo do usuário. Prefira caminho absoluto confirmado. Um alias ou função de shell não é um executável.

Após criar ou alterar, confira a resposta e reporte o identificador e a configuração efetivamente aceita. Em caso de timeout de criação, repita com a mesma chave de idempotência e o mesmo corpo para evitar duplicatas. Não repita uma criação com nova chave antes de verificar se a primeira foi aceita.

## Iniciar e acompanhar execuções

- Para iniciar agora, use `POST /schedules/{scheduleId}/runs` ou a CLI `run ID`.
- Guarde o identificador da ocorrência e consulte `GET /occurrences/{occurrenceId}` ou `run-show ID` até o estado ser terminal.
- Para visão consolidada, consulte `GET /overview?from=...&to=...`; para agenda de instâncias futuras use `GET /agenda?from=...&to=...`. Ambos recebem limites RFC 3339; a agenda exige os dois limites.
- Para histórico entre schedules, use `GET /occurrences` com filtros `status`, `scheduleId`, `from` e `to`, e continue pela `nextCursor` sem interpretá-la; cursores são baseados na chave de ordenação. A listagem omite saídas; busque um item por `GET /occurrences/{occurrenceId}` para consultar `stdout`/`stderr`. `GET /schedules` aceita filtro `status`.
- Apresente `status`, código de saída, `stdout` e `stderr` relevantes. Saídas podem ser truncadas; respeite os indicadores do contrato.
- Cancele uma ocorrência apenas quando solicitado; cancelamento é uma solicitação para a árvore de processos, inclusive durante a drenagem dos pipes, e não garantia de que ela já parou. O teste de comando tem prazo total de 30 segundos para processo e saída; preserva saída capturada e indica timeout/truncamento quando incompleta.
- Retry cria outra ocorrência e preserva a anterior. Execução interrompida requer confirmação explícita conforme contrato.

## Responder dúvidas

Localize primeiro a pergunta no índice Diátaxis e responda com a página de uso mais específica. Para dúvidas de API, cite rota e esquema da referência OpenAPI. Para erro de execução, consulte o guia de diagnóstico e explique a diferença entre execução direta e shell.

Para publicação, consulte `docs/diataxis/how-to/publish-open-source.md`, `SECURITY.md` e `THIRD_PARTY_NOTICES.md`. O código é MIT; dependências conservam suas próprias licenças. A checagem cruzada não comprova execução nativa, e os gates de release continuam vigentes. O pacote Cargo não instala a UI; redistribuição web precisa de `ui/dist` com os avisos de terceiros, servidos em `/third-party-notices.txt`.

Separe fatos documentados de inferências. Se a documentação de uso não definir um comportamento ou se a implementação observada divergir dela, diga isso claramente e não apresente uma hipótese como garantia.

## Fechar a interação

Depois de uma ação, informe o que foi criado ou alterado, identificadores retornados e o resultado da execução, se houve. Se não foi possível agir por falta de daemon, dados ou conectividade, explique exatamente o que falta e indique o guia Diátaxis apropriado. Para uma pergunta conceitual, responda diretamente e cite as páginas utilizadas.
