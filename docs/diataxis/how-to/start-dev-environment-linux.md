# Como iniciar o ambiente de desenvolvimento no Linux

Na raiz do projeto, instale dependências JavaScript uma vez e inicie os serviços:

```sh
pnpm install
make up
```

O comando inicia o daemon em `127.0.0.1:4372` com banco isolado em `.tmp/dev/command-scheduler.sqlite` e o Vite em `127.0.0.1:5173`. Abra `http://127.0.0.1:5173` para usar a interface.

Confira o estado do daemon e examine logs em caso de falha:

```sh
curl --fail http://127.0.0.1:4372/health
tail -n 80 .tmp/dev/backend.log
tail -n 80 .tmp/dev/frontend.log
```

Para iniciar um único componente no terminal atual, use `make backend` ou `make frontend`. `make down` encerra os processos que `make up` iniciou. Os logs e o banco de desenvolvimento permanecem em `.tmp/dev/`.

O Vite encaminha chamadas `/api` e `/health` ao daemon de desenvolvimento. A API padrão do binário, fora desse arranjo, usa `127.0.0.1:4371`; detalhes de configuração estão em [Configuração do daemon](../reference/configuration.md).
