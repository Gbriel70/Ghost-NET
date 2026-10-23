# GHOST//NET — Arquitetura inicial

**Estado:** proposta v0.1 para revisão; nenhuma funcionalidade comprovada.  
**Referência:** `GHOSTNET_BLUEPRINT.md` (21/10/2026).  
**Objetivo deste documento:** registrar arquitetura, princípios firmes da V0, opções ainda candidatas e fronteiras que os agentes devem respeitar.

## 1. Visão do sistema

Uma pessoa publica um pacote portátil contendo um artefato WebAssembly e metadados verificáveis. Nós P2P distribuem cópias; o consumidor valida a integridade e procedência; um executor local aplica política, consentimento e limites de recursos antes de usar o componente.

```text
Usuário / CLI
     |
     v
Orquestrador / política local ---------------> Executor Wasmtime (somente com autorização)
     |                                           ^
     +----> Manifesto, assinatura e confiança    |
     |                                           |
     +----> Store de conteúdo validado ---------+
                    ^
                    |
              Módulo P2P <-----> peers conhecidos (V0)
```

O diagrama representa uma **separação lógica**, não processos ou crates já implementados.

## 2. Fluxo mínimo de confiança e distribuição

1. Publicador gera artefato Wasm e manifesto que se refere de maneira inequívoca aos bytes do artefato.
2. Publicador assina os metadados conforme formato que ainda será definido.
3. Nó A oferece o pacote; B busca e valida antes de armazenar como válido.
4. B faz pin; A é desligado; C solicita o pacote a B.
5. C valida digest, assinatura e se a chave é aceita pela política local.
6. Somente ação de execução explícita e consentida leva o artefato ao host Wasmtime.
7. O host aplica limites; operações e falhas são observáveis.

**Invariantes:** um digest não substitui uma chave; uma assinatura válida não torna código seguro; um anúncio de disponibilidade não prova que o peer tenha o artefato; um download não implica execução; A desligado só não prejudica a recuperação caso B continue disponível e tenha o conteúdo íntegro.

## 3. Fronteiras de módulos — propostas

| Módulo lógico | Responsabilidade | O que não pode fazer |
|---|---|---|
| Protocolo | Mensagens, tamanhos, validação, compatibilidade | Abrir sockets por conta própria |
| Rede | Sessões P2P, requests, timeout e backpressure | Aceitar dados remotos como confiáveis |
| Store | Staging, blobs por digest, pins, índice, recuperação | Executar componentes |
| Identidade e confiança | Chaves, manifesto assinado, política local | Conceder capacidades de host automaticamente |
| Runtime | WIT/Wasmtime, limites, capabilities e resultados | Conectar-se à rede P2P ou decidir confiança sozinho |
| Core/orquestração | Publicar, obter, verificar e solicitar execução | Reimplementar transportes e criptografia |
| CLI | Acionar operações, mostrar evidências e confirmar permissões | Duplicar lógica crítica de segurança |

**Formato do código:** começar com dois pequenos binários experimentais separados e extrair crates apenas quando houver responsabilidades reais. A árvore de crates do blueprint não é ordem para criar pastas vazias.

## 4. Decisões de escopo adotadas para a V0

- Uma máquina Debian, com nós locais separados (não depende de Docker, nuvem ou hardware extra).
- Peers conhecidos por endereços explícitos; DHT e NAT traversal amplo ficam para depois.
- Primeiro duas provas de conceito independentes: comunicação libp2p e execução Wasmtime.
- CLI e logs em vez de GUI na etapa inicial.
- Download/armazenamento separados da autorização/execução.
- Privilégio mínimo; a IA não decide ou concede permissões de execução.
- Um cenário reproduzível de 3–4 nós é o gate funcional da V0.

## 5. Decisões candidatas — NÃO aprovadas definitivamente

| ADR futura | Opção atualmente preferida | Validação necessária |
|---|---|---|
| ADR-001 Arquitetura | Monólito modular / workspace Rust | Fronteiras de código e testes |
| ADR-002 P2P | rust-libp2p, inicialmente TCP + Noise + Yamux | Spike autenticado de dois nós |
| ADR-003 Wasm | Wasmtime + Component Model/WIT | Componente válido, capacidades negadas, limites testados |
| ADR-004 Transferência | Request-response com mensagens limitadas e chunks | Testes de framing, memória e desconexão |
| ADR-005 Confiança | Ed25519 + política local de chaves | Assinatura correta, errada e chave não confiável |
| ADR-006 Conteúdo | SHA-256 dos bytes do Wasm (`sha256:hex`) | Vetores de hash e corrupção |
| ADR-007 Persistência | Blob store no filesystem + SQLite | Atomicidade, restart e concorrência |
| ADR-009 Providers | Peers conhecidos e indícios de disponibilidade | Recuperação após queda de A |

**Princípios já definidos, não opções de implementação:** autorização explícita, negação por padrão, separação de rede e execução, IA fora da autoridade de segurança.

## 6. Questões abertas, a decidir após experimentos

1. API e versões interoperáveis de `rust-libp2p`, Tokio e Wasmtime no Debian.
2. Transporte inicial concreto (TCP é candidato; QUIC não é exigido).
3. Codec, limites de quadros, tamanho de chunks, streaming e negociação de versão.
4. Representação canônica e esquema do manifesto assinado; relação entre digest do blob, identidade do pacote e chave do publicador.
5. Contrato WIT definitivo e fallback WASI caso o Component Model não funcione no ambiente escolhido.
6. Limites reais de CPU/tempo/memória/saída e isolamento extra por processo ou SO.
7. Estrutura final de crates, SQLite, GC/quotas e persistência de anúncios de provedores.
8. Estratégia de rotação/revogação de chaves para estágios posteriores.

## 7. Ameaças mínimas e limites de segurança

- **Peer remoto malicioso:** envia conteúdo truncado, mensagens inválidas, dados sem fim ou anúncio falso. Rejeitar entradas fora de limites; autenticar transporte e verificar bytes.
- **Artefato adulterado:** sempre comparar hash final esperado antes de marcar objeto válido; assinatura deve cobrir manifesto e referência ao artefato.
- **Publicador malicioso:** uma assinatura não substitui confiança; capacidades e aprovação continuam necessárias.
- **Guest Wasm hostil:** negar imports/capacidades por padrão, limitar recursos e tratar traps como falhas; limites reais requerem teste. Wasmtime não equivale a VM infalível.
- **Armazenamento incompleto:** staged writes, commit atômico e reconciliação após restart; não expor blob parcial como válido.
- **CLI/UI e futura IA:** output e descrições podem conter conteúdo hostil; a IA nunca determina autorização.

## 8. Como a arquitetura evolui

Uma decisão arquitetural muda somente quando houver: problema concreto, alternativas, teste/argumento, registro ADR e validação humana. Os agentes não devem converter automaticamente os candidatos acima em certezas nem afirmar propriedades de segurança sem evidência.
