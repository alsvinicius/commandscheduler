# Histórico de revisões

Registro central de alterações dos artefatos. Cada seção usa o caminho do documento como identificador estável.

## `product/prd.md`

- r19 — 2026-10-13: amplia o MVP com presets de recorrência, cockpit operacional, agenda global e histórico filtrável/paginado.

- r18 — 2026-10-13: inclui teste transitório de comando na UI antes de criar schedule.

- r1 — 2026-10-12: primeiro draft criado a partir do pedido inicial.
- r2 — 2026-10-12: incorporadas respostas sobre execução na máquina, API, gerenciamento via interface e API, schedules por horário, atualização de parâmetros e gatilho de execução após atualização.
- r3 — 2026-10-12: definidos público e problema prioritários, interface web, schedules únicos e recorrentes por calendário, controles de execução, semântica de gatilhos/parâmetros e ausência de autenticação no MVP; pendente delimitar exposição da API e decisões operacionais finais.
- r4 — 2026-10-12: API sem autenticação restrita a localhost, fuso horário da máquina e concorrência de execuções confirmados.
- r5 — 2026-10-12: definidos prazo para atualização dos parâmetros, métrica de sucesso e permissões locais; PRD considerado pronto, com meta quantitativa e regra de horário de verão como decisões não bloqueadoras.
- r6 — 2026-10-12: confirmado que schedules/configurações persistem após reinício e que o tratamento de ocorrências vencidas é configurável por schedule.
- r7 — 2026-10-12: definidas as opções de recuperação de ocorrências vencidas configuráveis por schedule: pular, consolidar em uma execução ou executar todas em sequência.
- r8 — 2026-10-12: incorporadas decisões técnicas confirmadas sobre daemon multiplataforma, instalação manual, limite/metadados de consolidação, retry padrão e interrupção, retenção de 90 dias e horário de verão.
- r9 — 2026-10-12: incorporadas restrição de origem da API, versões de SO mantidas pelos fornecedores e automação BDD em CI multiplataforma.
- r10 — 2026-10-12: definida passagem de parâmetros JSON ao processo e chave de idempotência opcional para criação de schedules via API.
- r11 — 2026-10-12: registrados nomes das variáveis de ambiente, semântica de retry e comportamento de repetição de chave idempotente.
- r12 — 2026-10-12: stack Axum/Tokio/SQLx, API REST/JSON, formato program + args e esquema de metadados da ocorrência consolidada confirmados.
- r13 — 2026-10-12: parâmetros JSON limitados a 64 KiB; política de vencidas aplicada também a suspensão/atraso temporário; concorrência sem limite global; captura e retenção de saída definidas; OpenAPI 3.1 completo; metas de qualidade e denominador de disponibilidade registrados. Métrica de migração permanece sem meta ou período.
- r14 — 2026-10-13: reconciliação da criação via terminal registrada como CLI local; corrigida referência desatualizada de disponibilidade como não definida.
- r15 — 2026-10-13: CLI incorporada ao critério de aceitação de REQ-001; escopo do MVP e exclusões reescritos como decisões fechadas, sem linguagem de provisório ou não acordado.
- r16 — 2026-10-13: CLI ampliada de criação para gerenciamento de schedules e execuções; requisitos e canais de gestão atualizados.

- r17 — 2026-10-13: requisitos detalhados movidos para catálogo próprio; visão do PRD reduzida; métrica sem alvo delimitada como observacional no MVP, sem alegação contra meta inexistente.

## `product/user-journeys.md`

- r34 — 2026-10-13: sincroniza fontes para definição técnica r61 após implementar e validar localmente os cenários JRN-001/JRN-016.

- r33 — 2026-10-13: sincroniza fontes com requisitos r5 e definição técnica r60 após confirmação da semântica local dos intervalos.

- r32 — 2026-10-13: adiciona JRN-016, BDDs de cockpit/agenda/histórico e atualiza a matriz para 16 jornadas/100 cenários.

- r31 — 2026-10-13: adiciona BDD-JRN-001-16 e atualiza total para 91 cenários.

- r30 — 2026-10-13: atualiza rastreabilidade do REQ-007 e total para 91 cenários BDD.

- r29 — 2026-10-13: inclui o teste de comando na cobertura de produto e adiciona BDD-JRN-001-15.

- r28 — 2026-10-13: total BDD recalculado (89) após cenários de retomada e fontes técnicas sincronizadas.

- r27 — 2026-10-13: retomada após pausa aplica política de ocorrências vencidas salva, DEC-039; BDD atualizado.

- r26 — 2026-10-13: jornadas divididas por canal; cenários BDD identificados, estruturados em Dado/Quando/Então e ligados a requisitos; fontes reconciliadas para PRD r17 e definição técnica r40.

- r1 — 2026-10-12: criadas quatro jornadas amplas derivadas do PRD r5.
- r2 — 2026-10-12: detalhadas em nove jornadas independentes, separando criação, manutenção, atualização/gatilho, consulta, controle e recuperação; requisito REQ-001 a REQ-006 remapeado.
- r3 — 2026-10-12: canais web e API descritos nas jornadas de gestão e execução; adicionados cenários BDD de sucesso e exceção para facilitar rastreabilidade dos testes.
- r4 — 2026-10-12: consulta, execução/cancelamento e recuperação separados em jornadas próprias por canal; cobertura de requisitos atualizada.
- r5 — 2026-10-12: adicionado índice por canal para facilitar navegação e vínculo futuro com cenários BDD.
- r6 — 2026-10-12: incluída persistência pós-reinício e configuração por schedule para ocorrências vencidas, em jornadas web e API.
- r7 — 2026-10-12: cenários BDD cobrem as três políticas configuráveis; fonte atualizada para PRD r7.
- r8 — 2026-10-13: adicionados cenários BDD para consolidação automática acima de 100 vencidas, confirmação manual de retry após interrupção e retenção de 90 dias.
- r9 — 2026-10-13: referência técnica atualizada para definição r12.
- r10 — 2026-10-13: adicionados cenários BDD para metadados de consolidação, retry padrão e transição de horário local.
- r11 — 2026-10-12: PRD r9 e definição técnica r15 reconciliados; segurança de origem, retenção, metadados de processo/API e BDD em CI multiplataforma incorporados.
- r12 — 2026-10-12: referência técnica atualizada para definição r16.
- r13 — 2026-10-12: contratos de parâmetros JSON e idempotência opcional incorporados; referências reconciliadas para PRD r10 e definição técnica r18.
- r14 — 2026-10-12: nomes de variáveis, semântica de três retries adicionais e comportamento de chave idempotente confirmados; referências para PRD r11 e definição técnica r21.
- r15 — 2026-10-12: stack Rust, API REST/JSON, `Idempotency-Key`, campos de consolidação e comando program + args confirmados; fontes atualizadas para PRD r12 e definição técnica r24.
- r16 — 2026-10-12: referência de definição técnica atualizada para r25.
- r17 — 2026-10-12: referências atualizadas para definição técnica r26; retenção, retries e contratos já definidos removidos da lista de pendências.
- r18 — 2026-10-12: fonte técnica atualizada para r27.
- r19 — 2026-10-12: critérios BDD e referências atualizados para PRD r13/definição técnica r28; incluídos limite/validação JSON, ocorrências atrasadas, stdout/stderr, acessibilidade básica e metas de qualidade.
- r20 — 2026-10-12: adicionados cenários BDD de recuperação do daemon, pontualidade, referência OpenAPI e concorrência sem limite global.
- r21 — 2026-10-13: reconciliado o requisito de criação via terminal do AGENTS.md; adicionada JRN-013 CLI e removidas lacunas antigas de parâmetros/recuperação já decididas.
- r22 — 2026-10-13: referências sincronizadas com PRD r15 e definição técnica r34 após auditoria de prontidão.
- r23 — 2026-10-13: CLI ampliada para gerenciamento; adicionadas jornadas BDD para manutenção de schedules e controle/inspeção de execuções.
- r24 — 2026-10-13: fonte técnica atualizada para r36 após formalização da paridade operacional da CLI.
- r25 — 2026-10-13: BDD CLI inclui cancelamento de schedule e visibilidade da criação nos três canais.

## `engineering/technical-definition.md`

- r61 — 2026-10-13: registra endpoints implementados para cockpit, agenda e histórico; paginação, filtros e carregamento sob demanda de saídas.

- r60 — 2026-10-13: registra como confirmada a semântica de intervalo local e os valores de presets que mantêm cadência regular.

- r59 — 2026-10-13: reconcilia PRD r19/jornadas r32, registra DEC-041/042 e especifica agregados/API de agenda e histórico paginado.

- r58 — 2026-10-13: sincroniza rastreabilidade para jornadas web r6 e índice r31.

- r57 — 2026-10-13: alinha a definição técnica à revisão dos critérios UI de REQ-007.

- r56 — 2026-10-13: especifica endpoint transitório de teste, saída limitada e timeout confirmado de 30 s.

- r55 — 2026-10-13: padroniza o nome JSON público `missedOccurrences` após validar o contrato recuperado via API/UI.

- r54 — 2026-10-13: registra reprodução da instalação final systemd Linux com API/UI e persistência após restart.

- r53 — 2026-10-13: esclarece que a CLI usa HTTP loopback, conforme o daemon local sem TLS.

- r52 — 2026-10-13: registra restrição HTTP loopback e timeout de 30 s da CLI.

- r51 — 2026-10-13: registra smoke Linux de health/SIGTERM e limite da verificação do unit systemd.

- r50 — 2026-10-13: registra descoberta de assets ao lado do binário e referências de supervisor nos três SOs.

- r49 — 2026-10-13: registra tratamento SIGTERM/SIGINT e serviço systemd local para Linux.

- r48 — 2026-10-13: fontes de jornadas atualizadas para r28 e DEC-039 propagada.

- r47 — 2026-10-13: registra a semântica de retomada pós-pausa da DEC-039.

- r46 — 2026-10-13: integra recorrência/DST, executor, captura/cancelamento/retries e loop do daemon ao desenho técnico.

- r45 — 2026-10-13: registra cursor persistente, migração aditiva v2 e implementação inicial do cron/DST/recuperação da TASK-004.

- r44 — 2026-10-13: TASK-003 registra o caminho SQLite por usuário, `COMMAND_SCHEDULER_DB_PATH`, migrações embutidas, WAL, timestamps em microssegundos e rotina de retenção.
- r43 — 2026-10-13: TASK-002 concluída; contrato OpenAPI e referência local registrados, cron padrão de cinco campos escolhido em DEC-037 e validação automatizada documentada.
- r42 — 2026-10-13: orientação temporária do usuário permite validação Linux-only durante implementação; gates Windows/macOS e CI BDD tripla preservados no release.
- r41 — 2026-10-13: estrutura de projeto iniciada; comandos, dependências e evidência de build local registrados; compilação Windows/macOS permanece pendente.

- r40 — 2026-10-13: registros DEC-001–DEC-036 movidos para documentos por tema; fontes reconciliadas com PRD r17 e jornadas r26.

- r1 — 2026-10-12: iniciado draft técnico com restrições confirmadas, direção arquitetural provisória e entrevista pendente.
- r2 — 2026-10-12: serviço/daemon automático e suporte a Windows/macOS/Linux confirmados; JavaScript, Python e Rust registrados como opções ainda sem preferência escolhida.
- r3 — 2026-10-12: removidas perguntas respondidas sobre modelo de execução e suporte a SO; daemon confirmado separado da proposta de runtime local único.
- r4 — 2026-10-12: Rust confirmado como preferência, UI no navegador local, schedules persistentes e política de ocorrência vencida configurável por schedule; PRD e jornadas atualizados para r6.
- r5 — 2026-10-12: PRD/jornadas r7 incorporam as três políticas de ocorrências vencidas; preferência por Rust com apoio de IA e TypeScript como opção para UI registrados.
- r6 — 2026-10-12: armazenamento local delegado para recomendação; SQLite registrado como proposta técnica com alternativas e fontes oficiais; perguntas de recuperação e parâmetros por ocorrência refinadas.
- r7 — 2026-10-13: parâmetros mais recentes confirmados para recuperação vencida; retry configurado confirmado para execuções interrompidas; instalação manual confirmada nos três sistemas.
- r8 — 2026-10-13: SQLite confirmado pelo usuário como armazenamento local do MVP.
- r9 — 2026-10-13: retry de comando interrompido condicionado a confirmação manual; consolidação automática de excedentes sequenciais; histórico retido por 90 dias.
- r10 — 2026-10-13: limite de 100 ocorrências por schedule confirmado; Vue 3 + TypeScript + Vite recomendado; instruções de compilação e instalação manual nos três sistemas confirmadas.
- r11 — 2026-10-13: Vue 3 + Vite, consolidação acima do limite de 100 e instruções multiplataforma confirmadas.
- r12 — 2026-10-13: reconciliadas descrições de stack, retenção, limite e semântica de retry interrompido após confirmações do usuário.
- r13 — 2026-10-13: metadados de quantidade/intervalo para consolidação, retry padrão 1/5/15 minutos e regra de horário de verão confirmados.
- r14 — 2026-10-13: jornadas r10 reconciliadas; contexto do comando e API deve expor os metadados da consolidação.
- r15 — 2026-10-12: confirmados variáveis de ambiente/API para metadados, controle de origem web, suporte a versões com manutenção do fornecedor e BDD em CI nos três sistemas; PRD/jornadas reconciliados.
- r16 — 2026-10-12: reconciliada definição com PRD r9 e jornadas r11; corrigidas lacunas de cobertura e status de stack escolhida.
- r17 — 2026-10-12: referências atualizadas para jornadas r12.
- r18 — 2026-10-12: parâmetros JSON por variável de ambiente e chave opcional de idempotência confirmados; nomes de variáveis e semântica de retenção/conflicto definidos como próximos detalhes.
- r19 — 2026-10-12: reconciliadas referências e cenários BDD com PRD r10 e jornadas r13.
- r20 — 2026-10-12: confirmados nome da variável de parâmetros, nomes/formatos dos metadados e semântica/retensão da chave idempotente.
- r21 — 2026-10-12: referências atualizadas para PRD r11/jornadas r14; esclarecido que são três retries além da execução original; preenchidos nomes ambientais de metadados.
- r22 — 2026-10-12: reconciliadas definições de parâmetros, idempotência e retry com PRD r11 e jornadas r14.
- r23 — 2026-10-12: recomendações iniciais de runtime Rust, representação segura de comando e convenções HTTP/JSON registradas como propostas.
- r24 — 2026-10-12: Axum/Tokio/SQLx, program + args e contrato REST/JSON confirmados; header e esquema de metadados consolidados definidos.
- r25 — 2026-10-12: definição marcada ready após cobertura de escopo, arquitetura/contratos, recuperação, segurança, verificação e operações; lacunas remanescentes delimitadas como decisões não bloqueadoras para planejamento/implementação.
- r26 — 2026-10-12: referência atualizada para jornadas r16.
- r27 — 2026-10-12: jornadas r17 revisadas para remover pendências já resolvidas e atualizar sua fonte.
- r28 — 2026-10-12: parâmetros até 64 KiB e qualquer JSON válido; política de recuperação aplicada a todo atraso; concorrência sem limite global; saída capturada com truncamento; OpenAPI 3.1 completo; metas de pontualidade, acessibilidade e disponibilidade registradas.
- r29 — 2026-10-12: reconciliação com PRD r13 e jornadas r19, incluindo BDD para validação JSON, atrasos, saídas, acessibilidade e SLOs locais.
- r30 — 2026-10-12: referência e cobertura atualizadas para jornadas r20, com critérios BDD de pontualidade, recuperação, OpenAPI e concorrência.
- r31 — 2026-10-13: DEC-001 resolvida como direção do MVP: daemon único Rust coordena API, UI, SQLite, scheduler e executor; não requer validação antes do scaffold.
- r32 — 2026-10-13: arquitetura, segurança, build e instalação reconciliados com CLI local; referências atualizadas para PRD r14/jornadas r21; SQLx reconhecido como escolha fechada.
- r33 — 2026-10-13: fonte atualizada para PRD r15 e estado do SQLx esclarecido como decisão fechada; DEC-036 registra CLI de criação via API local.
- r34 — 2026-10-13: jornadas r22 sincronizadas com PRD r15 e definição revisada para manter fontes rastreáveis após a auditoria.
- r35 — 2026-10-13: cobertura de escopo técnico inclui JRN-013 e explicita a CLI de criação em REQ-001.
- r36 — 2026-10-13: DEC-036 e arquitetura ampliadas para criação e gerenciamento CLI; jornadas de manutenção e controle de execuções incluídas na cobertura técnica.
- r37 — 2026-10-13: fonte atualizada para jornadas r24 após formalização das três jornadas de CLI.
- r38 — 2026-10-13: seção de decisões e restrições atualizada para citar jornadas r24.
- r39 — 2026-10-13: jornadas r25 incorporadas como fonte após ampliar cenários BDD de cancelamento e criação CLI.

## `planning/implementation-plan.md`

- r59 — 2026-10-13: registra TASK-016–018 concluídas em Linux e a cobertura BDD 100/100; gates de release permanecem.

- r58 — 2026-10-13: remove a premissa de intervalo local após confirmação do usuário e registra valores regulares dos presets.

- r57 — 2026-10-13: amplia o MVP para REQ-001–REQ-011/JRN-001–JRN-016 e inclui TASK-017/018.

- r56 — 2026-10-13: aponta à jornada r31 e à definição técnica r58 após detalhamento da validação web.

- r55 — 2026-10-13: sincroniza fontes e mapeia TASK-016 no pacote API/web após detalhar validação dos campos.

- r54 — 2026-10-13: amplia o MVP para REQ-007 e inclui TASK-016 para teste transitório de comandos.

- r53 — 2026-10-13: consolida TASK-004–TASK-010 e TASK-013–TASK-015 como implementadas/validadas em Linux, preservando gates de release.

- r52 — 2026-10-13: registra a medição dedicada Linux de pontualidade em host ocioso; janela mensal continua aberta.

- r51 — 2026-10-13: registra nova rodada Linux completa e restart supervisionado com schedule recuperado.

- r50 — 2026-10-13: registra `Linger=yes` e o resultado parcial da janela de disponibilidade (93 probes, sem declarar meta mensal).

- r49 — 2026-10-13: registra segunda medição Linux e distingue validação do medidor de evidência em host ocioso.

- r48 — 2026-10-13: inclui medição de pontualidade na matriz CI tripla; medição dedicada em host ocioso permanece gate de release.

- r47 — 2026-10-13: liga 001-14 ao medidor Linux e separa cobertura executável de evidência ociosa e execução na matriz CI.

- r46 — 2026-10-13: registra verificação Linux de restart supervisionado, coletor mensal ativo e revisão dos guias; gates de release preservados.

- r45 — 2026-10-13: registra 88/89 BDDs automatizados e mantém medição ociosa e gates operacionais explícitos.

- r44 — 2026-10-13: registra 87/89 BDDs automatizados, mantém dois gates operacionais pendentes e sincroniza estado após ampliar a suíte.

- r43 — 2026-10-13: adiciona persistência e recuperação de política após restart ao BDD; 67/89 cenários cobertos.

- r42 — 2026-10-13: registra BDD de criação inválida pela CLI; cobertura sobe para 66/89.

- r41 — 2026-10-13: cobertura BDD sobe a 65/89 após ligar teste de programa/argumentos ao cenário 002-13.

- r40 — 2026-10-13: inicia coletor mensal de disponibilidade Linux e unit systemd independente; observação mensal segue pendente.

- r39 — 2026-10-13: método de disponibilidade mensal documentado; observação continua pendente.

- r38 — 2026-10-13: cobertura web de consulta/histórico validada; 64/89 BDDs cobertos, 25 pendentes; regra de validação Linux mantida.

- r37 — 2026-10-13: adiciona cenário web de aplicação da política após atraso; atualiza rastreabilidade para 62/89.

- r36 — 2026-10-13: inclui Playwright na CI multiplataforma; 59/89 BDDs cobertos, 30 pendentes.

- r35 — 2026-10-13: sincroniza definição técnica r55 e registra 46/89 BDDs (43 pendentes).

- r34 — 2026-10-13: registra 16 testes BDD e cobertura 38/89 após cenário CLI com daemon indisponível.

- r33 — 2026-10-13: alinha próxima ação com TASK-011/TASK-012 e confirma TASK-006 Linux validada.

- r32 — 2026-10-13: atualiza BDD para 37/89 (52 pendentes) após cobrir políticas e ciclo de vida de schedules pela API.

- r31 — 2026-10-13: registra validação do unit systemd final Linux com persistência após restart.

- r30 — 2026-10-13: registra 14 testes BDD e cobertura de 32/89 cenários; 57 seguem pendentes.

- r29 — 2026-10-13: registra suíte de 13 testes API/CLI com cobertura BDD 26/89.

- r26 — 2026-10-13: fontes sincronizadas com definição técnica r53.

- r25 — 2026-10-13: fontes sincronizadas com definição técnica r52 e testes CLI de loopback.

- r24 — 2026-10-13: acrescenta evidência de SIGTERM Linux e mantém instalação/ativação do serviço como pendente.

- r23 — 2026-10-13: status TASK-006/012 registra systemd, LaunchAgent, tarefa Windows e revisão nativa pendente.

- r22 — 2026-10-13: status de serviço e documentação Linux avançados; gates de BDD e outros SOs preservados.

- r21 — 2026-10-13: estado atualizado: CLI/web/API implementadas em Linux, CI tripla configurada; BDD, serviço e guias seguem pendentes.

- r20 — 2026-10-13: TASK-004 registra decisão e cobertura Linux da retomada pós-pausa; gates de release preservados.

- r19 — 2026-10-13: checkpoint TASK-004–TASK-006 atualizado com recorrência, executor e scheduler Linux implementados; gates de release preservados.

- r18 — 2026-10-13: TASK-004 em andamento com implementação Linux inicial; validação e decisão de retomada após pausa pendentes.

- r17 — 2026-10-13: fontes sincronizadas com definição técnica r44/DEC-038; TASK-001–TASK-003 concluídas com validação Linux, preservando gates multiplataforma de release.
- r16 — 2026-10-13: fontes sincronizadas com definição técnica r43; cron de cinco campos e detalhes escolhidos no contrato TASK-002 registrados no plano.
- r15 — 2026-10-13: registrada exceção temporária de validação Linux durante implementação; gates multiplataforma de release mantidos; TASK-002 liberada.
- r14 — 2026-10-13: estado de execução iniciado e recomendação atualizada para evidência multiplataforma de TASK-001.
- r13 — 2026-10-13: fonte técnica atualizada para r41 após registrar a estrutura concreta e os comandos do scaffold TASK-001.

- r1 — 2026-10-12: plano inicial do MVP com cobertura de REQ/JRN, tarefas dependentes, critérios observáveis e extensões futuras explicitamente não comprometidas.
- r2 — 2026-10-13: registrada como premissa a revisar em TASK-001 a direção arquitetural então proposta em DEC-001.
- r3 — 2026-10-13: arquitetura DEC-001 selecionada para o MVP e removida validação prévia do DoR de TASK-001; plano sem decisões pendentes para iniciar implementação.
- r4 — 2026-10-13: auditoria final conciliou CLI de criação em release criteria, Fase 1, tarefa dedicada, BDD JRN-013 e cobertura REQ/JRN; fontes e revisões sincronizadas.
- r5 — 2026-10-13: fonte técnica atualizada para r35 após conferência final de rastreabilidade do CLI.
- r6 — 2026-10-13: incluído gerenciamento de schedules e execuções pela CLI, com jornadas BDD, tarefas dedicadas, critérios de release e cobertura de escopo.
- r7 — 2026-10-13: fontes sincronizadas com jornada r24 e definição técnica r36 após detalhar paridade de operações CLI.
- r8 — 2026-10-13: fonte técnica sincronizada com r37 após atualizar sua referência para jornada r24.
- r9 — 2026-10-13: fonte técnica sincronizada com r38 após fechar referências internas às jornadas.
- r10 — 2026-10-13: fontes atualizadas para jornadas r25 e definição técnica r39; BDD confirma cancelamento de schedule pela CLI.
- r11 — 2026-10-13: documentação de instalação/manual inclui exemplos de uso da CLI de gerenciamento nos três sistemas.
- r12 — 2026-10-13: tarefas movidas para pacotes focados; cada tarefa recebeu contrato de execução autônoma, cenários BDD rastreáveis, regra de validação e condição de escalonamento; fontes reconciliadas.

## `product/requirements.md`

- r5 — 2026-10-13: confirma recorrências alinhadas ao relógio local e limita valores N dos presets para manter intervalos regulares.

- r4 — 2026-10-13: adiciona REQ-008–REQ-011 para presets, cockpit, agenda global e filtros/histórico escalável.

- r3 — 2026-10-13: esclarece rótulos, exemplo e heurística de detecção de linha completa no formulário.

- r2 — 2026-10-13: adiciona REQ-007 para testar comando durante a criação web.

- r1 — 2026-10-13: catálogo separado da visão do PRD, preservando REQ-001–REQ-006 e seus critérios.

## `product/journeys/web.md`

- r9 — 2026-10-13: atualiza a fonte técnica para r61 após validar presets, agenda, cockpit e filtros em Linux.

- r8 — 2026-10-13: sincroniza fontes após confirmar intervalos alinhados ao relógio local.

- r7 — 2026-10-13: detalha presets de recorrência na criação web e JRN-016 com sete cenários de cockpit, agenda e histórico.

- r6 — 2026-10-13: alinha fontes com a definição técnica r58.

- r5 — 2026-10-13: adiciona critérios e BDD de separação de executável/argumentos.

- r4 — 2026-10-13: adiciona a etapa e cenário BDD para testar comando antes de salvar.

- r3 — 2026-10-13: fonte técnica reconciliada para r48 após DEC-039.

- r2 — 2026-10-13: jornada web cobre aplicação da política salva ao retomar schedule pausado.

- r1 — 2026-10-13: jornadas web separadas do índice geral; cenários BDD preservados com IDs estáveis.

## `product/journeys/api-index.md`

- r1 — 2026-10-13: índice de jornadas da API separado do índice geral; referências por canal adicionadas.

## `product/journeys/api-schedules.md`

- r3 — 2026-10-13: fontes atualizadas para jornadas r28 e definição técnica r48.

- r2 — 2026-10-13: BDD de API define recuperação de horários vencidos ao retomar conforme política salva.

- r1 — 2026-10-13: jornadas API de schedules e parâmetros separadas; cenários BDD preservados com IDs estáveis.

## `product/journeys/api-executions.md`

- r2 — 2026-10-13: fonte técnica reconciliada para r48.

- r1 — 2026-10-13: jornadas API de execuções e recuperação separadas; cenários BDD preservados com IDs estáveis.

## `product/journeys/cli.md`

- r3 — 2026-10-13: fonte técnica atualizada para r48 e estado de pausa retomada sincronizado.

- r2 — 2026-10-13: fonte técnica atualizada para r47 após decisão de retomada pós-pausa.

- r1 — 2026-10-13: jornadas CLI separadas; cenários BDD preservados com IDs estáveis.

## `engineering/decisions/runtime-and-delivery.md`

- r3 — 2026-10-13: fontes sincronizadas com jornadas r28.

- r2 — 2026-10-13: DEC-038 registra diretório de dados SQLite por usuário, override, migrações SQLx, WAL, timestamps e expurgo.
- r1 — 2026-10-13: decisões DEC-001 e relacionadas separadas por tema da definição técnica r40, preservando IDs.

## `engineering/decisions/scheduling-and-execution.md`

- r4 — 2026-10-13: fontes sincronizadas com jornadas r28 e DEC-039.

- r3 — 2026-10-13: DEC-039 confirma a aplicação da política salva aos horários vencidos durante pausa.

- r2 — 2026-10-13: DEC-037 escolhe cron padrão de cinco campos para recorrências e mantém a semântica DST de DEC-018.
- r1 — 2026-10-13: decisões de agendamento e execução separadas por tema da definição técnica r40, preservando IDs.

## `engineering/api/openapi.yaml`

- r5 / v1.1.0 — 2026-10-13: acrescenta consultas agregadas, agenda, histórico global paginado e filtros de estado.

- r4 / v1.0.2 — 2026-10-13: documenta POST `/commands/test` e esquemas de entrada/resultado.

- r3 / v1.0.1 — 2026-10-13: corrige `missedOccurrences` para corresponder à serialização da API e ao consumo da UI.

- r2 — 2026-10-13: endpoint de retomada documenta aplicação da política de horários vencidos.

- v1.0.0 — 2026-10-13: contrato REST/JSON cobre schedules, parâmetros, ocorrências, controles, erros, paginação, idempotência, metadados de consolidação e exemplos.

## `engineering/decisions/interfaces-and-quality.md`

- r8 — 2026-10-13: DEC-041 passa a decisão confirmada de intervalos alinhados ao relógio local e valores N suportados.

- r7 — 2026-10-13: registra DEC-041 para presets compatíveis com cron e DEC-042 para consultas globais paginadas do cockpit.

- r6 — 2026-10-13: alinha fontes com o índice de jornadas r31.

- r5 — 2026-10-13: DEC-040 incorpora explicação visual e validação heurística sem rejeitar caminhos com espaços.

- r4 — 2026-10-13: registra DEC-040 para testar comandos sem persistir schedule ou ocorrência.

- r3 — 2026-10-13: DEC-027 registra o nome público `missedOccurrences` conforme o contrato executável.

- r2 — 2026-10-13: fontes sincronizadas com jornadas r28.

- r1 — 2026-10-13: decisões de interfaces e qualidade separadas por tema da definição técnica r40, preservando IDs.

## `planning/tasks/foundation.md`

- r38 — 2026-10-13: registra TASK-004/005 validadas em Linux e atualiza os gates de release.

- r37 — 2026-10-13: documenta parada da tarefa agendada Windows antes de atualizar arquivos em uso.

- r36 — 2026-10-13: sincroniza referências com o plano r49.

- r35 — 2026-10-13: sincroniza referências com o plano r48.

- r34 — 2026-10-13: sincroniza referências com o plano r47.

- r33 — 2026-10-13: registra teste de recuperação após falha supervisionada no Linux e sincroniza plano r46.

- r32 — 2026-10-13: sincroniza referências com o plano r45.

- r31 — 2026-10-13: sincroniza referências com o plano r44.

- r30 — 2026-10-13: sincroniza referências com o plano r43.

- r29 — 2026-10-13: sincroniza pacote com o plano r42.

- r28 — 2026-10-13: sincroniza referências com o plano r41.

- r27 — 2026-10-13: sincroniza referências com o plano r40.

- r26 — 2026-10-13: sincroniza referências com plano r37.

- r25 — 2026-10-13: alinha o campo do OpenAPI `missedOccurrences` em TASK-002.

- r24 — 2026-10-13: sincroniza fontes com plano r36 e definição técnica r55.

- r22 — 2026-10-13: sincroniza fonte do pacote com plano r34.

- r21 — 2026-10-13: sincroniza o pacote com plano r33.

- r20 — 2026-10-13: sincroniza referência ao plano r32 após validar unit systemd no Linux.

- r19 — 2026-10-13: fecha evidência Linux da TASK-006 com instalação real do unit e recuperação após restart.

- r18 — 2026-10-13: registra build release e evidência Linux de health/SIGTERM; unit não foi ativado.

- r18 — 2026-10-13: fontes sincronizadas com definição técnica r53 e plano r26.

- r17 — 2026-10-13: evidência final Linux atualizada para 22 testes e fontes r52/r25.

- r16 — 2026-10-13: registra smoke de SIGTERM e status verificável do unit systemd.

- r16 — 2026-10-13: fontes sincronizadas com definição técnica r50 e plano r23.

- r15 — 2026-10-13: registra configuração de serviços/supervisores nos três SOs e pendência de evidência nativa.

- r14 — 2026-10-13: registra shutdown por sinal Unix e unit systemd; validação Linux em andamento.

- r14 — 2026-10-13: registra shutdown por sinal Unix e unit systemd; validação Linux em andamento.

- r13 — 2026-10-13: fontes reconciliadas com definição técnica r48, jornadas r28 e plano r21.

- r12 — 2026-10-13: TASK-004 registra DEC-039 e teste de retomada; validação Linux continua autorizada.

- r11 — 2026-10-13: registra implementação Linux das TASK-004–TASK-006 e evidência inicial de 19 testes.

- r10 — 2026-10-13: registra implementação inicial do cron, DST, recuperação limitada e cursor persistente na TASK-004; validação Linux e semântica da retomada após pausa pendentes.

- r9 — 2026-10-13: TASK-003 concluída com migrações, persistência, idempotência e retenção testadas em Linux; TASK-004 ativa.
- r7 — 2026-10-13: TASK-002 concluída com contrato OpenAPI validado, referência local verificada em Linux e decisão de cron propagada.
- r6 — 2026-10-13: TASK-001 concluída para desenvolvimento com validação Linux conforme orientação do usuário; gates nativos Windows/macOS movidos à verificação de release sem removê-los.
- r5 — 2026-10-13: tentativas de linkagem cruzada e inventário de ambientes disponíveis registrados; TASK-001 segue bloqueada por hosts Windows/macOS.
- r4 — 2026-10-13: resultados de `cargo check` cruzado e smoke Linux registrados; TASK-001 segue parcial por falta de linkagem/execução nativa Windows/macOS.
- r3 — 2026-10-13: comandos de validação concretos registrados no repositório; fontes reconciliadas para definição técnica r41/plano r14.
- r2 — 2026-10-13: TASK-001 marcada parcial com scaffold, verificações locais aprovadas e bloqueio de evidência multiplataforma documentado.

- r1 — 2026-10-13: tarefas de base e runtime separadas do plano r12, com contrato para execução autônoma e cenários BDD vinculados.

## `planning/tasks/api-and-web.md`

- r35 — 2026-10-13: registra TASK-016–018 implementadas, 15 testes Playwright e verificação Linux.

- r34 — 2026-10-13: sincroniza fontes e enumera os intervalos confirmados para TASK-018.

- r33 — 2026-10-13: especifica TASK-017 para consultas globais e TASK-018 para presets e cockpit web.

- r32 — 2026-10-13: alinha fontes da TASK-016 com jornada r31, definição técnica r58 e plano r56.

- r31 — 2026-10-13: TASK-016 passa a exigir rótulos claros e bloqueio de padrões comuns de linha completa.

- r29 — 2026-10-13: adiciona TASK-016 para endpoint e botão de teste no formulário de criação.

- r28 — 2026-10-13: registra validação Linux de API, interface web, BDDs e acessibilidade básica.

- r27 — 2026-10-13: sincroniza referências com o plano r49.

- r26 — 2026-10-13: sincroniza referências com o plano r48.

- r25 — 2026-10-13: sincroniza referências com o plano r47.

- r24 — 2026-10-13: sincroniza a referência do plano r46.

- r23 — 2026-10-13: sincroniza referências com o plano r45.

- r22 — 2026-10-13: sincroniza referências com o plano r44 e a cobertura BDD atual.

- r21 — 2026-10-13: registra BDDs de criação inválida e recuperação após restart.

- r20 — 2026-10-13: rastreia BDD-JRN-013-02 de criação inválida pela CLI e sincroniza plano r42.

- r19 — 2026-10-13: rastreia BDD-JRN-002-13 à prova existente de programa e argumentos separados.

- r18 — 2026-10-13: atualiza referência para o plano r40.

- r17 — 2026-10-13: registra nove testes web e cobertura da consulta de histórico/saída (BDD-JRN-004-01/02).

- r16 — 2026-10-13: documenta cenário Playwright para recuperação consolidada e política salva.

- r15 — 2026-10-13: alinha critério de resposta API ao campo camelCase `missedOccurrences`.

- r14 — 2026-10-13: documenta sete testes Playwright e suas jornadas web vinculadas.

- r13 — 2026-10-13: corrige o nome do metadado consolidado para `missedOccurrences` e sincroniza as fontes.

- r12 — 2026-10-13: fontes sincronizadas com definição técnica r53 e plano r26.

- r11 — 2026-10-13: fontes sincronizadas com definição técnica r50 e plano r23.

- r10 — 2026-10-13: fontes sincronizadas com definição técnica r49 e plano r22.

- r9 — 2026-10-13: fontes sincronizadas com definição técnica r48, jornadas r28 e plano r21.

- r8 — 2026-10-13: status TASK-007–TASK-010 atualizado para implementação integrada em Linux, com BDD pendente.

- r7 — 2026-10-13: fontes sincronizadas com definição técnica r46 e plano r19.

- r5 — 2026-10-13: fontes sincronizadas com persistência DEC-038, definição técnica r44 e plano r17.
- r6 — 2026-10-13: fontes sincronizadas com definição técnica r45 e plano r18.
- r4 — 2026-10-13: fontes atualizadas para OpenAPI, DEC-037, definição técnica r43 e plano r16 após conclusão de TASK-002.
- r3 — 2026-10-13: fontes sincronizadas para plano r15/definição técnica r42 com regra de validação Linux temporária.
- r2 — 2026-10-13: comandos concretos do projeto vinculados como base de validação; fontes sincronizadas para definição técnica r41/plano r14.
- r1 — 2026-10-13: tarefas de API e web separadas do plano r12, com contrato para execução autônoma e cenários BDD vinculados.

## `planning/tasks/verification-docs-and-cli.md`

- r49 — 2026-10-13: reconcilia fontes e evidências Linux com TASK-016–018 concluídas, 100/100 BDDs e 15 testes Playwright.

- r48 — 2026-10-13: sincroniza fontes e cobertura de TASK-011 com requisitos r5 e plano r58.

- r47 — 2026-10-13: sincroniza TASK-011 e rastreabilidade BDD/fontes após adicionar JRN-016 e TASK-017/018.

- r46 — 2026-10-13: registra BDDs JRN-013–JRN-015 validados no Linux.

- r45 — 2026-10-13: atualiza evidência de pontualidade ociosa e sincroniza referências à medição concluída.

- r44 — 2026-10-13: comprova P95/P99 em execução Linux após janela de CPU ociosa acima de 95%.

- r43 — 2026-10-13: registra nova rodada de validações Linux e repetição bem-sucedida do teste systemd.

- r42 — 2026-10-13: registra linger confirmado e relatório parcial corretamente incompleto, sem alegar meta mensal.

- r41 — 2026-10-13: registra atualização do instalador Windows para parar o daemon antes de substituir arquivos.

- r40 — 2026-10-13: registra segunda medição Linux e mantém pendente comprovação em host ocioso.

- r39 — 2026-10-13: adiciona medição de pontualidade à matriz CI tripla; mantém requisito de evidência ociosa.

- r38 — 2026-10-13: registra 89/89 verificações vinculadas e medidor de pontualidade Linux fora da CI.

- r37 — 2026-10-13: registra supervisor Linux reproduzido, observação mensal iniciada e guias revisados.

- r36 — 2026-10-13: atualiza suíte para 46 testes Python e cobertura 88/89; um BDD e evidência operacional seguem pendentes.

- r35 — 2026-10-13: registra 45 testes Python, 87/89 BDDs e os gates de supervisor/pontualidade.

- r34 — 2026-10-13: registra 23 testes de jornada e cobertura 67/89 após recuperação em restart.

- r33 — 2026-10-13: registra 22 testes de jornada e 66/89 BDDs após cobrir criação CLI inválida.

- r32 — 2026-10-13: sincroniza cobertura para 65/89 BDDs (24 pendentes) e plano r41.

- r31 — 2026-10-13: registra coletor Linux, unit systemd e doze testes de relatório.

- r30 — 2026-10-13: documenta o método mensal de disponibilidade e mantém observação como gate pendente.

- r29 — 2026-10-13: atualiza evidência Linux para nove testes Playwright e cobertura 64/89.

- r28 — 2026-10-13: registra 8 testes Playwright, 62/89 cenários vinculados e 27 pendentes.

- r27 — 2026-10-13: sincroniza fontes com o plano r36 após incluir suíte Playwright.

- r26 — 2026-10-13: adiciona execução Playwright à CI e registra cobertura web/API/CLI 59/89.

- r25 — 2026-10-13: registra 21 testes BDD, cobertura 46/89 e confirmação de retry interrompido pela CLI.

- r23 — 2026-10-13: registra cenário CLI offline, 16 testes e cobertura 38/89; sincroniza plano r34.

- r22 — 2026-10-13: sincroniza o pacote com plano r33; mantém 15 testes BDD e cobertura 37/89.

- r21 — 2026-10-13: inclui BDD de política/ciclo de vida de schedules; suíte passa com 15 testes e cobertura 37/89.

- r20 — 2026-10-13: sincroniza o pacote com plano r30; registra 14 testes, cobertura 32/89 e validação Linux.

- r19 — 2026-10-13: sincroniza a fonte do pacote com o plano r29 e cobertura BDD 26/89.

- r18 — 2026-10-13: CI executa 13 testes de jornada e informa rastreabilidade de 26/89 BDDs; pontualidade Linux medida.

- r15 — 2026-10-13: TASK-012 registra guias multiplataforma e evidências de build/smoke Linux.

- r14 — 2026-10-13: fontes sincronizadas com definição técnica r53 e plano r26.

- r13 — 2026-10-13: CLI limitada a loopback, com timeout; testes de URL local/remota adicionados.

- r12 — 2026-10-13: guias iniciais de instalação agora cobrem Linux, macOS e Windows, com revisão nativa pendente.

- r11 — 2026-10-13: inclui guia CLI e serviço Linux; fontes sincronizadas com definição técnica r49/plano r22.

- r10 — 2026-10-13: registra guia CLI, smoke tests Linux e CI de build/testes nos três SOs.

- r9 — 2026-10-13: registra CLI funcional e smoke tests Linux de criação, manutenção e execuções.

- r8 — 2026-10-13: fontes sincronizadas com definição técnica r47/plano r20; CLI iniciada.

- r7 — 2026-10-13: fontes sincronizadas com definição técnica r46 e plano r19.

- r5 — 2026-10-13: fontes sincronizadas com persistência DEC-038, definição técnica r44 e plano r17.
- r6 — 2026-10-13: fontes sincronizadas com definição técnica r45 e plano r18.
- r4 — 2026-10-13: fontes atualizadas para OpenAPI, DEC-037, definição técnica r43 e plano r16.
- r3 — 2026-10-13: fontes sincronizadas para plano r15/definição técnica r42; gates CI Windows/macOS/Linux mantidos como critérios de release.
- r2 — 2026-10-13: comandos concretos do projeto vinculados como base de validação; fontes sincronizadas para definição técnica r41/plano r14.
- r1 — 2026-10-13: tarefas de verificação, documentação e CLI separadas do plano r12, com contrato para execução autônoma e cenários BDD vinculados.

## `product/workflow-state.md`

- r70 — 2026-10-13: registra nova auditoria open source, correções de dependências/transação/pacote, avisos de terceiros, CI de segurança e metadados do repositório previsto.

- r69 — 2026-10-13: registra a validação dos 14 comentários de revisão, sete falhas remanescentes corrigidas, novas reproduções e evidências Linux; preserva gates nativos e CI remota pendentes.

- r68 — 2026-10-13: sincroniza pacote de verificação/docs/CLI r49 após a validação Linux final.

- r67 — 2026-10-13: registra implementação Linux de TASK-016–018, 100/100 BDDs e validações executadas.

- r66 — 2026-10-13: registra confirmação da semântica de intervalo local e sincroniza artefatos até plano r58.

- r65 — 2026-10-13: registra a revisão de produto, plano r57, nove BDDs novos e próxima etapa TASK-017/018.

- r64 — 2026-10-13: reconcilia revisões e checkpoint de TASK-016 com BDD-JRN-001-16 e build UI.

- r63 — 2026-10-13: registra labels/help e detecção heurística no formulário, build UI aprovado e dois BDDs pendentes.

- r62 — 2026-10-13: registra implementação parcial de TASK-016, builds/lint aprovados e verificação funcional pendente.

- r61 — 2026-10-13: registra correção do 403 no proxy de desenvolvimento, isolamento da porta do daemon e verificação do ciclo make up/down.

- r60 — 2026-10-13: atualiza checkpoint para plano r53 e estado das tarefas Linux validadas.

- r59 — 2026-10-13: sincroniza status Linux das tarefas e revisões dos pacotes/plano.

- r58 — 2026-10-13: sincroniza estado e documentação de contribuição com a medição ociosa aprovada.

- r57 — 2026-10-13: fecha evidência Linux de pontualidade e atualiza a próxima ação para observação mensal.

- r56 — 2026-10-13: registra build/testes Linux atuais e segunda comprovação de restart com recuperação.

- r55 — 2026-10-13: registra confirmação de linger, endpoints saudáveis e relatório parcial da observação mensal.

- r54 — 2026-10-13: registra correção do fluxo de upgrade do instalador Windows e sincroniza revisões das tarefas.

- r53 — 2026-10-13: esclarece que matriz CI está configurada mas ainda sem execução remota comprovada.

- r52 — 2026-10-13: ajusta próxima ação para medição ociosa e acompanhamento da janela mensal.

- r51 — 2026-10-13: registra duas medições Linux, CI de pontualidade na matriz e plano r49.

- r50 — 2026-10-13: sincroniza plano r48 e registra medidor de pontualidade na matriz CI.

- r49 — 2026-10-13: registra 89/89 verificações vinculadas e separa medição manual dos gates de release.

- r48 — 2026-10-13: registra observação mensal ativa, verificador do supervisor Linux, guias revisados e plano r46.

- r47 — 2026-10-13: registra 88/89 BDDs, 46 testes Python, plano r45 e a medição ociosa pendente.

- r46 — 2026-10-13: sincroniza plano r44 e registra 87/89 BDDs, dois gates operacionais e evidências atualizadas.

- r45 — 2026-10-13: registra cobertura 67/89 e sincroniza plano r43/pacotes atualizados.

- r44 — 2026-10-13: sincroniza plano/pacotes r42 e cobertura BDD 66/89.

- r43 — 2026-10-13: corrige próxima ação para 24 cenários BDD pendentes.

- r42 — 2026-10-13: sincroniza pacote base r28 e registra coletor com detecção de reinício.

- r41 — 2026-10-13: cobertura 65/89 e plano/pacotes sincronizados após rastrear BDD-002-13.

- r40 — 2026-10-13: registra coletor Linux, unit de usuário systemd e suíte Python com 33 testes.

- r39 — 2026-10-13: registra o método de disponibilidade mensal e sincroniza plano r39/tarefa r30.

- r38 — 2026-10-13: sincroniza plano r38, pacotes r17/r29 e cobertura 64/89 após novos BDDs web.

- r37 — 2026-10-13: sincroniza contagem 62/89, 8 testes de browser e pacotes de tarefas atualizados.

- r36 — 2026-10-13: sincroniza pacotes de tarefas r25/r15/r27 e próxima revisão do plano r36.

- r35 — 2026-10-13: registra 7 testes Playwright, 59/89 cenários e DEC-027/OpenAPI v1.0.1.

- r33 — 2026-10-13: sincroniza contagem 46/89, tarefas r23/r13/r25 e definição técnica r55.

- r31 — 2026-10-13: atualiza cobertura para 38/89 e sincroniza o plano r34.

- r30 — 2026-10-13: sincroniza a próxima ação e os artefatos com plano r33 e pacotes atualizados.

- r29 — 2026-10-13: registra 15 testes BDD, 37/89 cenários, validação Linux do unit systemd e próximas ações atualizadas.

- r28 — 2026-10-13: registra validação do systemd final Linux e sincroniza definição técnica r54/plano r31.

- r27 — 2026-10-13: registra cobertura BDD 32/89, 14 testes Linux e plano r30.

- r26 — 2026-10-13: corrige a cobertura BDD para 26/89, sincroniza revisões dos artefatos e registra validação local somente em Linux durante a implementação.

- r25 — 2026-10-13: registra suíte de 13 testes BDD API/CLI (26/89 IDs), evidência systemd e medição Linux; 63 BDDs seguem pendentes.

- r21 — 2026-10-13: validação Linux atualizada com build release, 22 testes e referências para plano r26.

- r19 — 2026-10-13: fontes sincronizadas com definição técnica r53 e plano r26.

- r18 — 2026-10-13: registra 22 testes Linux e validação CLI limitada a loopback.

- r17 — 2026-10-13: registra smoke Linux de encerramento por sinal e pendência de ativação real do unit.

- r16 — 2026-10-13: inclui templates e instruções de supervisor nos três SOs; validação de release permanece pendente.

- r15 — 2026-10-13: estado atualizado com systemd Linux, guia operacional e pendências de release.

- r14 — 2026-10-13: registra unit systemd e guia Linux; evidência operacional, BDD e demais SOs permanecem pendentes.

- r13 — 2026-10-13: estado e referências sincronizados; próximo foco em BDD, serviço e documentação operacional.

- r12 — 2026-10-13: atualiza revisões e contagem BDD; registra CI de build/testes nos três SOs e avanço TASK-013–TASK-015.

- r11 — 2026-10-13: registra decisão DEC-039, UI/API integradas e avanço do cliente CLI.

- r10 — 2026-10-13: checkpoint registra implementação Linux inicial de recorrência, executor e daemon; pausa, API/UI/CLI/BDD e gates seguem no workflow.

- r9 — 2026-10-13: checkpoint Linux da TASK-004 com 11 testes e gravação transacional das ocorrências recuperadas; integração daemon e retomada após pausa pendentes.

- r8 — 2026-10-13: TASK-003 concluída com evidência Linux; TASK-004 selecionada e fontes atualizadas para DEC-038/definição técnica r44/plano r17.
- r7 — 2026-10-13: TASK-002 registrada como concluída com evidência Linux; TASK-003 selecionada e revisões sincronizadas com OpenAPI v1.0.0/DEC-037.
- r6 — 2026-10-13: usuário autorizou validação Linux durante implementação; TASK-001 concluída e TASK-002 registrada como próxima atividade.
- r5 — 2026-10-13: checkpoint atualizado com tentativas de linkagem cruzada, inventário de ambientes e bloqueio explícito para execução nativa Windows/macOS.
- r4 — 2026-10-13: checkpoint atualizado com checagem cruzada Windows/macOS, smoke Linux e pendência de linkagem/execução nativa.
- r3 — 2026-10-13: passo `/mvp` iniciado; TASK-001 ativa, evidências e bloqueio Windows/macOS registrados.

- r1 — 2026-10-13: checkpoint documental registrou o conjunto de artefatos, revisões e próximo passo.
- r2 — 2026-10-13: histórico separado do estado; próximo passo mantido como seção própria.
