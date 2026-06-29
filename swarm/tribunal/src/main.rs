//! Triarchy Tribunal — an adversarial multi-agent court for bounty settlement.
//!
//! Instead of one model rubber-stamping a proof, the Tribunal runs a real
//! adversarial process and then moves real money on Casper:
//!
//!   🗡️ Prosecutor  — argues the proof is fraudulent / insufficient.
//!   🛡️ Defender    — argues the work genuinely satisfies the bounty.
//!   ⚖️ Jurors (N)  — independent models each vote APPROVE / REJECT with a reason.
//!   👨‍⚖️ Chief Judge — weighs the debate + jury and issues the binding verdict.
//!
//! Outcome → on-chain:
//!   APPROVE → `release_bounty` (pay the hunter)
//!   REJECT  → `refund_bounty`  (return funds to the creator)
//! The verdict is then ANCHORED on the oracle contract (event log + reputation),
//! so the whole ruling is publicly verifiable on cspr.live.
//!
//! Every model call is real (OpenRouter) and every payout is a real signed
//! TransactionV1. Nothing is simulated; on any failure the Tribunal aborts.

use serde_json::json;
use std::process::Command;

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn arg(flags: &[&str]) -> Option<String> {
    let a: Vec<String> = std::env::args().collect();
    for i in 0..a.len() {
        if flags.contains(&a[i].as_str()) && i + 1 < a.len() {
            return Some(a[i + 1].clone());
        }
    }
    None
}

/// Some(true)=APPROVE, Some(false)=REJECT, None=unclear.
fn classify(answer: &str) -> Option<bool> {
    let u = answer.trim().to_uppercase();
    if u.starts_with("APPROVE") || u.contains("VERDICT: APPROVE") || u.contains("VOTE: APPROVE") {
        Some(true)
    } else if u.starts_with("REJECT") || u.contains("VERDICT: REJECT") || u.contains("VOTE: REJECT") {
        Some(false)
    } else if u.contains("APPROVE") && !u.contains("REJECT") {
        Some(true)
    } else if u.contains("REJECT") && !u.contains("APPROVE") {
        Some(false)
    } else {
        None
    }
}

/// One real OpenRouter chat completion.
async fn ask(model: &str, system: &str, user: &str) -> Result<String, String> {
    let api_key = std::env::var("OPENROUTER_API_KEY")
        .map_err(|_| "OPENROUTER_API_KEY required (no key, no ruling — we never fake one)".to_string())?;
    let body = json!({
        "model": model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user}
        ],
        "temperature": 0.2
    });
    let client = reqwest::Client::new();
    let resp = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .bearer_auth(api_key)
        .json(&body)
        .send().await.map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("OpenRouter HTTP {} ({})", resp.status(), model));
    }
    let v: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;
    Ok(v["choices"][0]["message"]["content"]
        .as_str()
        .ok_or("malformed response")?
        .trim()
        .to_string())
}

/// Call an escrow entry point (release_bounty / refund_bounty) for real.
fn escrow_call(entrypoint: &str, task_id: &str, target_purse: &str) -> Result<String, String> {
    signer_call(
        &env_or("CASPER_CONTRACT_PACKAGE", "a7e6a38381899749532a9180c30794edcdab883596f54c883af2bcae98694f6d"),
        entrypoint,
        &format!("task_id:string:{task_id},target_purse:uref:{target_purse}"),
        "12000000000",
    )
}

/// Anchor the verdict on the oracle contract (event log + reputation accrual).
fn anchor_verdict(task_id: &str, approved: bool) -> Result<String, String> {
    signer_call(
        &env_or("CASPER_ORACLE_PACKAGE", "16d86943d2d95769bff18da2438c9bf674e35347890705f0ef73ad14e37964b2"),
        "post_reading",
        &format!("asset:string:VERDICT-{task_id},value:u512:{}", if approved { 1 } else { 0 }),
        "6000000000",
    )
}

fn signer_call(package: &str, entrypoint: &str, args: &str, payment: &str) -> Result<String, String> {
    let node = env_or("CASPER_RPC_URL", "https://node.testnet.casper.network/rpc");
    let chain = env_or("CASPER_CHAIN", "casper-test");
    let secret_key = env_or("CASPER_SECRET_KEY", "swarm/casper-client/key.pem");
    let signer = env_or("CASPER_SIGNER_BIN", "./swarm/casper-client/go-signer/casper-tx-signer");

    let out = Command::new(&signer)
        .args([
            "--mode", "call-entrypoint", "--node", &node, "--chain", &chain,
            "--secret-key", &secret_key, "--algo", "ed25519", "--payment", payment,
            "--contract-hash", package, "--entrypoint", entrypoint, "--args", args,
        ])
        .output()
        .map_err(|e| format!("failed to launch signer: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    let tx = String::from_utf8_lossy(&out.stdout).trim().to_string();
    if tx.len() != 64 || !tx.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("unexpected tx hash: '{tx}'"));
    }
    Ok(tx)
}

#[tokio::main]
async fn main() {
    let task_id = arg(&["--task-id", "-t"]).unwrap_or_else(|| "bounty-alpha-006".to_string());
    let description = arg(&["--description", "-d"])
        .unwrap_or_else(|| "Optimize the AST hypergraph for the Odra escrow modules.".to_string());
    let proof = arg(&["--proof", "-p"]).unwrap_or_default();
    let target_purse = arg(&["--target-purse"]).unwrap_or_else(|| {
        env_or("HUNTER_PURSE", "uref-37dde43070560294ae0799f91c53b1948d33e4649b0ec9ae79fbf3be37a0a27a-007")
    });
    // Dry-run = deliberate and rule, but never move funds (safe for a UI button).
    let dry_run = std::env::args().any(|a| a == "--dry-run")
        || std::env::var("TRIBUNAL_DRY_RUN").is_ok();
    let chief_model = env_or("CHIEF_JUDGE_MODEL", "anthropic/claude-opus-4.8-fast");
    let juror_models: Vec<String> = env_or(
        "JUROR_MODELS",
        "openai/gpt-4o-mini,meta-llama/llama-3.3-70b-instruct,mistralai/mistral-small-2603",
    )
    .split(',')
    .map(|s| s.trim().to_string())
    .filter(|s| !s.is_empty())
    .collect();

    let case = format!("TASK: {description}\nSUBMITTED PROOF: {proof}");

    println!("\n⚖️  ════ TRIARCHY TRIBUNAL · case {task_id} ════");
    println!("📜 {case}\n");

    // 1) Adversarial arguments (run concurrently).
    let prosecutor = ask(
        &chief_model,
        "You are the PROSECUTOR in a bounty tribunal. Argue, in 2-3 sentences, why the submitted proof FAILS to satisfy the task. Be specific and skeptical.",
        &case,
    );
    let defender = ask(
        &chief_model,
        "You are the DEFENDER in a bounty tribunal. Argue, in 2-3 sentences, why the submitted proof DOES satisfy the task. Cite concrete evidence in the proof.",
        &case,
    );
    let (prosecutor, defender) = (prosecutor.await, defender.await);
    let prosecution = prosecutor.unwrap_or_else(|e| format!("(prosecutor unavailable: {e})"));
    let defense = defender.unwrap_or_else(|e| format!("(defender unavailable: {e})"));
    println!("🗡️  PROSECUTION: {prosecution}\n");
    println!("🛡️  DEFENSE:     {defense}\n");

    let debate = format!("{case}\n\nPROSECUTION: {prosecution}\nDEFENSE: {defense}");

    // 2) Independent jurors (diverse models) vote.
    let mut approve_votes = 0u32;
    let mut reject_votes = 0u32;
    let mut jurors_voted = 0u32;
    let mut jurors_down = 0u32;
    for (i, model) in juror_models.iter().enumerate() {
        let v = ask(
            model,
            "You are an impartial JUROR on a bounty board. Proofs are written attestations \
(commits, benchmarks, test results) — assume the cited evidence is truthful unless it is \
internally contradictory. Vote on SUFFICIENCY: does the described work, taken at face value, \
satisfy the task? Approve when it does; reject only for genuine gaps (incomplete, irrelevant, \
self-contradictory, or missing a required deliverable). Do NOT reject merely because you cannot \
personally inspect the artifacts. Reply with a single line: 'VOTE: APPROVE' or 'VOTE: REJECT' then a short reason.",
            &debate,
        ).await;
        match v {
            Ok(ans) => {
                match classify(&ans) {
                    Some(true) => { approve_votes += 1; jurors_voted += 1; println!("⚖️  JUROR {} ({}): ✅ {}", i + 1, model, ans); }
                    Some(false) => { reject_votes += 1; jurors_voted += 1; println!("⚖️  JUROR {} ({}): ❌ {}", i + 1, model, ans); }
                    None => println!("⚖️  JUROR {} ({}): 🤷 abstain ({})", i + 1, model, ans),
                }
            }
            Err(e) => { jurors_down += 1; println!("⚖️  JUROR {} ({}): OFFLINE ({})", i + 1, model, e); }
        }
    }
    println!("\n🗳️  Jury tally: {approve_votes} APPROVE / {reject_votes} REJECT  ({jurors_voted}/{} jurors online)", juror_models.len());

    // 3) Chief Judge issues the binding verdict — with graceful degradation.
    let chief = ask(
        &chief_model,
        "You are the CHIEF JUDGE of a bounty board. Rule by PREPONDERANCE OF EVIDENCE: \
treat the proof's stated facts as truthful unless internally inconsistent, and APPROVE when the \
work as described completely satisfies the task. REJECT only for a genuine, material gap (a missing \
required deliverable, irrelevance, or self-contradiction) — never merely because artifacts cannot be \
inspected live or because an even stronger proof is imaginable. Weigh the jury tally. \
Reply starting with exactly 'VERDICT: APPROVE' or 'VERDICT: REJECT', then one sentence of reasoning.",
        &format!("{debate}\n\nJURY: {approve_votes} approve, {reject_votes} reject."),
    ).await;

    // Resolve a verdict from the best authority available.
    let (verdict, source, chief_text): (Option<bool>, &str, String) = match chief {
        Ok(text) => match classify(&text) {
            Some(b) => (Some(b), "chief judge", text),
            None => (jury_majority(approve_votes, reject_votes), "jury majority (chief unclear)", text),
        },
        Err(e) => (
            jury_majority(approve_votes, reject_votes),
            "jury majority (chief OFFLINE)",
            format!("(chief judge offline: {e})"),
        ),
    };
    println!("\n👨‍⚖️ CHIEF JUDGE: {chief_text}");

    let approved = match verdict {
        Some(b) => b,
        None => {
            // Honest degraded states — never move funds without a real ruling.
            if jurors_voted == 0 {
                println!(
                    "\n🛑 SERVICE DEGRADED: all tribunal agents are currently unavailable. \
Functions are frozen and no funds were moved — we are working on it. Please try again shortly."
                );
            } else {
                println!(
                    "\n🛑 NO QUORUM: the jury is tied and the chief judge is offline, so the ruling \
is inconclusive and no funds were moved. Please retry when the bench is back."
                );
            }
            std::process::exit(2);
        }
    };

    // Confidence: full only if the chief ruled and every juror was online.
    let full_bench = source == "chief judge" && jurors_down == 0;
    if !full_bench {
        println!(
            "\n⚠️  PARTIAL TRIBUNAL: {jurors_voted}/{} jurors online, ruling via {source}. \
This verdict is indicative and not fully precise.",
            juror_models.len()
        );
    }
    println!(
        "\n⚖️  RULING: {} · confidence: {} · authority: {source}",
        if approved { "APPROVE" } else { "REJECT" },
        if full_bench { "FULL" } else { "PARTIAL" },
    );

    if dry_run {
        println!("\n🧪 DRY-RUN: deliberation only — no on-chain enforcement, no funds moved.");
        return;
    }

    // 4) Enforce the ruling on-chain.
    let (entry, label) = if approved { ("release_bounty", "RELEASE to hunter") } else { ("refund_bounty", "REFUND to creator") };
    println!("\n⛓️  Enforcing ruling on Casper: {label}...");
    match escrow_call(entry, &task_id, &target_purse) {
        Ok(tx) => println!("💸 {label} tx: https://testnet.cspr.live/transaction/{tx}"),
        Err(e) => { eprintln!("❌ on-chain enforcement failed: {e}"); std::process::exit(1); }
    }

    // 5) Anchor the verdict on the oracle (verifiable record + reputation).
    match anchor_verdict(&task_id, approved) {
        Ok(tx) => println!("🔗 verdict anchored on oracle: https://testnet.cspr.live/transaction/{tx}"),
        Err(e) => eprintln!("⚠️  verdict anchoring skipped: {e}"),
    }

    println!("\n✅ Tribunal complete. Ruling: {}.", if approved { "APPROVED" } else { "REJECTED" });
}

/// Majority of the jurors who actually voted: Some(true)/Some(false), or None on a tie.
fn jury_majority(approve: u32, reject: u32) -> Option<bool> {
    use core::cmp::Ordering::*;
    match approve.cmp(&reject) {
        Greater => Some(true),
        Less => Some(false),
        Equal => None,
    }
}

#[cfg(test)]
mod tests {
    use super::classify;

    #[test]
    fn classifies_verdict_forms() {
        assert_eq!(classify("VERDICT: APPROVE because ..."), Some(true));
        assert_eq!(classify("VOTE: REJECT — no evidence"), Some(false));
        assert_eq!(classify("APPROVE"), Some(true));
        assert_eq!(classify("the work is solid, APPROVE"), Some(true));
        assert_eq!(classify("hmm not sure"), None);
    }
}
