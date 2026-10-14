# Como o Command Scheduler executa comandos

Um schedule guarda o caminho do executável em `program` e uma lista ordenada de argumentos em `args`. Quando chega o horário ou um cliente solicita execução manual, o daemon cria um processo local usando esses valores diretamente. Não há expansão ou interpretação implícita por shell.

Essa separação corresponde à forma como o sistema operacional recebe um processo: um executável e uma lista de argumentos. Ao receber uma string como `python -c 'print("123")'` inteira em `program`, o daemon procura um arquivo com esse nome literal. Um shell interativo pode, por sua vez, dividir o texto, resolver aliases e aplicar configurações do ambiente antes de iniciar o programa; por isso, os dois caminhos não são equivalentes.

O daemon executa com as permissões do usuário que o iniciou. O ambiente de serviço pode ter PATH, diretório de trabalho e variáveis diferentes do terminal interativo. Caminhos absolutos reduzem ambiguidades de resolução. Scripting, pipes, redirecionamento e expansão devem ser pedidos explicitamente ao shell como programa.

Parâmetros do schedule são disponibilizados como JSON em `COMMAND_SCHEDULER_PARAMS_JSON`; não são interpolados em `program` ou `args`. Para metadados de horários consolidados, o daemon fornece variáveis `COMMAND_SCHEDULER_MISSED_*`. `stdout` e `stderr` são capturados, cada um limitado a 1 MiB por ocorrência.

Consulte [Como executar um programa com argumentos](../how-to/run-program-with-arguments.md) para configurar um caso concreto e o [contrato API](../../engineering/api/openapi.yaml) para os campos serializados.
