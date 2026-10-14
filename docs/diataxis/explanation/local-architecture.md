# Arquitetura local

O Command Scheduler é executado como um daemon na máquina do usuário. Ele mantém o scheduler, a API HTTP local, a persistência SQLite e o executor de processos. A UI Vue é servida pelo daemon em produção; durante desenvolvimento, o Vite serve a UI e encaminha os endpoints locais ao daemon.

A CLI é outro cliente da API. Ela não possui uma camada separada de regras ou persistência: criação, validação e consulta passam pelo mesmo contrato usado pela interface. Aplicações locais também podem consumir essa API.

SQLite preserva schedules e ocorrências entre reinícios. Quando o daemon reinicia, horários vencidos são avaliados conforme `missedPolicy` do schedule. Uma execução que estava ativa quando o daemon parou fica interrompida e requer confirmação para nova tentativa. O daemon adquire um lock exclusivo associado ao banco antes dessa recuperação. A política `sequence` mantém a ordem e executa uma ocorrência do schedule por vez. Cancelar um schedule impede retries automáticos pendentes e futuros.

O limite local permite funcionamento sem serviço hospedado, mas significa que schedules dependem de a máquina e o daemon estarem ativos. A API aceita somente loopback e não inclui autenticação no MVP. Chamadas de páginas web são restringidas à origem local da interface.

O contrato de comunicação está descrito em [OpenAPI 3.1](../../engineering/api/openapi.yaml); os detalhes de configuração ficam na [referência do daemon](../reference/configuration.md).

Para seguir a implementação pelo repositório, consulte [Funcionamento técnico e mapa do repositório](technical-architecture-and-repository.md), que detalha os módulos Rust, os fluxos de inicialização e execução, a camada Vue e as migrações.
