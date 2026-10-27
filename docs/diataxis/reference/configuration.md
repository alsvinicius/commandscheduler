# Configuração do daemon

Os avisos das dependências distribuídas ficam em `/third-party-notices.txt` quando a UI compilada está instalada. O pacote Cargo contém os binários e licenças; a UI deve ser compilada e instalada separadamente.

O daemon aceita as seguintes variáveis de ambiente:

| Variável | Padrão | Efeito |
|---|---|---|
| `COMMAND_SCHEDULER_ADDR` | `127.0.0.1:30002` | Endereço de escuta do daemon. Deve ser loopback. |
| `COMMAND_SCHEDULER_WEB_DIR` | diretório `ui/dist` empacotado | Diretório de assets da interface. |
| `COMMAND_SCHEDULER_DB_PATH` | diretório de dados local do usuário | Caminho para o arquivo SQLite. |

O diretório de dados padrão é escolhido pela biblioteca `ProjectDirs` para o sistema operacional. As migrações SQLx são aplicadas durante a inicialização. O histórico de ocorrências finalizadas é retido por 90 dias.

Cada banco pode ser usado por um único daemon, independentemente da porta. O daemon mantém um lock no arquivo `<caminho-canônico-do-banco>.lock` antes de aplicar migrações ou recuperar execuções interrompidas. Caminhos simbólicos para o mesmo banco compartilham esse lock. Uma segunda inicialização falha sem alterar ocorrências do daemon ativo.

A API v1 usa o prefixo `/api/v1`. O daemon serve a referência navegável em `/docs/api` e a especificação em `/openapi.yaml`. A escuta é restrita a endereços de loopback. O proxy do Workspace pode encaminhar a porta de um projeto pela interface Tailscale; nesse caso, `Host` também pode conter um IP Tailscale IPv4 (`100.64.0.0/10`). Chamadas originadas em páginas web exigem origem HTTP com a mesma autoridade. A API não inclui autenticação própria; a política de acesso do tailnet controla o acesso remoto.

`make up` inicia a UI em `127.0.0.1:30014` e o daemon em `127.0.0.1:30002`, usando um banco isolado em `.tmp/dev/`. Enquanto esse daemon estiver ativo, a CLI padrão em `30002` acessa esse banco. O `make down` encerra apenas os processos lançados por esse comando.

A CLI configura seu destino por `--server URL` ou `COMMAND_SCHEDULER_URL`; seu padrão é `http://127.0.0.1:30002`. O contrato HTTP está em [OpenAPI 3.1](../../engineering/api/openapi.yaml).
