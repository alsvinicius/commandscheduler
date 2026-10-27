# Como iniciar o ambiente de desenvolvimento no Linux

Na raiz do projeto, instale dependências JavaScript uma vez e inicie os serviços:

```sh
pnpm install
make up
```

O comando inicia o daemon em `127.0.0.1:30002` com banco isolado em `.tmp/dev/command-scheduler.sqlite` e o Vite em `127.0.0.1:30014`. Abra `http://127.0.0.1:30014` para usar a interface.

Confira o estado do daemon e examine logs em caso de falha:

```sh
curl --fail http://127.0.0.1:30002/health
tail -n 80 .tmp/dev/backend.log
tail -n 80 .tmp/dev/frontend.log
```

Para iniciar um único componente no terminal atual, use `make backend` ou `make frontend`. `make down` encerra os processos que `make up` iniciou. Os logs e o banco de desenvolvimento permanecem em `.tmp/dev/`.

O Vite encaminha chamadas `/api` e `/health` do endereço `127.0.0.1:30014` ao daemon em `127.0.0.1:30002`. A CLI padrão também usa `30002`, portanto trabalha com o banco `.tmp/dev/` enquanto este daemon estiver ativo. Consulte [Configuração do daemon](../reference/configuration.md).
