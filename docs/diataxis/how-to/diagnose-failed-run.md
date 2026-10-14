# Como diagnosticar uma execução que falhou

Consulte a ocorrência pelo endpoint `GET /api/v1/occurrences/{occurrenceId}` ou pelo comando `command-scheduler-cli run-show OCCURRENCE_ID`. Leia `status`, `exitCode`, `stdout` e `stderr`.

Se a mensagem disser `No such file or directory` ao iniciar o comando, confira `program`: deve ser o caminho de um executável real, acessível ao daemon. Um comando digitado em zsh pode funcionar por alias, função, PATH configurado no shell ou integração com gerenciador de versões sem que o daemon tenha a mesma resolução.

Para Python em Linux, descubra o executável com `command -v python3` no ambiente do daemon e use esse caminho em `program`. Passe `-c` e o código Python como itens distintos de `args`, por exemplo `["-c", "print(\"123\")"]`. Veja [Como executar um programa com argumentos](run-program-with-arguments.md).

O daemon inicia `program` diretamente e não invoca shell implicitamente. Em erros de programa ou argumento, compare a configuração salva no schedule com o contrato [OpenAPI](../../engineering/api/openapi.yaml). Se o processo iniciou e retornou código diferente de zero, use `stderr` e `stdout` para diagnosticar a aplicação executada.
