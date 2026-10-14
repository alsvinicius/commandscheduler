# Como publicar o código no GitHub

Use este procedimento para abrir o código como MVP em desenvolvimento em
`https://github.com/alsvinicius/commandscheduler`.

## Preparar os arquivos

Mantenha `LICENSE`, `SECURITY.md`, `THIRD_PARTY_NOTICES.md`, os avisos em
`ui/public/third-party-notices.txt`, os lockfiles e os workflows no repositório.
Não adicione bancos SQLite, parâmetros reais, saídas de comandos, `.env`,
logs ou diretórios de build/dependências. A `.gitignore` exclui esses arquivos
locais; ela não remove um arquivo que já esteja versionado.

Depois de instalar as ferramentas descritas no [guia de contribuição](../../../CONTRIBUTING.md):

```sh
pnpm install --frozen-lockfile
python3 scripts/export_third_party_notices.py --check
python3 scripts/check_publication.py
cargo audit
pnpm audit
```

`cargo audit` é instalado com `cargo install cargo-audit --version 0.22.2 --locked`.
As duas auditorias devem terminar sem vulnerabilidades conhecidas e sem
exceções. Mudanças de dependências exigem regenerar os avisos de terceiros
antes de compilar a UI. `check_publication.py` confere também links locais e
que todas as migrações SQL estão embutidas no daemon.

## Criar e revisar o repositório

Crie o repositório no GitHub, habilite **Settings → Security → Private
vulnerability reporting** e envie apenas os arquivos revisados. Antes do push,
revise a lista de arquivos preparados pelo Git e faça a varredura local com
Gitleaks. A varredura do diretório atual não comprova a limpeza de um histórico
Git antigo.

Os workflows `CI` e `Security` rodam após o push. `Security` varre o histórico
Git completo e consulta as bases atuais de vulnerabilidades; também repete
a auditoria semanalmente. Revise os resultados, corrija qualquer falha e
configure as verificações como obrigatórias antes de aceitar merges.

## Anunciar o estado comprovado

Informe no README que o projeto está em desenvolvimento. Publicar o código
com licença MIT permite contribuições; o anúncio de release deve respeitar
os gates registrados no [estado do workflow](../../product/workflow-state.md).
Compatibilidade multiplataforma requer sucesso da matriz nativa Linux,
macOS e Windows, inclusive processos descendentes e a tarefa contínua Windows.
Compilação cruzada no Linux não comprova esses comportamentos nativos.

As metas de pontualidade e disponibilidade precisam das medições e janelas
descritas nos guias correspondentes. Não anuncie disponibilidade mensal
antes de completar essa janela.

O pacote Cargo contém daemon/CLI, migrações, contrato e licenças. Ele não
inclui a aplicação web compilada. Para distribuir a interface, compile com
`pnpm ui:build` e distribua `ui/dist` com o daemon, conforme os guias de
instalação; a instalação por `cargo install` isoladamente não instala a UI.
