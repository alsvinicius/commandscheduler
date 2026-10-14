# Documentação de uso

Esta seção ajuda usuários e integradores a aprender e operar o Command Scheduler. Ela segue Diátaxis e fica separada dos artefatos de descoberta e planejamento do produto, que continuam em [`../product/`](../product/) e [`../planning/`](../planning/).

O Command Scheduler mantém comandos locais recorrentes ou únicos. O daemon precisa estar ativo para avaliar horários e iniciar processos; schedules e histórico ficam no SQLite local. A interface web e a CLI são clientes da mesma API. Para começar, siga o tutorial; para uma tarefa específica, escolha um guia; consulte referências para opções e formatos.

## Aprender fazendo

- [Criar e executar seu primeiro schedule](tutorials/first-schedule.md): inicia o ambiente local, cria um schedule pela API e acompanha uma execução.
- [Criar e executar um schedule pela CLI](tutorials/first-schedule-cli.md): compila os binários, inicia o daemon e percorre o fluxo com comandos de terminal.

## Resolver uma tarefa

- [Como integrar uma aplicação pela API](how-to/integrate-with-api.md): criar e acompanhar schedules a partir de um cliente local.
- [Como administrar schedules e execuções](how-to/manage-schedules-and-runs.md): editar, pausar, retomar, cancelar, atualizar parâmetros e tratar ocorrências.
- [Como instalar e manter o daemon](how-to/install-and-operate.md): compilar e executar como serviço de usuário; procedimentos por sistema operacional.
- [Como criar um schedule pela interface web](how-to/create-schedule-in-web-ui.md): usar presets, testar comandos e consultar cockpit, agenda e histórico.
- [Como executar um programa com argumentos](how-to/run-program-with-arguments.md): configurar executáveis, argumentos e scripts sem depender do shell interativo.
- [Como iniciar o ambiente de desenvolvimento no Linux](how-to/start-dev-environment-linux.md): subir, observar e encerrar UI e daemon.
- [Como diagnosticar uma execução que falhou](how-to/diagnose-failed-run.md): consultar o resultado e investigar erros de inicialização.
- [Como publicar o código no GitHub](how-to/publish-open-source.md): revisar arquivos, auditorias, licenças e verificações de publicação.

## Consultar fatos

- [Referência da CLI](reference/cli.md): subcomandos, opções e padrões.
- [Configuração do daemon](reference/configuration.md): variáveis de ambiente, endereços e armazenamento.
- [Schedules, horários e tentativas](reference/schedules-and-occurrences.md): campos, estados, políticas de recorrência e origem das ocorrências.
- [Contrato da API](../engineering/api/openapi.yaml): endpoints, esquemas, exemplos e respostas.

## Entender o sistema

- [Como o Command Scheduler executa comandos](explanation/execution-model.md): processo direto, isolamento de argumentos e ambiente.
- [Arquitetura local](explanation/local-architecture.md): relação entre UI, API, daemon, CLI e banco.
- [Funcionamento técnico e mapa do repositório](explanation/technical-architecture-and-repository.md): inicialização, fluxos, persistência, execução e responsabilidade de cada módulo.
- [Recorrência, recuperação e retries](explanation/recurrence-and-retries.md): por que horários vencidos e falhas geram ocorrências diferentes.

## Escopo de validação

Os passos de desenvolvimento e os exemplos desta documentação são validados em Linux durante a implementação atual. Isso descreve o ambiente de validação disponível agora; o suporte de produto a Windows e macOS continua descrito nos artefatos de engenharia e release.
