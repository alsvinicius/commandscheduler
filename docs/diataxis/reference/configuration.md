# Configuração do daemon

Os avisos das dependências distribuídas ficam em `/third-party-notices.txt` quando a UI compilada está instalada. O pacote Cargo contém os binários e licenças; a UI deve ser compilada e instalada separadamente.

O daemon aceita as seguintes variáveis de ambiente:

| Variável | Padrão | Efeito |
|---|---|---|
| `COMMAND_SCHEDULER_ADDR` | `127.0.0.1:4371` | Endereço de escuta. Deve ser loopback. |
| `COMMAND_SCHEDULER_WEB_DIR` | diretório `ui/dist` empacotado | Diretório de assets da interface. |
| `COMMAND_SCHEDULER_DB_PATH` | diretório de dados local do usuário | Caminho para o arquivo SQLite. |

O diretório de dados padrão é escolhido pela biblioteca `ProjectDirs` para o sistema operacional. As migrações SQLx são aplicadas durante a inicialização. O histórico de ocorrências finalizadas é retido por 90 dias.

Cada banco pode ser usado por um único daemon, independentemente da porta. O daemon mantém um lock no arquivo `<caminho-canônico-do-banco>.lock` antes de aplicar migrações ou recuperar execuções interrompidas. Caminhos simbólicos para o mesmo banco compartilham esse lock. Uma segunda inicialização falha sem alterar ocorrências do daemon ativo.

A API v1 usa o prefixo `/api/v1`. O daemon serve a referência navegável em `/docs/api` e a especificação em `/openapi.yaml`. A escuta é restrita a endereços de loopback. Clientes locais nativos não usam autenticação; `Host` deve ser `localhost` ou um endereço numérico de loopback. Chamadas originadas em páginas web exigem origem HTTP com o mesmo host e porta.

`make up` configura endereço `127.0.0.1:4372`, define um banco em `.tmp/dev/` e aponta a UI de desenvolvimento para esse daemon. O `make down` encerra apenas os processos lançados por esse comando.

A CLI configura seu destino por `--server URL` ou `COMMAND_SCHEDULER_URL`; seu padrão é `http://127.0.0.1:4371`. O contrato HTTP está em [OpenAPI 3.1](../../engineering/api/openapi.yaml).
