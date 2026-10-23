# GHOST//NET — Roadmap sequencial de desenvolvimento e estudos

**Status:** execução inicial. **Fase 0 aprovada em 22/10/2026. Fase ativa: 1** (etapa 1.1: estudar a conexão). Nenhuma implementação técnica concluída ainda.

## Como usar este roadmap

- **Uma fase ativa por vez.** Não começar tarefas da próxima antes de demonstrar o gate da atual.
- **Estudar apenas os conceitos que permitem realizar e revisar a tarefa da fase.** Não exigir domínio completo de uma biblioteca.
- **Primeiro hipótese/contrato; depois código pequeno; depois teste real.** Testes precisam incluir um cenário normal e um de falha (e limite quando aplicável).
- **A IA implementa junto, não decide sozinha.** O desenvolvedor deve compreender o comportamento, revisar diffs e aprovar alterações de arquitetura.
- Sem estimativas obrigatórias de dias ou semanas; registrar resultado e dificuldades reais.

---

## Fase 0 — Acordo arquitetural e agentes [APROVADA]

**Entregas:** `AGENTS.md`, `docs/architecture.md`, `docs/ROADMAP.md`. Sem código funcional e sem dependências Rust nesta fase.

**Estudar agora:**
1. **ADRs:** por que registrar uma decisão, alternativas, consequências e evidências.
2. **Separação de responsabilidades:** rede, store, confiança, orquestração e runtime.
3. **Threat model básico:** quem é confiável, que dados são externos e o que não podemos permitir.
4. **Invariantes:** regras que continuam verdadeiras mesmo quando um peer falha (ex.: download não executa).

**Fazer:** revisar os três documentos; marcar como hipótese o que depende de teste; aceitar ou modificar princípios, escopo e sequência. Só então iniciar o repositório Rust, se ainda não existir.

**Gate:** o responsável compreende, revisa e aprova o contrato de trabalho e a arquitetura provisória. **Criar os arquivos não aprova automaticamente o gate.**

**Não fazer agora:** código P2P, Wasmtime, UI, IA, blockchain, árvore inteira de crates.

---

## Fase 1 — Primeira comunicação entre dois nós [ATIVA]

**Entregável:** dois processos Rust locais com identidade própria estabelecem conexão autenticada usando libp2p e trocam uma mensagem simples via request-response; logs identificam remetente, conexão e resultado.

**Estudar antes/durante:**
1. **TCP, sockets, streams:** entrega de bytes e desconexão; não existe delimitação automática de mensagens no TCP.
2. **`async/await` e Tokio:** runtime, tasks, `select!`, timeout; diferença entre tarefa assíncrona e thread.
3. **PeerId e chaves de transporte:** identidade de peer, sem confundir com assinatura de pacote.
4. **Noise e Yamux:** autenticação/criptografia do transporte e multiplexação de streams.
5. **`Swarm` e `request_response`:** eventos, estados e troca tipada de mensagens.

**Passos:**
1. Criar um exemplo mínimo Rust específico para o spike (não o produto completo).
2. Iniciar A na porta local definida e B conectando por endereço conhecido.
3. Confirmar `peer_connected`; trocar uma mensagem `ping/pong` de aplicação, ou equivalente.
4. Inserir timeout e logs de conexão/desconexão.
5. Testar peer ausente e desligamento do peer durante pedido.

**Gate:** dois nós trocam a mensagem, conexões são autenticadas, a identidade dos peers é observável e erros/desconexões não deixam o programa travado.

**Não estudar ainda:** DHT/Kademlia, NAT traversal, QUIC e protocolo completo de transferência.

---

## Fase 2 — Transferir o primeiro blob pequeno

**Entregável:** B solicita a A um arquivo pequeno de teste, recebe bytes limitados, calcula SHA-256 e os compara com um digest esperado obtido previamente de fonte local confiável.

**Estudar:**
1. **Framing e serialização:** como distinguir mensagens e impor limites.
2. **Bytes vs texto:** comprimentos, UTF-8 e buffers binários.
3. **SHA-256:** hash como integridade, não autoria ou segurança do programa.
4. **Leituras parciais, timeout e backpressure:** transferência não deve bloquear indefinidamente.

**Passos:** acrescentar protocolo de pedido de blob simples; limitar resposta; verificar hash; testar arquivo truncado/corrompido e desconexão.

**Gate:** B recebe bytes exatos; corrupção/truncamento são rejeitados; tempo e tamanho são limitados; nenhuma execução ocorre.

**Não fazer:** chunks complexos, armazenamento definitivo, IPFS/CIDs ou download de arquivos enormes.

---

## Fase 3 — Executar o primeiro WebAssembly localmente (spike independente)

**Entregável:** host Rust executa um componente Wasm local com interface tipada e registra resultado/erro sem depender da rede.

**Estudar:**
1. **Wasm vs runtime:** o que representa o binário e quem o executa.
2. **WASI, Component Model e WIT:** interfaces e capacidades disponíveis ao guest.
3. **Wasmtime host/guest:** carregar, instanciar, chamar e tratar falhas.
4. **Limites de recursos:** memória, fuel/epoch, timeout e limites de saída.
5. **Capabilities:** permissões explicitamente expostas pelo host.

**Passos:** componente `uppercase` ou `word-count`; host invoca função; testar import não concedido; loop infinito; limites de entrada/saída; host continua respondendo.

**Gate:** saída esperada, erro de importação negada e interrupção por recurso observados em testes; limitações documentadas.

**Não fazer:** integrar P2P e Wasm ainda; executar artefatos desconhecidos da internet.

---

## Fase 4 — Identidade, manifesto e armazenamento local

**Entregável:** `publish` local (ainda sem P2P real nesta fase) produz manifesto e assinatura; store preserva objetos por digest; `inspect/verify/pin` consultam e validam.

**Estudar:**
1. **Chaves públicas e Ed25519:** assinar, verificar e distinguir autenticação de confiança.
2. **Canonicalização:** quais bytes exatamente são assinados; por que JSON arbitrário pode ser ambíguo.
3. **CAS (content-addressed storage):** imutabilidade e deduplicação por digest.
4. **Staging, `rename` e `fsync`:** consistência e comportamento após crash.
5. **SQLite e transações:** índice/pins, dados órfãos e recuperação.

**Passos:** fechar ADRs do manifesto/assinatura; implementar validações localmente; escrever staging→verify→commit; testar persistência, corrupção, assinatura errada e chave não confiável.

**Gate:** conteúdo válido persiste após reinício; corrupção e assinatura inválida são rejeitadas; `pin` funciona; nenhuma chave desconhecida ganha execução automática.

---

## Fase 5 — Transferência P2P real com persistência

**Entregável:** A oferece pacote real; B baixa com tamanho, memória e concorrência limitados, grava em staging, valida e comita no store.

**Estudar:**
1. **Chunking e streaming:** blocos e verificação final.
2. **Estado da transferência:** início, progresso, finalização, interrupção e retry.
3. **Backpressure e concorrência:** limites globais/por peer, bounded queues.
4. **Integridade em falhas:** arquivos temporários, commit atômico e retomada/recomeço.

**Passos:** definir framing e protocolo versionado em ADR; implementar fluxo de transferência incremental; testar truncamento, tamanho excessivo, peer lento, concorrência e `SIGKILL`.

**Gate:** pacote íntegro chega ao B, sem explosão de RAM; dados incompletos não entram no store válido; erros são identificáveis.

---

## Fase 6 — Replicação quando publicador desaparece

**Entregável:** A publica, B obtém/pina, A é encerrado, C recupera de B; cenário negativo falha quando nenhuma cópia está acessível.

**Estudar:**
1. **Disponibilidade vs persistência:** pin local não garante uma cópia online.
2. **Provider discovery inicial:** peers conhecidos e anúncios tratados como indícios, não provas.
3. **Replicação vs cópia:** quem possui bytes verificáveis e quem apenas anuncia.
4. **Falhas e particionamento:** timeouts, retries e indisponibilidade real.

**Passos:** implementar anúncios/provedores simples; script de três nós; desligar A; transferir a partir de B; testar todas as réplicas offline.

**Gate:** repetível sem ajuda de A; digest confere; falha honesta e observável sem provedores.

---

## Fase 7 — Execução local autorizada do pacote recebido

**Entregável:** C só executa o Wasm obtido via P2P depois de confirmar manifesto, integridade, assinatura, política e consentimento.

**Estudar:**
1. **Fronteira de confiança:** peer, publicador, artefato e host são papéis diferentes.
2. **Trust policy:** chave confiável, desconhecida e bloqueada.
3. **Capability-based security:** requested ≠ supported ≠ granted.
4. **Runtime e SO:** limites do Wasmtime e isolamento complementar Linux.
5. **Máquinas de estados e UX:** `fetch` não altera estado para `running` implicitamente.

**Passos:** integrar Wasmtime ao core; mostrar origem, digest e permissões antes da execução; impor limites; registrar decisão e resultado.

**Gate:** execução correta autorizada; download sem execução; chave bloqueada e import sem permissão não executam; loop infinito não derruba o nó.

---

## Fase 8 — Hardening e demonstração V0

**Entregável:** script reprodutível A→B (pin)→A offline→C (verify+run), com testes adversariais, logs estruturados, documentação e CI mínimos.

**Estudar:**
1. **Threat modeling:** superfícies de ataque de rede/store/runtime/CLI.
2. **Observabilidade com `tracing`:** eventos e correlação de solicitações.
3. **Diagnóstico Linux:** `strace`, `tcpdump`, logs, sinais, memória e processos.
4. **Testes de falha e fuzzing (conforme viável):** limites, parser, concorrência e corrupção.

**Passos:** automatizar happy path e falhas; testar adulteração, assinatura, loop, saída excessiva e reinício; revisar README e documentação de limites.

**Gate:** outra pessoa consegue reproduzir a V0 em Debian; resultados de testes são guardados; o projeto não promete segurança/anonimato/disponibilidade absoluta.

---

## Depois da V0 — somente se os gates anteriores passarem

V1: catálogo e busca textual; depois IA assistiva sem autoridade de execução. Evoluções independentes possíveis: DHT e peers remotos; revogação/reputação; composição de ferramentas; interface web; portabilidade browser; investigação de computação remota voluntária. Nenhuma destas faz parte do primeiro ciclo de implementação.

## Registro de andamento

| Fase | Estado inicial | Evidência exigida |
|---|---|---|
| 0 | Aprovada em 22/10/2026 | documentos aceitos pelo responsável |
| 1 | Ativa, 1.1 em preparação; sem código validado | dois nós + falhas |
| 2 | Não iniciada | transferência pequena + hash |
| 3 | Não iniciada | host Wasmtime + isolamento testado |
| 4 | Não iniciada | manifesto, assinatura, store |
| 5 | Não iniciada | transferência persistente segura |
| 6 | Não iniciada | A offline, C baixa de B |
| 7 | Não iniciada | política + consentimento + execução |
| 8 | Não iniciada | demonstração V0 reproduzível |
