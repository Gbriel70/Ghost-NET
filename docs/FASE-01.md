# GHOST//NET — Fase 1: primeira comunicação entre dois nós

**Estado:** ativa. **Subetapa corrente:** 1.1 — compreender a conexão.  
**Origem:** `docs/ROADMAP.md` e `docs/architecture.md`; o `AGENTS.md` é obrigatório para agentes.

## Objetivo e fronteira

Construir **somente** um laboratório Rust que execute dois peers em `127.0.0.1`, conecte-os por rust-libp2p com TCP + Noise + Yamux e troque uma mensagem **de aplicação** `PING`/`PONG` via request-response. Registrar PeerIds, conexão, resposta e falhas. Não desenvolver transferência de blobs, persistência, DHT, Wasm ou interface gráfica.

O `ping::Behaviour` do libp2p (subetapa 1.2) e um `PING/PONG` de aplicação via `request_response` (subetapa 1.3) são protocolos **diferentes**. Não confundir evidência de liveness com protocolo de aplicação.

## 1.1 — Estudo guiado (antes do primeiro diff)

Estudar nesta ordem, apenas até conseguir explicar a responsabilidade de cada elemento:

1. **TCP e fluxo de bytes:** `connect`, `listen`, `accept`, EOF, ausência de fronteiras de mensagem no stream. *Pergunta:* por que uma única leitura não corresponde necessariamente a uma mensagem?
2. **Tokio e `async/await`:** runtime, futures, `await`, `select!`, temporizadores; task não equivale a thread. *Pergunta:* o que acontece se bloquear o event loop?
3. **PeerId e Multiaddr:** diferença entre identidade criptográfica do peer e endereço de transporte. *Pergunta:* dois nós podem escutar no mesmo IP com portas distintas?
4. **Noise e Yamux:** autenticação e criptografia da conexão vs multiplexação de streams. *Pergunta:* essas camadas executam nossa lógica de negócios?
5. **Swarm e NetworkBehaviour:** o Swarm recebe eventos e conduz transporte/comportamentos. *Pergunta:* quem faz a aplicação reagir a conexões e solicitações?

**Materiais primários:**
- Tutorial oficial libp2p ping: https://github.com/libp2p/rust-libp2p/blob/master/libp2p/src/tutorials/ping.rs
- Documentação libp2p: https://docs.rs/libp2p/latest/libp2p/
- Request-response: https://docs.rs/libp2p/latest/libp2p/request_response/
- Tutorial Tokio spawning: https://tokio.rs/tokio/tutorial/spawning
- Tutorial Tokio select: https://tokio.rs/tokio/tutorial/select

**Evidência de 1.1:** explicar oralmente/em notas: TCP → Noise → Yamux → comportamento libp2p → evento recebido pela aplicação. Não é necessário decorar métodos.

## 1.2 — Primeiro experimento (ainda sem protocolo próprio)

**Tarefa de código:** gerar só `experiments/p2p-spike`, com um único binário que serve para A ou B, utilizando `libp2p::ping::Behaviour`. Exibir endereço de escuta, PeerId e eventos de conexão/ping; permitir ligar B a A por Multiaddr explícito. Não criar o workspace completo do produto.

**Acompanhamento:** separar alterações para iniciar Swarm, adicionar `listen_on`, tratar eventos e então testar dois processos. Entender cada diff antes do próximo.

**Testes necessários:**
- Positivo: A escuta; B conecta; log mostra PeerIds e ping bem-sucedido.
- Negativo: B tenta se conectar com endereço onde nada escuta; erro visível, sem pânico.
- Encerramento: matar A e observar evento de desconexão em B.

**Observação:** IDs gerados de novo por execução bastam neste experimento. Identidade persistente pertence à Fase 4.

## 1.3 — Nosso primeiro protocolo de aplicação

Trocar o exemplo ping por, ou acrescentar, `request_response` para uma solicitação `PING` e resposta `PONG`. Usar versão de protocolo identificável e codec com limites explícitos; escolher serialização simples sem congelar o formato futuro do GhostNet. Registrar o ID da solicitação para correlacionar a resposta.

**Testes:** resposta válida, pedido desconhecido/malformado rejeitado, peer indisponível, timeout. Sem blobs, hashes, DHT ou Wasmtime.

## 1.4 — Revisão e gate de passagem

Registrar comandos executados e saídas reais. A Fase 1 só estará concluída após:
- Dois nós estabelecem conexão autenticada local; PeerIds observáveis.
- O protocolo de aplicação retorna PONG para PING.
- Conexão inválida, timeout e desconexão durante requisição resultam em erro identificável.
- Não ocorrem panic nem espera infinita nos casos testados.
- O autor consegue descrever eventos `Swarm`, transporte, Noise, Yamux e request-response sem precisar memorizar cada função.
- ADR sobre a escolha inicial do transporte e request-response: registrar candidato e evidências, aceitar somente com aprovação humana.

## Comandos iniciais (somente quando começar 1.2)

Na raiz do repositório de experimentos:

```bash
mkdir -p experiments
cargo new --bin experiments/p2p-spike
cd experiments/p2p-spike
rustc --version
cargo --version
```

Antes de instalar crates, conferir compatibilidade de Rust e versões no ambiente. Consulta em 22/10/2026: docs.rs lista `libp2p 0.57.1` (publicada em 15/10/2026); isso **não** é validação de compilação no Debian do projeto. Fixar no `Cargo.lock` a versão efetivamente testada. Não presumir que um exemplo antigo é compatível sem conferir documentação da versão selecionada.

## Instrução de handoff ao agente para 1.2 (usar quando 1.1 terminar)

> Leia `AGENTS.md`, `docs/architecture.md`, `docs/ROADMAP.md` e este arquivo. Estamos na Fase 1, subetapa 1.2. Antes de escrever código, explique em linguagem acessível `PeerId`, `Multiaddr`, TCP, Noise, Yamux, `Swarm` e `ping::Behaviour`. Proponha o menor experimento com dois processos e seus testes. Só depois implemente **uma pequena mudança por vez**. Não adicione request-response ainda: isso pertence à 1.3. Liste os diffs e os comandos/testes realmente executados. Não marque testes como aprovados sem evidência. Não altere decisões de arquitetura sem propor ADR.
