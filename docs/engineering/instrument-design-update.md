# Instrument Design · revisão de 2026-10-19

Fontes: REQ-008–011 · JRN-001/016 · TASK-009/010/018.

Workshop continua padrão. Seletor Tema oferece Chalk/Graphite/Workshop e guarda preferência local; falha de localStorage mantém interface funcional. CSS usa papéis semânticos para superfícies, estados e tipografia; movimento reduzido, pressão somente em ações habilitadas e alvos de navegação de 44 px.

A direção distingue espaço editorial calmo e controles/readouts táteis. A skill
`instrument-design` contém a receita, os tokens e critérios de revisão. Paletas
são adaptações originais; não são paletas oficiais Teenage Engineering.

A skill usada nesta revisão é local e depende de referências que não são
redistribuídas com o projeto. Ela não está incluída no repositório; os critérios
de interface e os limites das evidências necessários para manter esta direção
estão resumidos nesta página e no arquivo de revisão.

## Verificação

A revisão de 2026-10-20 está em `instrument-review.json`, com capturas em `instrument-review/` e hashes das fontes. Foram recapturados nove casos da tela inicial, nove do formulário de criação e nove de falha simulada da API, nos três temas e larguras 1440, 390 e 320 px. Enter em formulário vazio foi bloqueado pela validação nativa. Não foram criados schedules nem executados comandos. Busca não foi exercitada neste produto; os demais limites estão explícitos no JSON.
