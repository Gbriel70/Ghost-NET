# AGENTS.md — GHOST//NET

> **Status:** regras aprovadas na Fase 0; fase ativa atual: Fase 1 — comunicação entre dois nós.  
> **Projeto:** GHOST//NET — plataforma experimental *local-first* de distribuição P2P e execução consentida de ferramentas WebAssembly.  
> **Fonte da proposta:** `GHOSTNET_BLUEPRINT.md`, versão 0.1 (21/10/2026).  
> **Importante:** documentação de planejamento não equivale a software implementado. Não invente resultados de testes, funcionalidades existentes, comandos ou versões de dependências.

## 1. Antes de qualquer alteração

1. Leia este arquivo, `docs/architecture.md` e `docs/ROADMAP.md`.
2. Identifique a **fase ativa** no roadmap e seu critério de conclusão (*gate*).
3. Consulte os ADRs existentes e os testes da área afetada, **se já existirem**.
4. Distinga: **princípio obrigatório**, **decisão aceita**, **candidato a decisão**, **questão aberta** e **comportamento comprovado por teste**.
5. Não comece trabalho de fases futuras, não gere o projeto inteiro e não crie estruturas vazias apenas para preencher o desenho do blueprint.
6. Antes de usar APIs ou sugerir comandos dependentes de versão, confira documentação atual das dependências e fixe versões testadas na implementação.

## 2. Propósito e demonstração-alvo da V0

A V0 deve funcionar **em um único computador Debian**, com 3–4 processos Rust separados, sem infraestrutura externa obrigatória:

1. A publica um pacote com artefato Wasm, manifesto e assinatura.
2. B obtém, valida, armazena e faz pin do pacote.
3. A é encerrado de verdade.
4. C encontra a cópia em B e obtém os bytes sem consultar A.
5. C valida digest, assinatura e política de confiança.
6. C executa **localmente**, após autorização explícita, usando Wasmtime com recursos limitados.
7. Testes demonstram rejeição de adulteração, negação de capacidades não concedidas e tratamento de execução excessiva.

**Fora da V0:** DHT global obrigatória, NAT traversal complexo, rede pública, blockchain, token, IA executora autônoma, interface gráfica obrigatória, anonimato e execução remota em computadores de outras pessoas.

## 3. Princípios arquiteturais obrigatórios

- **Separação de responsabilidades:** rede, protocolo, armazenamento, identidade/confiança, políticas e execução não devem se misturar arbitrariamente.
- **Distribuir não é executar:** `fetch`/`pin` nunca iniciam um programa; `run` é uma ação separada e consentida.
- **Integridade ≠ autoria ≠ confiança:** SHA-256 identifica bytes esperados; assinatura vincula metadados a uma chave; política local decide se o publicador é aceito.
- **Negação por padrão:** sem acesso a rede, filesystem, variáveis sensíveis ou outras capacidades para o guest Wasm sem concessão explícita.
- **Entrada externa é hostil:** mensagens P2P, metadados, artefatos, saída do Wasm e futuramente respostas da IA devem ser validados.
- **Limites explícitos:** tamanhos, timeout, memória, CPU/combustível, concorrência, filas e saída devem possuir limites verificáveis adequados ao estágio.
- **Falhas previsíveis:** nunca promover dados parcialmente recebidos ou não verificados para objetos válidos; testar interrupções, corrupção e peer indisponível.
- **IA não é autoridade:** modelos podem recomendar, gerar e revisar código; autorização de execução/permissões deve existir no programa, fora de prompts.
- **Reutilização antes de reinvenção:** não escrever criptografia, protocolo de transporte seguro ou runtime próprio sem necessidade demonstrada.
- **Sem promessas absolutas:** não alegar sandbox infalível, anonimato ou disponibilidade permanente.

## 4. Direção arquitetural: aceita vs. proposta

**Direção de escopo adotada para o protótipo:**

- Desenvolvimento individual em Debian, sem serviços pagos obrigatórios.
- Primeira comunicação em nós locais com peers conhecidos explicitamente.
- Primeiro validar a rede e a execução Wasm **em experimentos independentes**.
- CLI e logs antes de painel gráfico; IA somente após demonstrar a V0.
- Projeto modular, testável, com decisões e ameaças documentadas.

**Candidatos — precisam ser validados por spike e ADR antes de serem tratados como definitivos:**

- Rust + Tokio + `rust-libp2p` (primeiro experimento com TCP/Noise/Yamux/request-response).
- Wasmtime + Component Model/WIT; plano alternativo: programa WASI com stdin/stdout limitados.
- SHA-256 dos bytes `.wasm`, Ed25519 para assinatura de manifesto canônico e confiança configurada localmente.
- Filesystem endereçado por conteúdo para blobs + SQLite para índice, pins e histórico.
- Transferência em blocos, arquivos temporários (*staging*), hash final e commit atômico.
- Um workspace Rust modular; extrair crates apenas quando houver limites de responsabilidade reais.

**Não escolher agora:** versões específicas de crates, esquema de assinatura canônica, serialização do manifesto, codec/framing definitivo, tamanho de bloco definitivo, ABI WIT definitiva e mecanismo final de isolamento adicional no Linux. Esses itens exigem testes ou ADRs.

Leia `docs/architecture.md` para fluxos, fronteiras e registro das decisões em aberto.

## 5. Trabalho obrigatório por tarefa

**Antes de escrever código**, responda de modo sucinto:

1. Qual é a tarefa exata e em qual fase ela pertence?
2. Quais **3–6 conceitos** o desenvolvedor deve estudar para entender esta tarefa? Explique o motivo de cada um.
3. Quais são entradas, saídas, estados, erros e invariantes?
4. Quais testes demonstrarão caso normal, caso negativo e limite/falha?
5. Há decisão arquitetural a registrar ou uma mudança de escopo a aprovar?

**Ao implementar:**

- Faça **uma mudança pequena por vez**, revisável e com testes proporcionais; não reescreva o repositório.
- Prefira código legível, contratos explícitos, erros tipados e tratamento correto de falhas.
- Evite `unwrap` e `expect` com dados externos e caminhos de produção; não use fallback inseguro.
- Não aloque memória a partir de tamanhos declarados por peers sem limites e não carregue blobs inteiros em RAM sem necessidade.
- Não adicione bibliotecas sem justificar função, manutenção, superfície de ataque e alternativas.
- Não introduza dependências entre rede e runtime que façam download executar código automaticamente.
- Não amplie filesystem, rede, subprocessos, permissões ou autonomia da IA sem revisão humana explícita.
- Não altere formato de pacotes, protocolo público, modelo de identidade/confiança ou limites de segurança sem propor ADR.
- Não execute código ou artefatos não confiáveis fora do laboratório controlado.
- Não realize `git push`, publique pacotes ou execute ações destrutivas sem pedido explícito.

**Depois da mudança**, entregue:

1. Arquivos alterados e justificativa do diff.
2. Comandos de teste efetivamente executados e resultados observados (ou informar claramente que não foram executados).
3. Casos ainda não cobertos, riscos e próximo passo **dentro da mesma fase**.
4. Três perguntas técnicas curtas para o desenvolvedor demonstrar entendimento.

## 6. Segurança e observabilidade

- Proteja chaves privadas e segredos; jamais grave dados sensíveis em logs, fixtures públicos ou commits.
- Hash e assinatura verificam propriedades distintas; chave conhecida não significa código benigno.
- Prefira armazenamento temporário fora do namespace de blobs válidos até verificação integral.
- Trate saídas do Wasm e mensagens remotas como texto/binário hostil, inclusive escapes ANSI.
- Se não puder comprovar limites de Wasmtime em um teste, indique a limitação; não afirme isolamento comprovado.
- Use logs estruturados progressivamente, correlacionando peer, operação, digest e resultado, sem registrar conteúdo sensível.
- Modele falhas de rede, reinicialização, blobs truncados, peers maliciosos e execução que consome recursos.

## 7. ADRs e documentação

Registre decisões significativas em `docs/decisions/ADR-NNN-nome.md` quando essa pasta for criada, com:

`Contexto → Alternativas → Decisão → Consequências → Riscos → Experimento de validação → Status/Data`.

Estados possíveis: `proposto`, `aceito`, `substituído`, `rejeitado`. **Não marque como aceito sem decisão do responsável.**

Quando comportamento real divergir do blueprint, documente a evidência; não reescreva silenciosamente o requisito. Atualize `docs/architecture.md` e `docs/ROADMAP.md` apenas conforme decisões e resultados confirmados.

## 8. Gate e modo de aprendizagem

- **Fase ativa: Fase 1 — primeira comunicação autenticada entre dois nós.**
- Gate da Fase 0: aprovado pelo responsável em 22/10/2026 após leitura e revisão dos documentos. O gate da Fase 1 depende de testes observados, não só de código escrito.
- **Etapa ativa da Fase 1: 1.1 — fundamentos e desenho da conexão.** Ler `docs/FASE-01.md` antes de implementar; em seguida, concluir 1.2, 1.3 e 1.4 sequencialmente.
- Antes de avançar, mostre o que funciona, as falhas reproduzidas, o que permanece aberto e um pequeno teste de compreensão.

A finalidade da IA é **acelerar a engenharia sem substituir o entendimento do autor**. Explique escolhas e mecanismos; não transforme esta regra em uma exigência de estudar toda uma biblioteca antes de iniciar um experimento.
