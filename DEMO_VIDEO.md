# Demo Video — shooting script & storyboard

Target length **~3:00**. Narration in English (international jury). RU voiceover is
fine — keep the on-screen text/URLs in English. Record screen at 1080p+. Every hash
shown is real and already on-chain, so nothing can fail live.

Pre-roll setup (have these open in tabs/terminals before recording):
- Terminal A in repo root, env ready: `export OPENROUTER_API_KEY=… OPENROUTER_MODEL=anthropic/claude-opus-4.8-fast`
- Browser tab 1: https://testnet.cspr.live/contract-package/a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d
- Browser tab 2: the gateway `npm run dev` → http://localhost:3000/dashboard
- `DEPLOYMENTS.md` open in the editor.

---

### Scene 1 — Hook (0:00–0:20)
**On screen:** README title / a single line of the manifesto.
**Narration:** "Most web3-AI projects ship one agent holding a private key that
blindly trusts its inputs. We built the opposite: a machine-to-machine bounty
economy on Casper where AI agents escrow funds and verify each other's work —
and it's live on testnet, not a mockup."

### Scene 2 — It's real on-chain (0:20–0:55)
**On screen:** cspr.live contract-package tab; then scroll `DEPLOYMENTS.md`.
**Narration:** "Here's the escrow contract deployed on Casper testnet. Deploy,
init, register, create_bounty, release, refund — the full state machine, every
transaction successful. We even read the bounty state back from the contract's
dictionary: one bounty Released, one Refunded. All hashes are in the repo."
**Show:** the transactions table + the state read-back table.

### Scene 3 — Live transaction (0:55–1:30)
**On screen:** Terminal A → run `./run_demo.sh`.
**Narration:** "This isn't a recording of fake logs. The demo script hits the
live node, signs a real TransactionV1, and broadcasts it."
**Action:** when the `register_agent` tx link prints, **click it** → it opens on
cspr.live as a Success. Pause on the explorer page for 2–3 seconds.

### Scene 4 — The agentic loop (THE money shot) (1:30–2:35)
**On screen:** Terminal A.
**Step 4a — lock a bounty** (optional if pre-locked): briefly mention 10 CSPR are
locked in escrow for `bounty-alpha-004`.
**Step 4b — run the judge on a WEAK proof:**
```bash
cargo run -q -p bounty-judge -- --task-id bounty-alpha-004 \
  --description "Optimize the AST hypergraph for the Odra escrow modules." \
  --proof "https://github.com/Triarchy-Labs/casper-agentic-mesh/pull/1"
```
**Narration:** "An autonomous agent asks an LLM to verify the submitted proof.
Weak proof — just a bare link — and the agent rejects it. No funds move."
**Show:** the `REJECT: …` line.
**Step 4c — run the judge on a STRONG proof:**
```bash
cargo run -q -p bounty-judge -- --task-id bounty-alpha-004 \
  --description "Optimize the AST hypergraph for the Odra escrow modules; reduce redundant edges." \
  --proof "PR #1 merged (commit 4ad2744). Edges 18412->11067 (-39.9%); latency 240ms->121ms; 37 tests pass; reproducible benchmark + CI."
```
**Narration:** "Now a real proof with metrics and tests. The LLM approves — and
the agent autonomously releases the escrow on-chain, paying the hunter."
**Action:** when the `🔗 https://testnet.cspr.live/transaction/…` link prints,
**click it** → show the Success on the explorer. This is the core message: an AI
decision triggering a real on-chain payout.

### Scene 5 — Gateway + close (2:35–3:00)
**On screen:** gateway dashboard (the 4 tabs / cinematic UI). If a wallet is
connected, run one task and show the **PAYMENT** row with the cspr.live link.
**Narration:** "The gateway verifies every payment against the live ledger — no
bypass, no mock hashes. Stake-weighted slashing, ZK proofs of execution, and a
soulbound reputation layer are on our roadmap. Triarchy Agentic Mesh — the trust
layer for the agent economy, live on Casper."

---

## One-line proof reel (pin in the description)
- Contract: https://testnet.cspr.live/contract-package/a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d
- Agent-driven payout: https://testnet.cspr.live/transaction/c333c9d1513c633d161627c39ff9cb3cf28ef2f3acf3cda3d19c0d55f9dfcb89
- Full ledger: `DEPLOYMENTS.md`

## Tips
- Keep terminal font large; pre-clear scrollback.
- If `run_demo.sh` step 3 broadcasts a new tx, that's fine — it's cheap and real.
- Do NOT show the raw OPENROUTER_API_KEY on screen (export it before recording).
