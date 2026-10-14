# Cenários BDD executáveis

Os testes sobem um daemon real em loopback com SQLite temporário e exercitam os endpoints, o processo filho, a CLI e a UI em Chromium. Os comentários `Covers:` ligam testes Python e Playwright a IDs de `docs/product/journeys/`. `check_coverage.py` valida os links e informa quantos IDs ainda faltam; não considera os cenários ausentes cobertos.

Execute depois de compilar daemon, CLI e UI:

```sh
pnpm exec playwright install chromium
python3 -m unittest discover -s tests/bdd -v
pnpm bdd:web
python3 tests/bdd/check_coverage.py
```

A matriz GitHub Actions instala Chromium e executa as suítes Python e Playwright em Linux, macOS e Windows. A cobertura total é calculada pelo verificador; cenários sem teste ligado continuam pendentes para o gate BDD de release.
