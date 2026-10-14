# Segurança

## Escopo e versões

O Command Scheduler está em desenvolvimento (0.1.0). As correções são feitas
na versão atual do código; versões anteriores não recebem backports. A
validação local comprovada é Linux. A CI inclui Windows e macOS; resultados
nativos precisam passar antes de anunciar uma release para esses sistemas.

## Modelo de confiança

- O daemon executa comandos com as permissões da conta que o iniciou.
- A API não tem autenticação e deve permanecer em loopback. Processos locais
  com acesso à porta podem criar schedules, executar comandos e ler parâmetros
  e saídas. A proteção de Host/Origin impede origens web externas; não separa
  usuários ou processos locais.
- Não exponha a API por túnel, proxy público ou interface de rede externa.
  Execute o daemon com uma conta sem privilégios administrativos.
- Comandos e descendentes não são isolados por sandbox. Cancelamento e timeout
  não desfazem efeitos já produzidos; no Unix, processos que deliberadamente
  escapem do grupo de processos não são controlados por esse grupo.
- Parâmetros e saídas podem conter segredos. O banco SQLite, backups e logs
  precisam das mesmas restrições de acesso que esses dados.

## Relatar vulnerabilidades

Use [Report a vulnerability](https://github.com/alsvinicius/commandscheduler/security/advisories/new)
quando o mantenedor tiver habilitado relatórios privados no GitHub. Se esse canal ainda
não estiver disponível, abra uma issue solicitando um canal privado, sem
incluir segredos, detalhes de exploração ou dados de terceiros. O canal
privado deve ser habilitado antes de anunciar a publicação.

Inclua versão/commit, sistema operacional, impacto e reprodução com dados
fictícios. Não execute reproduções contra instalações de outras pessoas.

## Verificações de dependências

`cargo audit` e `pnpm audit` consultam bases de vulnerabilidades conhecidas.
Um resultado limpo registra a situação dessas bases naquele instante;
ele não prova ausência de vulnerabilidades desconhecidas.
