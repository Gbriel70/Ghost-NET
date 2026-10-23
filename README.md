# GHOST//NET — experimento da subetapa 1.2

Um binário, executado em processos independentes, com identidade Ed25519 nova a
cada execução, TCP + Noise + Yamux e `libp2p::ping::Behaviour`. Não implementa o
PING/PONG de aplicação da 1.3. Chaves privadas não são persistidas nem registradas.

## Executar dois nós

A partir da raiz do repositório:

```bash
cargo build --locked --manifest-path experiments/p2p-spike/Cargo.toml
```

Terminal A:

```bash
./experiments/p2p-spike/target/debug/p2p-spike /ip4/127.0.0.1/tcp/4101
```

Copie o valor `address=` da linha `event="listening"` de A. No terminal B,
substitua `PEER_ID_DE_A` pelo identificador real exibido nessa linha:

```bash
./experiments/p2p-spike/target/debug/p2p-spike /ip4/127.0.0.1/tcp/4102 /ip4/127.0.0.1/tcp/4101/p2p/PEER_ID_DE_A
```

Ambos devem mostrar `connected` com o PeerId remoto e `ping_ok`. Ctrl+C encerra
normalmente o processo. Ao encerrar A, B deve registrar `disconnected` e continuar
ativo. Para evitar escolher portas manualmente, use porta `0` na escuta: o SO
escolhe uma porta disponível e o programa registra o endereço efetivo.

Uma Multiaddr de destino sem `/p2p/...` também é aceita. Noise autentica a
identidade apresentada, mas sem um PeerId esperado não há comparação com uma
identidade previamente conhecida. Com `/p2p/...`, `Swarm::dial` extrai a identidade
esperada; divergência produz `WrongPeerId`, sem evento `connected` no discador.
Autenticação de transporte não significa confiança no conteúdo do peer.

## Implementação e limites do laboratório

- `local_address` aceita somente IPv4 loopback + TCP e um PeerId final opcional no
  destino. Endereços inválidos e falhas de configuração local encerram com erro.
- Tokio executa o laço assíncrono em uma thread. `Swarm` conduz o transporte e os
  comportamentos; `select!` acompanha eventos e Ctrl+C.
- Noise e Yamux são fornecidos pelo libp2p, sem criptografia ou framing próprios.
- Negociação/conexão: timeout de 5 s. Ping: intervalo de 1 s e timeout de 3 s.
- Conexões: até 4 recebidas pendentes, 1 saída pendente, 4 estabelecidas no total
  e 1 estabelecida por peer. Conexão ociosa: 60 s.
- Falhas de discagem, negociação e ping são registradas; o processo segue
  processando eventos e pode aceitar outras conexões. Não há retry automático.
- Logs estruturados de texto incluem identidade, endereço, conexão e resultado;
  ANSI está desabilitado. Não registram payloads ou segredos.

Estes valores são limites do spike, não um contrato definitivo do GHOST//NET.
Não há persistência, descoberta, DHT, NAT traversal, arquivos ou Wasm. O programa
não é uma política de admissão: outros processos locais podem tentar conectar.
Limites de conexões não demonstram proteção completa contra DoS ou teto de RAM.

## Dependências e APIs verificadas

Ambiente observado: `rustc 1.98.1 (48a229cea 2026-09-01)` e
`cargo 1.98.1 (797e8a9bc 2026-08-05)`.

O registro crates.io confirmou `libp2p 0.57.1`, Rust mínimo 1.88.0. O lockfile
registra libp2p 0.57.1, Tokio 1.54.1, futures 0.3.34, tracing 0.1.44 e
tracing-subscriber 0.3.23. Use `--locked` para reproduzir essa resolução.

Libp2p fornece a stack de rede e limites; Tokio fornece o runtime/sinais;
futures permite consultar o stream de eventos; tracing e tracing-subscriber
fornecem emissão e formatação dos logs. Foram preferidos componentes mantidos dos
respectivos ecossistemas a implementar transporte, criptografia e logging. A CLI
usa a biblioteca padrão, sem adicionar um parser. O lockfile não é auditoria de
segurança; dependências e código transitivo continuam sendo superfície de ataque.

Referências consultadas: [tutorial oficial de ping](https://github.com/libp2p/rust-libp2p/blob/master/libp2p/src/tutorials/ping.rs)
e fontes das versões baixadas em `~/.cargo/registry/src/` (builder, dial_opts e
connection-limits). A versão compilada e seus testes prevalecem sobre exemplos
antigos do tutorial.

## Verificação reproduzível

Na raiz do repositório:

```bash
cargo fmt --manifest-path experiments/p2p-spike/Cargo.toml -- --check
cargo test --locked --offline --manifest-path experiments/p2p-spike/Cargo.toml
cargo clippy --locked --offline --manifest-path experiments/p2p-spike/Cargo.toml --all-targets -- -D warnings
python3 experiments/p2p-spike/tests/processes.py
```

O modo offline exige as dependências já baixadas. O teste Python usa somente a
biblioteca padrão, inicia binários reais e possui prazos finitos. Requer permissão
para sockets de loopback. Gera logs por processo em `validation/`, sobrescritos a
cada execução. Os PeerIds nesses logs são públicos e efêmeros.

Resultados observados nesta implementação:

- Build e teste Rust: sucesso; 1 teste de validação de endereços passou.
- Formatação e Clippy com warnings tratados como erros: sucesso.
- Suíte de processos: conexão autenticada com identidades distintas e ping;
  desligamento abrupto de A com B ativo; PeerId esperado incorreto rejeitado;
  peer ausente; TCP aceito sem negociação resulta em timeout; nova conexão após
  falha; quinta conexão excede o limite de quatro.
- A tentativa inicial no sandbox falhou porque sockets foram bloqueados
  (`Operation not permitted`). Os testes de rede foram executados novamente com
  autorização fora do sandbox.

Não foram testados interoperabilidade com outras versões, redes remotas, fuzzing,
exaustão de memória ou o timeout de ping de um peer autenticado que deixa de
responder. O teste de timeout cobre a negociação inicial, não pedidos de aplicação.

A subetapa fica disponível para revisão humana; a Fase 1 permanece incompleta.
O próximo passo é reproduzir os comandos e revisar o comportamento antes de
autorizar 1.3. A adoção definitiva da stack continua dependente de ADR e aprovação.
