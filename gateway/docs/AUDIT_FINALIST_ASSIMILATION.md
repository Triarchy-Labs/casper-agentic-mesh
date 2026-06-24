# x402 Gateway — Finalist Architecture Assimilation Audit
> Date: 2026-04-28 | Triarchy Labs Security Audit

## Источники (6 скачанных репозиториев)
```
Stellar Hacks Finalists (Public GitHub repos):
├── Cards402/       # 1st Place — Virtual Visa for AI agents
├── clevercon/      # 2nd Place — Trustless AI marketplace
├── rendergate/     # 3rd Place — Pay-per-render headless browser
├── x402-mcp-stellar-template/  # 4th Place — Drop-in middleware
├── toll/           # 5th Place — Stripe for MCP servers
└── devloot-mcp-server/  # MCP prediction market bot (AgentFund)
```

---

## TOLL (5th Place) — Самый архитектурно зрелый

**Путь:** `toll/packages/gateway/src/` (29 файлов)

### Механики для ассимиляции:

#### 1. ReplayGuard (middleware.ts:36-55) — КРИТИЧЕСКАЯ
Защита от повторного использования платёжных подписей.
```typescript
// TTL = 5 минут. Map<signature, timestamp>. cleanup() по cutoff.
class ReplayGuard {
  private used = new Map<string, number>()
  check(sig: string): boolean  // true = уже использовалась
  mark(sig: string): void      // записать после успешной оплаты
}
```
**Наш аналог:** Отсутствует. Один txHash можно переиспользовать бесконечно.

#### 2. SpendingPolicy (spendingPolicy.ts) — 136 строк — ВЫСОКИЙ
Дневные лимиты per-caller + allowlist/blocklist.
```
1. allowedCallers / blockedCallers — whitelist/blacklist Stellar адресов
2. maxPerCall — максимальная цена одного вызова
3. maxDailyPerCaller — дневной бюджет per-caller
4. maxDailyGlobal — общий дневной бюджет всей платформы
```
**Наш аналог:** Отсутствует.

#### 3. BudgetDegradation (budgetDegradation.ts) — 52 строки
Три стратегии при исчерпании бюджета:
- `reject` — отказ
- `downgrade` — предложить более дешёвый инструмент (downgradeMap)
- `queue` — поставить в очередь с TTL
**Наш аналог:** Отсутствует.

#### 4. Negotiator (negotiation.ts) — 114 строк
HMAC-signed price-lock tokens. Скидки по объёму/лояльности/бандлу.
**Наш аналог:** Отсутствует.

#### 5. EarningsTracker + Redis Store
Персистентный трекинг заработков с поддержкой Redis для distributed state.
**Наш аналог:** `agent_registry.ts` — только in-memory.

---

## RENDERGATE (3rd Place)

**Путь:** `rendergate/server.js` (186 строк)

#### 6. isAllowedUrl() — SSRF Protection (строки 22-38)
Блокирует localhost, 127.0.0.1, приватные подсети (10.x, 172.16-31, 192.168), IPv6 literals.
**Наш аналог:** Отсутствует. `api/hire` не фильтрует URL.

#### 7. getPayerAddress() — Soroban Auth Extraction (строки 41-62)
Извлекает адрес плательщика из base64 `payment-signature` header → XDR → sorobanCredentialsAddress.
**Наш аналог:** Мы берём publicKey из requestAccess(), но не из payment header.

#### 8. Автоматический Refund (строки 130-147)
При сбое рендера: `sendRefund(payerAddress, REFUND_AMOUNT, reason)`.
**Наш аналог:** Отсутствует. Если задача падает, деньги пропадают.

---

## CLEVERCON (2nd Place)

**Путь:** `clevercon/contracts/` — 2 Rust Soroban контракта

#### 9. agent-vault — On-chain изоляция средств агента
#### 10. budget-guardian — On-chain бюджетный контроль
**Наш аналог:** Всё off-chain (JSON файл).

**Путь:** `clevercon/packages/` — 5 пакетов
- `agents/` — Агентский фреймворк
- `orchestrator/` — Координация агентов
- `registry/` — Реестр агентов
- `dashboard/` — React дашборд
- `common/` — Общие утилиты

---

## DEVLOOT MCP SERVER (AgentFund / Артур)

**Путь:** `devloot-mcp-server/src/server.ts` — 1403 строки

#### 11. Полноценный MCP Server с 20+ tools
`buy_shares`, `sell_shares`, `claim_winnings`, `create_market`, `enter_lp_active`, `refund_shares` и т.д.
**Наш аналог:** Мы не создали MCP server. Наш бот — REST API.

---

## БАГИ В НАШЕМ КОДЕ

| # | Файл | Строка | Проблема | Fix |
|---|---|---|---|---|
| 1 | `bounties/page.tsx` | 76 | Пустой `catch (e) {}` | Добавить `console.error` |
| 2 | `bounties/page.tsx` | 53 | Fallback `"GXYZ..."` | Throw error |
| 3 | `bounties/page.tsx` | 55 | `setTimeout` hardcoded 1500ms | Убрать для production |
| 4 | `dashboard/page.tsx` | 284 | TOP-UP кнопка без onClick | Добавить handler |
| 5 | `dashboard/page.tsx` | 271 | Model selector без onChange | Добавить state |
| 6 | `xrpl-transactor.ts` | 82 | `split("\\\\n")` двойное экранирование | `split("\\n")` |
| 7 | Корень проекта | — | `bounties.json` не в `.gitignore` | Добавить |
| 8 | `bounties/page.tsx` | 46 | Русский комментарий | Перевести |
| 9 | Dashboard models | — | Список 2024 года, не 2026 | Обновить |

---

## ПРИОРИТЕТ АССИМИЛЯЦИИ

| P | Механика | Источник | Строки | Эффект |
|---|---|---|---|---|
| P0 | ReplayGuard | Toll | ~20 | Закрывает replay-атаку |
| P0 | SpendingPolicy | Toll | ~136 | Бюджетные лимиты |
| P1 | SSRF Protection | RenderGate | ~15 | Блокирует SSRF |
| P1 | Bug Fixes (9 шт) | Наш код | ~30 | Стабильность |
| P2 | BudgetDegradation | Toll | ~52 | Graceful degradation |
| P2 | Auto-Refund | RenderGate | ~30 | UX при сбоях |
| P3 | Negotiator | Toll | ~114 | Динамические цены |
| P3 | EarningsTracker | Toll | ~60 | Персистентный трекинг |
