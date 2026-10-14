# Decisões técnicas — Interfaces e qualidade

- Estado: ready
- Revisão: 8
- Histórico de revisões: [registro central](../../revision-history.md) — seção `engineering/decisions/interfaces-and-quality.md`
- Atualizado: 2026-10-13
- Fontes e revisões: [PRD r19](../../product/prd.md), [requisitos r5](../../product/requirements.md), [jornadas r32](../../product/user-journeys.md)
- Artefatos relacionados: [definição técnica](../technical-definition.md), [estado do workflow](../../product/workflow-state.md)
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.

### DEC-002 — Restringir a API local sem autenticação

- Estado: confirmada
- Fontes: REQ-002, REQ-003; JRN-002, JRN-006, JRN-010–JRN-012
- Decisão e alternativas: API do MVP aceita conexões somente por localhost sem autenticação; acesso por rede foi descartado para o MVP.
- Justificativa e consequências: confirmado pelo usuário no PRD. Chamadas diretas de programas locais são aceitas; chamadas web são limitadas à origem da UI local conforme DEC-020.
- Evidência: PRD r5.
- Implementação no MVP e extensão futura: restrição localhost no MVP; autenticação e acesso remoto futuros não estão no escopo confirmado.

### DEC-040 — Testar comandos sem persistir ocorrência

- Estado: confirmada
- Fontes: REQ-007, JRN-001; pedido e confirmação do usuário em 2026-10-13.
- Decisão e alternativas: a UI oferece um botão para executar o comando do formulário imediatamente. Uma rota transitória retorna código de saída, `stdout`, `stderr` e truncamento. O resultado não vira schedule nem ocorrência persistida.
- Justificativa e consequências: permite verificar executável e argumentos antes de criar schedule, sem poluir histórico. Como o processo é real e usa permissões do daemon, a interface avisa sobre efeitos locais. O daemon encerra o processo após 30 segundos e limita cada stream capturado a 1 MiB.
- Evidência: confirmação do usuário para execução real com aviso e sem persistência; limite de 30 segundos escolhido pelo usuário.
- Implementação no MVP e extensão futura: `POST /api/v1/commands/test`; sem shell implícito, timeout fixo e sem retry. A UI separa e explica os campos Executável/Argumentos, dá um exemplo Python e bloqueia padrões comuns de linha completa no executável sem rejeitar genericamente caminhos com espaços.

### DEC-041 — Oferecer presets locais sem alterar o contrato de recorrência

- Estado: confirmada.
- Fontes: REQ-008, JRN-001; escolha do usuário por presets com cron avançado e diretriz de usar calendário local, 2026-10-13.
- Decisão e alternativas: formulários web de criação/edição oferecem “a cada N minutos” (1, 2, 3, 4, 5, 6, 10, 12, 15, 20 ou 30), “a cada N horas” (1, 2, 3, 4, 6, 8 ou 12), horário diário, dias da semana com horário, dia do mês com horário e modo cron avançado. Intervalos seguem limites do relógio/calendário local, com N divisível por 60 minutos ou 24 horas, e são representados internamente por expressões cron de cinco campos. Não introduzir formato público alternativo na API.
- Justificativa e consequências: reduz a necessidade de conhecer cron em configurações comuns, mantendo sua expressividade para casos avançados e compatibilidade do contrato externo existente. Valores ausentes no calendário mensal mantêm a semântica atual do cron.
- Evidência: pedido de melhorar a escolha do intervalo, seleção explícita de presets com cron avançado e confirmação pelo usuário de que os intervalos devem seguir o relógio local, em 2026-10-13.
- Implementação no MVP e extensão futura: Vue apresenta formulário de frequência e resumo legível; conversão para cron ocorre no cliente e os campos de criação/edição/API existentes continuam sendo a fonte persistida.

### DEC-042 — Servir cockpit, agenda e histórico por consultas globais paginadas

- Estado: proposta técnica para o MVP, consistente com requisitos confirmados.
- Fontes: REQ-009–REQ-011, JRN-016; endpoints existentes de listagem paginada e API local.
- Decisão e alternativas: estender a API local/OpenAPI com consultas somente leitura para agregados do cockpit e atividade global por período; suportar filtro de estado na lista de schedules e consulta global de ocorrências paginada, com filtros de período, estado e schedule. Não agregar centenas de execuções carregando todos os registros no navegador.
- Justificativa e consequências: fornece visão global e previsível de memória/carga para UI com histórico extenso. Não muda criação ou formato público de timing de schedules. Agregados usam períodos no fuso local, contam cada ocorrência/retry individualmente, `succeeded` como sucesso e `failed` como falha, e respeitam retenção de 90 dias.
- Evidência: a inspeção do backend confirma paginação de ocorrências apenas por schedule e ausência de filtro de estado na listagem de schedules; a jornada pede consulta global para cockpit/agenda/histórico.
- Implementação no MVP e extensão futura: implementar os recursos de leitura na API e documentá-los no OpenAPI ao executar TASK-017; detalhes de rotas e paginação ficam sob autoridade da tarefa, respeitando os filtros e agregações definidos aqui.
### DEC-009 — Usar SQLite como armazenamento local

- Estado: confirmada
- Fontes: REQ-001, REQ-002; JRN-001, JRN-002, JRN-004, JRN-010
- Decisão e alternativas: SQLite embutido para schedules, parâmetros e histórico de execuções. Alternativas consideradas: arquivos JSON, que exigiriam mais cuidado com atualizações transacionais e recuperação; banco servidor, que adicionaria um serviço externo ao produto local.
- Justificativa e consequências: SQLite é autocontido, serverless e transacional, características compatíveis com um daemon local persistente sem dependência de banco separado. Concorrência de escritas, migrações, backup, retenção e recuperação de corrupção exigem implementação cuidadosa. SQLx está definido para acesso Rust; estratégia de backup/restore guiado fica fora do MVP.
- Evidência: [SQLite — About](https://sqlite.org/about.html) descreve SQLite como engine embutida/serverless e transacional; [SQLite — Serverless](https://www.sqlite.org/serverless.html).
- Implementação no MVP e extensão futura: SQLite para persistência local no MVP, com SQLx, migrações versionadas e histórico de execuções retido por 90 dias; backup/restore guiado é candidato futuro.
### DEC-020 — Restringir chamadas web à origem da UI local

- Estado: confirmada
- Fontes: REQ-002, REQ-003; JRN-001, JRN-002, JRN-006, JRN-010
- Decisão e alternativas: não adicionar autenticação à API local no MVP; permitir chamadas diretas de programas locais e aceitar chamadas de navegador somente a partir da origem local da UI, sem CORS amplo.
- Justificativa e consequências: decisão explícita do usuário; origens web externas não devem controlar schedules via navegador. Processos locais continuam dentro do limite de confiança assumido pelo MVP.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-12.
- Implementação no MVP e extensão futura: aplicar restrição de origem para chamadas de browser e manter API local sem autenticação.
### DEC-021 — Suportar versões dos sistemas cobertas pelo fornecedor

- Estado: confirmada
- Fontes: REQ-001, REQ-002; JRN-001, JRN-002
- Decisão e alternativas: suportar versões de Windows, macOS e Linux que recebam atualizações de segurança dos fornecedores na data de lançamento do MVP; não fixar agora números de versão.
- Justificativa e consequências: decisão explícita do usuário; matriz exata de versões deve ser atualizada de acordo com a data de release.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-12.
- Implementação no MVP e extensão futura: documentar versões efetivamente verificadas na liberação.
### DEC-022 — Automatizar BDD em CI multiplataforma

- Estado: confirmada
- Fontes: REQ-001–REQ-006; JRN-001–JRN-012
- Decisão e alternativas: cenários BDD vinculados às jornadas devem ter automação correspondente e executar em CI para Windows, macOS e Linux.
- Justificativa e consequências: decisão explícita do usuário para facilitar rastreabilidade e garantir cobertura nos sistemas-alvo; provedor de CI e ferramentas de teste dependem da stack de implementação.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-12.
- Implementação no MVP e extensão futura: suíte de aceitação derivada dos BDD com matriz de três sistemas no MVP.
### DEC-024 — Aceitar chave de idempotência opcional na criação via API

- Estado: confirmada
- Fontes: REQ-003; JRN-002
- Decisão e alternativas: criação de schedule pela API aceita chave de idempotência opcional; mesma chave e mesmo conteúdo retornam o resultado original, sem criar outro schedule; mesma chave com conteúdo diferente é conflito. A chave permanece enquanto o schedule existir.
- Justificativa e consequências: decisão confirmada para reduzir duplicações após timeout; clientes devem gerar chaves únicas por criação lógica e não reutilizá-las para conteúdo diferente.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-12.
- Implementação no MVP e extensão futura: chave opcional no endpoint de criação, persistida enquanto o schedule correspondente existir.
### DEC-027 — Usar contrato HTTP REST/JSON para API local

- Estado: confirmada
- Fontes: REQ-002–REQ-006; JRN-002–JRN-012
- Decisão e alternativas: API HTTP REST/JSON v1; header opcional `Idempotency-Key` na criação; dados consolidados em `missedOccurrences: { count, from, through }`, com datas RFC 3339. Publicar OpenAPI 3.1 versionado com schemas, erros e exemplos, e referência navegável na UI local.
- Justificativa e consequências: contrato JSON é adequado para integrar aplicações locais em diferentes linguagens. Nomes claros e header padronizado tornam retries e metadados fáceis de consumir.
- Evidência: resposta do usuário durante a entrevista técnica, 2026-10-12.
- Implementação no MVP e extensão futura: OpenAPI 3.1 completo e navegável no MVP; paginação e códigos HTTP concretos serão documentados no contrato.
### DEC-028 — Limitar e validar parâmetros JSON

- Estado: confirmada
- Fontes: REQ-004; JRN-002, JRN-003, JRN-007
- Decisão e alternativas: aceitar qualquer valor JSON válido, sem schema obrigatório nem exigência de objeto na raiz; limitar a 64 KiB por schedule na criação e nas atualizações.
- Justificativa e consequências: mantém flexibilidade para integrações; JSON inválido ou payload maior que o limite é rejeitado sem substituir parâmetros aceitos anteriormente.
- Evidência: resposta do usuário durante a definição técnica, 2026-10-12.
- Implementação no MVP e extensão futura: aplicar validação e limite igualmente na API e UI; serializar valor aceito em `COMMAND_SCHEDULER_PARAMS_JSON`.
### DEC-032 — Publicar referência OpenAPI 3.1 completa

- Estado: confirmada
- Fontes: REQ-002, REQ-003; JRN-002, JRN-006–JRN-007, JRN-010–JRN-012
- Decisão e alternativas: manter especificação OpenAPI 3.1 versionada junto ao código, cobrindo schemas, erros e exemplos, e disponibilizar referência navegável na UI local.
- Justificativa e consequências: facilita integração por aplicações locais e mantém contrato revisável junto à implementação.
- Evidência: resposta do usuário durante a definição técnica, 2026-10-12.
- Implementação no MVP e extensão futura: especificação completa e página navegável no MVP; ferramenta de geração/validação será escolhida na implementação.
### DEC-033 — Definir meta de pontualidade e ausência de alvo de carga

- Estado: confirmada
- Fontes: REQ-001; JRN-001–JRN-002
- Decisão e alternativas: em máquina ociosa, medir atraso entre horário previsto e início real; meta P95 ≤ 5 s e P99 ≤ 30 s. Não há meta formal de volume nem carga de referência no MVP.
- Justificativa e consequências: cria critério verificável para operação local sem afirmar capacidade sob carga não especificada.
- Evidência: respostas do usuário durante a definição técnica, 2026-10-12.
- Implementação no MVP e extensão futura: verificar pontualidade em ambiente de teste ocioso; não assumir garantia sob saturação/concorrência.
### DEC-034 — Adotar requisitos básicos de acessibilidade sem certificação

- Estado: confirmada
- Fontes: REQ-002; JRN-001, JRN-004–JRN-005, JRN-008–JRN-009
- Decisão e alternativas: sem meta formal de conformidade WCAG no MVP; interface deve permitir navegação por teclado e fornecer rótulos acessíveis aos controles.
- Justificativa e consequências: assegura uso básico sem assumir auditoria ou certificação formal.
- Evidência: respostas do usuário durante a definição técnica, 2026-10-12.
- Implementação no MVP e extensão futura: teclado e rótulos acessíveis incluídos nos critérios da interface e verificação.
### DEC-035 — Definir disponibilidade mensal local em 99,5%

- Estado: confirmada
- Fontes: REQ-001; JRN-001, JRN-002, JRN-004
- Decisão e alternativas: disponibilidade mensal alvo de 99,5%, calculada somente enquanto a máquina estiver ligada e apta a executar schedules; períodos desligados/suspensos são excluídos. O daemon reinicia automaticamente após falha e recupera schedules persistidos.
- Justificativa e consequências: adequa a meta ao escopo local, sem penalizar períodos em que a máquina não pode executar; requer inicialização supervisionada e recuperação do estado persistido.
- Evidência: respostas do usuário durante a definição técnica, 2026-10-12.
- Implementação no MVP e extensão futura: documentar e medir disponibilidade conforme esse denominador; implementar reinício supervisionado e recuperação dos schedules.
