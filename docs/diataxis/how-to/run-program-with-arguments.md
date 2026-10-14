# Como executar um programa com argumentos

Configure o executável em `program` e cada argumento em um item independente de `args`. O daemon inicia o processo diretamente; ele não interpreta uma linha de comando usando seu shell interativo.

Para executar Python com `-c`, por exemplo, envie:

```json
{
  "program": "/usr/bin/python3",
  "args": ["-c", "print(\"123\")"]
}
```

Na CLI, os mesmos valores são:

```sh
command-scheduler-cli create /usr/bin/python3 \
  --arg=-c \
  --arg='print("123")' \
  --at 2099-01-01T00:00:00Z
```

O exemplo usa `python3` pelo caminho absoluto comum em Linux; confirme o caminho no ambiente em que o daemon executa. Um alias ou função `python` definido no zsh não é um executável que o daemon possa iniciar.

Se o comando depende de pipes, redirecionamento, globbing, expansão de variáveis ou encadeamento, configure o shell como executável e passe a expressão como argumento dele. Isso torna explícita a dependência do shell e evita que a aplicação altere ou reinterprete os argumentos.

## Testar antes de criar pela interface web

No formulário de criação, use **Testar comando** para executar imediatamente o programa e os argumentos configurados. O teste também envia os parâmetros atuais em `COMMAND_SCHEDULER_PARAMS_JSON`. A interface mostra `stdout`, `stderr`, código de saída e indicadores de truncamento; cada stream é limitado a 1 MiB.

No campo **Executável**, informe somente o caminho ou nome do programa, por exemplo `/usr/bin/python3`. No campo **Argumentos**, informe um array JSON de strings, com cada argumento em um item: `["-c", "print('Hello, world!')"]`. Não cole `python3 -c ...` inteiro no campo Executável. A interface sinaliza algumas formas comuns desse erro e bloqueia testar/salvar até separar os campos. Caminhos com espaços são aceitos; a detecção é uma heurística, não um parser universal de comandos.

Este teste inicia o processo real com as permissões do daemon e pode alterar arquivos ou produzir outros efeitos. Ele não cria um schedule nem uma ocorrência no histórico. O prazo de 30 segundos inclui a execução e a leitura das saídas, inclusive pipes herdados por subprocessos. Ao expirar, a árvore de processos é encerrada e `timedOut` é verdadeiro. A saída já capturada é preservada e streams incompletos são marcados como truncados. Um código de saída diferente de zero ou uma falha ao localizar o programa aparece no resultado e não impede, por si só, que você crie o schedule.

Execuções persistentes continuam canceláveis durante a leitura da saída dos descendentes, mesmo depois que o processo pai termina. A drenagem após a saída do pai tem limite de 30 segundos; uma expiração é registrada como falha. O encerramento usa grupos de processos no Unix e Job Objects no Windows.

Clientes locais também podem chamar `POST /api/v1/commands/test`; o corpo e a resposta estão na [referência OpenAPI](../../engineering/api/openapi.yaml). A rota usa os mesmos limites e a política de origem da API.

Veja também [modelo de execução](../explanation/execution-model.md) e a referência de [CLI](../reference/cli.md).
