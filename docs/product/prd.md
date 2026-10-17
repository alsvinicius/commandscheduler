# Command Scheduler — PRD

- Status: ready
- Revisão: 19
- Histórico de revisões: [registro central](../revision-history.md) — seção `product/prd.md`
- Atualizado: 2026-10-13
- Fontes e revisões: pedido inicial e respostas de descoberta/definição do usuário (2026-10-12 a 2026-10-13), pedidos de melhorias na interface e escolhas confirmadas nesta revisão, além do escopo em `AGENTS.md`
- Idioma: português, conforme as entrevistas e os artefatos de produto deste projeto. Novas revisões seguem o idioma usado pelo usuário no workflow ativo.
- Artefatos relacionados: [estado do workflow](workflow-state.md), [índice](../README.md)

## Visão e necessidade de negócio

Criar uma plataforma local para desenvolvedores individuais gerenciarem a execução agendada de comandos na própria máquina. O produto deve oferecer CLI para criar e gerenciar schedules, interface web e API para gerenciar e controlar execuções, aceitar agendamentos e atualizações de parâmetros de aplicações externas e substituir cron e scripts de agendamento espalhados por um ponto central de controle.

O valor pretendido é centralizar schedules e controle de execução local, mantendo compatibilidade com fluxos de outras aplicações que preparam execuções por API. A dor principal confirmada é substituir cron e scripts espalhados.

## Usuários e stakeholders

- **Desenvolvedor individual:** cria, mantém e controla agendamentos pela CLI e interface web.
- **Aplicação integradora:** cria agendamentos e fornece parâmetros para a execução.
- **Máquina local:** ambiente do desenvolvedor onde a plataforma executa comandos.

## Entregáveis esperados e medidas de sucesso

Entregável pretendido: uma plataforma local com CLI, interface web e API que coordene comandos executados na máquina do desenvolvedor. CLI, web e API permitem gerenciar schedules e execuções; aplicações locais também integram pela API. Inclui schedules únicos e recorrentes por calendário, parâmetros de aplicações externas e execução opcional imediata a cada atualização.

Medida de sucesso: quantidade de agendamentos migrados de cron/scripts para a plataforma. Meta numérica e período de medição não definidos.

## Escopo, exclusões e MVP

### Incluído no escopo pretendido

- Gerenciar schedules para comandos de terminal.
- Criar, consultar, editar, pausar/retomar e cancelar schedules pela CLI local; schedules criados por qualquer canal continuam consultáveis e gerenciáveis pela CLI, web e API.
- Controlar execuções pela CLI, incluindo executar agora, consultar histórico/resultado/saídas, cancelar execução ativa, repetir falhas e confirmar retry após interrupção.
- Executar comandos na máquina local do desenvolvedor.
- Permitir gerenciar, manter e controlar agendamentos por interface web e API.
- Permitir que aplicações externas criem agendamentos.
- Permitir testar programa e argumentos durante a criação web, exibindo resultado sem salvar schedule ou ocorrência.
- Oferecer presets de recorrência em formulário web e manter a expressão cron como opção avançada.
- Apresentar um cockpit com estado do daemon, próximo schedule, contagens por estado e execuções bem-sucedidas/falhas no período.
- Permitir consultar schedules e execuções em agenda global por dia, semana e mês, com filtros por estado e histórico paginado.
- Permitir parametrização ao criar o agendamento e atualização dos parâmetros antes de cada execução.
- Aceitar qualquer valor JSON válido como parâmetros, limitado a 64 KiB por schedule na criação e em cada atualização.
- Permitir configurar, por agendamento, se uma atualização de parâmetros executa imediatamente e se esse gatilho convive com o horário.
- Permitir repetir execuções com falha manualmente ou por política automática configurável.
- Preservar schedules e configurações após reinício da máquina e permitir configurar por schedule como tratar horários vencidos: pular as ocorrências, consolidá-las em uma execução ao retornar ou executar todas em sequência.
- Executar como serviço/daemon local iniciado automaticamente, com suporte a Windows, macOS e Linux; fornecer instruções para compilar e instalar manualmente.
- No modo sequencial, limitar a 100 ocorrências vencidas por schedule e consolidar excedentes em uma execução; a execução consolidada usa parâmetros recentes e expõe a quantidade e o intervalo cobertos.
- Manter histórico de execuções por 90 dias.
- Aplicar por padrão até três retries automáticos para falhas normais, nos intervalos de 1, 5 e 15 minutos; a política pode ser configurada por schedule. Execução interrompida por reinício requer confirmação manual antes de repetir.
- Em recorrências locais, pular horários inexistentes na mudança do relógio e executar uma única vez horários repetidos.

### MVP

O núcleo do MVP é permitir que um desenvolvedor individual substitua cron/scripts espalhados por schedules criados e gerenciados pela CLI, web ou API. Deve suportar execuções únicas e recorrência por calendário flexível; na interface web, presets comuns complementam cron avançado. Schedules e configurações persistem após reinício, e cada schedule permite pular ocorrências vencidas, consolidá-las em uma execução ao retornar ou executar todas em sequência. No modo sequencial, até 100 ocorrências por schedule são mantidas individualmente; excedentes são consolidados em uma execução, que usa parâmetros recentes e disponibiliza quantidade e intervalo cobertos. Parâmetros podem ser enviados na criação e atualizados antes do início da execução. Se configurado, cada atualização dispara imediatamente uma execução; esse gatilho pode substituir ou coexistir com o horário por agendamento. Inclui pausar/retomar, cancelar, executar agora e repetir falhas manualmente ou com política automática. CLI, web e API permitem consultar execuções e seus resultados; a interface oferece cockpit, agenda dia/semana/mês, filtros de estado e histórico paginado. Falhas normais têm padrão de até três retries em 1, 5 e 15 minutos, configurável por schedule; execução interrompida exige confirmação manual antes de novo retry. O histórico é mantido por 90 dias.

### Fora do MVP e limites de medição

Execução distribuída, acesso pela rede, autenticação, instalador/atualização automática e notificações ficam fora do MVP. A API local não exige autenticação e aceita conexões somente por localhost; chamadas de navegador ficam limitadas à origem da UI. A métrica de schedules migrados não tem meta ou período definidos. O histórico e as saídas têm retenção de 90 dias.

## Requisitos detalhados

Os requisitos REQ-001–REQ-011 estão no [catálogo de requisitos](requirements.md). Esta visão do produto mantém apenas seu objetivo, escopo e medidas de sucesso.

## Restrições e riscos de negócio

- A plataforma dispara comandos com efeitos na máquina local, usando as permissões do usuário local; não há restrições adicionais de produto no MVP.
- A API não terá autenticação no MVP e ficará acessível somente por localhost.
- Atualizações podem gerar execuções imediatas; execuções concorrentes do mesmo agendamento são permitidas.
- Não foram informados requisitos regulatórios, escala, isolamento entre usuários ou modelo comercial.
- Meta de pontualidade no MVP: em máquina ociosa, P95 de início até 5 segundos após o horário previsto e P99 até 30 segundos; sem meta formal de volume.
- Meta de disponibilidade mensal do daemon: 99,5% durante o período em que a máquina está ligada e apta a executar schedules; períodos desligados ou suspensos ficam fora do denominador. O daemon deve reiniciar automaticamente após falha e recuperar schedules persistidos.
- Interface sem meta formal de conformidade de acessibilidade no MVP; navegação por teclado e rótulos acessíveis são requisitos básicos.

## Decisões confirmadas, premissas e itens encerrados

### Confirmado

- O público inicial é o desenvolvedor individual que quer substituir cron e scripts espalhados.
- A CLI, a interface web e a API local são canais do MVP; aplicações locais integram pela API.
- As regras funcionais detalhadas e seus critérios estão no [catálogo de requisitos](requirements.md); decisões de arquitetura e operação estão na [definição técnica](../engineering/technical-definition.md).

### Métrica sem alvo quantitativo no MVP

- A quantidade de schedules migrados de cron/scripts será acompanhada como medida descritiva; a descoberta não definiu meta numérica nem período-alvo. Impacto: não há base para alegar sucesso contra um alvo quantitativo. Reavaliar uma meta após observar os resultados do MVP. Fonte: respostas de descoberta registradas em 2026-10-12 e 2026-10-13.
