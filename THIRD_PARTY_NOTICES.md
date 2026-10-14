# Componentes de terceiros

A licença MIT do projeto aplica-se ao código próprio. Dependências conservam
suas licenças, avisos e condições de redistribuição.

Os textos e atribuições das dependências Rust e JavaScript estão em
[`ui/public/third-party-notices.txt`](ui/public/third-party-notices.txt).
O build da UI copia esse arquivo para os assets distribuídos e o daemon o
serve em `/third-party-notices.txt`. Inclua também `LICENSE` e este índice ao
redistribuir os binários ou a UI.

Regenere o arquivo após mudar dependências:

```sh
pnpm install --frozen-lockfile
python3 scripts/export_third_party_notices.py
python3 scripts/export_third_party_notices.py --check
```

Textos ausentes nos pacotes publicados são preservados em `licenses/`, com
fontes upstream fixadas por commit em `licenses/upstream.json`. O gerador
falha se uma dependência não tiver licença declarada ou texto de aviso.
