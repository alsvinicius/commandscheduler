# Como criar um schedule pela interface web

Use a interface local para configurar um comando, testar seus argumentos e salvar um horário recorrente ou único.

## Abrir o formulário

Abra o daemon local e selecione **Novo schedule**. Em **Executável**, informe somente o caminho ou nome do programa. Coloque cada opção e argumento em um item do array JSON. Por exemplo, para imprimir uma mensagem com Python:

- Executável: `/usr/bin/python3`
- Argumentos: `["-c", "print('Olá!')"]`

Use **Testar comando** para executar o processo com a configuração atual. O processo real pode ter efeitos locais; o teste não cria schedule nem ocorrência no histórico, termina após 30 segundos e limita cada stream a 1 MiB.

## Configurar o horário

Selecione **Recorrente** e escolha uma opção em **Repetição**:

- **A cada N minutos** ou **A cada N horas**: selecione um intervalo permitido. A repetição segue os limites do relógio local.
- **Diariamente**: informe a hora local.
- **Dias da semana**: informe a hora local e marque os dias desejados.
- **Mensalmente**: informe a hora local e o dia do mês.
- **Cron avançado**: informe uma expressão cron padrão com cinco campos. A prévia das opções simples mostra a expressão que será salva.

Escolha **Uma vez** para informar uma data e hora local. Revise a política de horários vencidos e os parâmetros antes de criar.

## Consultar o schedule

Após salvar, o schedule aparece na lista e no cockpit. A lista permite filtrar ativos, pausados e cancelados. A agenda apresenta instâncias futuras por dia, semana ou mês. O histórico global aceita filtros por período, estado e identificador de schedule; use **Carregar mais execuções** para consultar páginas seguintes.

Para alterar um schedule existente, abra-o na lista e selecione **Editar**. A UI reconhece presets comuns de cron; expressões fora desses formatos aparecem como **Cron avançado**.

Para os detalhes dos campos e das operações HTTP, consulte a [referência OpenAPI](../../engineering/api/openapi.yaml). Para separar executável e argumentos, veja [como executar um programa com argumentos](run-program-with-arguments.md).
