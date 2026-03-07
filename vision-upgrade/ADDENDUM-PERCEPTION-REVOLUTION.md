# ADDENDUM: THE PERCEPTION REVOLUTION
## Fixing the Fundamental Flaw That Makes Vision Expensive and Dumb

> **Trigger:** Architectural review — "AgenticVision uses too many tokens for mapping websites and screenshots. Every website is different. There must be a more optimized way."
> **Date:** March 2026
> **Status:** CRITICAL — This addendum supersedes all perception-related sections of the Vision architecture. Token waste is not a UX problem — it's a design philosophy failure.
> **Principle:** If an agent visits the same website twice and pays full screenshot cost both times, we have failed. If an agent takes a full-page screenshot to find a price, we have failed. If perception cost scales with page complexity rather than task complexity, we have failed.

---

## 1. THE ROOT CAUSE (Be Honest About It)

Here's what every agent built on AgenticVision today experiences. They need to find a price on Amazon. They need to post a tweet. They need to check a GitHub status. **They pay full-page screenshot costs. Every time. For every site. For every task.**

```
CURRENT FLOW (BROKEN):
══════════════════════

  Task: "What is the price of this laptop?"
      ↓
  Capture full-page screenshot
      ↓
  Send ~2,000+ vision tokens to multimodal LLM
      ↓
  LLM processes the ENTIRE PAGE: navigation bar,
  recommendations, reviews, Q&A, footer, ads...
      ↓
  Finally extracts: "$1,299"
      ↓
  Total cost: 2,000+ tokens
  Actual information needed: ~5 tokens
  Waste ratio: 400:1

                              THIS IS THE FAILURE POINT
                              
  The current approach makes NO distinction between:
  • A task requiring visual understanding (chart analysis)
  • A task that needs structured data (price lookup)
  • A task that needs interaction (form submission)
  • A task on a page we've visited 100 times before
  
  RESULT: "AgenticVision uses so much token for mapping
           websites and screenshots."
```

This is not a performance bug. This is a **design philosophy failure**. We built perception that treats every page visit as if the agent is blind, visiting the internet for the first time, needing to process everything it sees to understand anything.

**What every browser automation tool got wrong:** They all start with the DOM or a screenshot and throw it at an LLM. Playwright, Puppeteer, Selenium, browser-use, computer-use — they all make the same mistake: **capture first, understand later, pay the full cost every time.**

**What we got wrong:** AgenticVision built extraordinary infrastructure (.avis format, visual diff, multi-context, grounding) and then used the most expensive perception primitive as the default. Screenshots should be the exception. They became the rule.

---

## 2. THE FIX: THE ADAPTIVE PERCEPTION STACK (ZERO TOKEN DEFAULT)

The solution is a layered perception architecture where the agent starts at the cheapest layer and only escalates when the task genuinely requires it. Five layers. The first layer costs zero vision tokens. The second layer costs near-zero. Full screenshots — the current default — become Layer 4: the last resort.

```
THE ADAPTIVE PERCEPTION STACK:
═══════════════════════════════

  ┌──────────────────────────────────────────────────────────────────┐
  │                                                                    │
  │   LAYER 0: SEMANTIC DOM EXTRACTION (Zero vision tokens.)          │
  │   ════════════════════════════════════════════════════════════     │
  │                                                                    │
  │   Every browser exposes an Accessibility Tree — a structured      │
  │   semantic representation of what's on screen. Text, roles,       │
  │   labels, interactive elements, live regions. All available       │
  │   without taking a single screenshot.                             │
  │                                                                    │
  │   AgenticVision uses the a11y tree + CSS selector engine to       │
  │   extract TYPED STRUCTURED DATA from any page:                    │
  │                                                                    │
  │   Task: "price of this product"                                   │
  │   → query: [class*="price"] | [itemprop="price"] | .a-price       │
  │   → result: "$1,299"                                              │
  │   → vision tokens used: 0                                         │
  │   → text tokens used: ~15                                         │
  │                                                                    │
  │   GUARANTEE: Any page with readable DOM structure can be          │
  │   queried for structured data at zero vision cost.                │
  │                                                                    │
  ├──────────────────────────────────────────────────────────────────┤
  │                                                                    │
  │   LAYER 1: SITE GRAMMAR (Amortized to near-zero.)                │
  │   ════════════════════════════════════════════════════════         │
  │                                                                    │
  │   The FIRST visit to any site pays a learning cost.               │
  │   Every subsequent visit: FREE.                                   │
  │                                                                    │
  │   On first visit, AgenticVision extracts and stores the           │
  │   STRUCTURAL GRAMMAR of the site: which selectors identify        │
  │   which content types, how navigation works, where interactive    │
  │   elements live, how pagination functions.                        │
  │                                                                    │
  │   Stored in .avis as a Site Grammar Record.                       │
  │   Versioned. Drift-detected. Shared across agents (Collective).   │
  │                                                                    │
  │   Visit 1: Learning cost (one-time)                               │
  │   Visit 2-∞: Grammar lookup + Layer 0 → 0 tokens                 │
  │                                                                    │
  │   GUARANTEE: Visited sites are free to re-visit.                  │
  │   Same-site tasks get cheaper the more you use them.              │
  │                                                                    │
  ├──────────────────────────────────────────────────────────────────┤
  │                                                                    │
  │   LAYER 2: INTENT-SCOPED EXTRACTION (Proportional cost.)         │
  │   ═════════════════════════════════════════════════════════        │
  │                                                                    │
  │   Before ANY perception happens, the agent declares intent.       │
  │   Intent gates what gets extracted. Nothing outside scope         │
  │   enters the token budget.                                        │
  │                                                                    │
  │   Task: find_price → extract [.price, [itemprop=price]]           │
  │   Task: post_tweet → extract [textarea, [data-testid=tweet]]      │
  │   Task: find_link   → extract [a[href], [role=link]]              │
  │                                                                    │
  │   The navigation bar, footer, recommendations, ads, sidebar —     │
  │   never loaded. Not filtered after the fact. Never seen.          │
  │                                                                    │
  │   Token cost: proportional to answer complexity,                  │
  │   NOT to page complexity.                                          │
  │                                                                    │
  │   GUARANTEE: A simple task on a complex page costs                │
  │   the same as a simple task on a simple page.                     │
  │                                                                    │
  ├──────────────────────────────────────────────────────────────────┤
  │                                                                    │
  │   LAYER 3: DELTA VISION (Only perceive what changed.)            │
  │   ═════════════════════════════════════════════════════            │
  │                                                                    │
  │   For agents that revisit pages or monitor changing content,      │
  │   only CHANGES are perceived. Not the page. The diff.             │
  │                                                                    │
  │   First visit: full extraction → stored as structural baseline    │
  │   Subsequent visits: DOM diff against baseline → only changed     │
  │   nodes extracted                                                  │
  │                                                                    │
  │   Amazon product page: price changed.                             │
  │   → Perceive: 1 changed node (~15 tokens)                        │
  │   → NOT: re-extract entire page (~2,000 tokens)                  │
  │                                                                    │
  │   Twitter feed: 3 new tweets.                                     │
  │   → Perceive: 3 new article nodes (~150 tokens)                  │
  │   → NOT: re-render and screenshot entire feed (~3,000 tokens)    │
  │                                                                    │
  │   GUARANTEE: Monitoring tasks scale with change volume,           │
  │   not page volume. Watching a stable page is free.               │
  │                                                                    │
  ├──────────────────────────────────────────────────────────────────┤
  │                                                                    │
  │   LAYER 4: SCOPED SCREENSHOT (Visual content only.)              │
  │   ═════════════════════════════════════════════════════            │
  │                                                                    │
  │   Screenshots are only triggered when:                            │
  │   • Content is genuinely visual (charts, diagrams, images)        │
  │   • Canvas/WebGL renders that have no DOM representation          │
  │   • CAPTCHA or visual verification required                       │
  │   • Site actively blocks DOM access                               │
  │   • Visual layout itself is the deliverable                       │
  │                                                                    │
  │   AND even then: SCOPED screenshots.                              │
  │   Not the full page. A bounding-box crop of the specific          │
  │   element that needs visual processing.                           │
  │                                                                    │
  │   Full page screenshot: ~2,000+ tokens                           │
  │   Scoped element screenshot: ~50-300 tokens                      │
  │                                                                    │
  │   GUARANTEE: When screenshots are taken, they are the             │
  │   minimum region necessary to answer the question.               │
  │                                                                    │
  └──────────────────────────────────────────────────────────────────┘
```

---

## 3. THE SITE GRAMMAR SYSTEM (The Core Invention)

The most important architectural invention in the Perception Revolution is **Site Grammar Crystallization** — the system that makes browsing get cheaper every time, not stay the same cost forever.

### 3.1 What is a Site Grammar?

A Site Grammar is not just a list of CSS selectors. It is the complete behavioral fingerprint of a website, stored as a typed data structure in `.avis`.

```
SITE GRAMMAR RECORD (.avis):
═════════════════════════════

site_grammar {
  domain: "amazon.com"
  grammar_version: "2026-Q1-v2"
  last_verified: timestamp
  structural_hash: "blake3:a4f8..."   # Detects if site changed
  
  // STRUCTURAL GRAMMAR — What lives where
  content_map {
    product_price:     [class*="a-price-whole"]
    product_title:     [id="productTitle"]
    product_rating:    [id="acrPopover"]
    product_images:    [id="imgTagWrapperId"] img
    add_to_cart:       [id="add-to-cart-button"]
    availability:      [id="availability"] span
    seller:            [id="merchant-info"]
    prime_badge:       [id="isPrimeBadge"]
    review_count:      [id="acrCustomerReviewLink"]
  }
  
  // BEHAVIORAL GRAMMAR — How to interact
  interaction_patterns {
    search {
      input:  [name="field-keywords"]
      submit: [id="nav-search-submit-button"]
      result_container: [class="s-result-list"]
    }
    pagination {
      type: "link-click"
      next: .a-last a
      current_page: [aria-current="page"]
    }
    add_to_cart {
      button: [id="add-to-cart-button"]
      confirmation: [id="huc-v2-order-row-confirm-text"]
      success_indicator: [id="NATC_SMART_WAGON_CONF_MSG_SUCCESS"]
    }
  }
  
  // STATE GRAMMAR — What indicates system state  
  state_indicators {
    loading:   [class*="loading"]
    error:     [class*="a-alert-error"]
    success:   [class*="a-alert-success"]
    captcha:   [id="captchacharacters"]
    logged_in: [id="nav-link-accountList"]
  }
  
  // NAVIGATION GRAMMAR — How to move around
  navigation {
    type: "multi-page"
    js_required: false
    spa_navigation: false
    back_button_safe: true
    session_state: "cookie"
  }
  
  // INTENT ROUTING — Which grammar section answers which intent
  intent_routes {
    find_price:       content_map.product_price
    add_to_cart:      interaction_patterns.add_to_cart
    search_products:  interaction_patterns.search
    check_stock:      content_map.availability
    check_reviews:    content_map.product_rating + content_map.review_count
  }
}
```

### 3.2 The Grammar Learning Pipeline

```
FIRST VISIT TO A NEW SITE:
════════════════════════════

  1. STRUCTURAL SCAN (Layer 0 — free)
     Extract full accessibility tree
     Extract all interactive elements
     Extract all semantic roles
     Cost: ~500 text tokens (one-time)
  
  2. GRAMMAR INFERENCE (LLM-assisted)
     Classify element types by ARIA roles + class patterns
     Identify intent-to-selector mappings
     Identify navigation patterns
     Identify state indicators
     Cost: ~1,000 tokens (one-time learning cost)
  
  3. GRAMMAR STORAGE
     Serialize to Site Grammar Record in .avis
     Index by domain + structural hash
     
  4. ALL FUTURE VISITS: GRAMMAR LOOKUP
     Domain lookup → Grammar record
     Intent → selector list
     Query DOM directly
     Cost: ~0-15 tokens per query


  ONE-TIME COST:    ~1,500 tokens to learn
  PER-VISIT COST:   ~0 tokens using grammar
  BREAK-EVEN:       2nd visit pays for itself
  VISIT 100:        100× cheaper than screenshot approach
```

### 3.3 Grammar Drift Detection

Sites change. Twitter redesigns its UI. Amazon reorganizes its product pages. The grammar goes stale. Detection must be automatic.

```
GRAMMAR DRIFT DETECTION:
═════════════════════════

  On every visit:
  1. Compute structural hash of key grammar nodes
  2. Compare against stored hash
  3. If match: use grammar as-is (0 tokens)
  4. If mismatch: PARTIAL re-learning
  
  PARTIAL RE-LEARNING (not full re-learn):
  ─────────────────────────────────────────
  • Only re-learn the sections that changed
  • If product_price selector fails → re-learn product section only
  • If navigation changed → re-learn navigation section only
  • Grammar accumulates corrections rather than being replaced
  
  DRIFT COST: ~200-500 tokens for a partial re-learn
              (10-30% of initial learning cost)
              
  DRIFT FREQUENCY: Most sites: quarterly minor changes
                   Some sites: weekly UI experiments
                   
  ADAPTIVE RESPONSE: High-drift sites → more frequent verification
                     Stable sites → annual verification sufficient
```

---

## 4. INTENT-SCOPED PERCEPTION (The Token Budget Gate)

Before any perception primitive is invoked, intent declaration is mandatory. This is not optional. It is architectural.

```rust
// Every perception call MUST include intent
pub struct PerceptionRequest {
    pub intent: PerceptionIntent,
    pub site: Option<SiteContext>,
    pub budget: TokenBudget,           // Hard cap on tokens allowed
    pub fallback: FallbackStrategy,    // What to do if layer fails
}

pub enum PerceptionIntent {
    // Structured data — Layer 0 only
    ExtractData { fields: Vec<DataField> },
    
    // Interactive — Layer 0 + 1
    FindInteractable { action: ActionType },
    VerifyState { expected: StatePattern },
    
    // Content analysis — Layer 2 + optional 3
    ReadContent { scope: ContentScope },
    MonitorChanges { baseline: StructuralSnapshot },
    
    // Visual only — Layer 4
    AnalyzeChart,
    ReadDocument { format: DocumentFormat },
    VerifyCaptcha,
    CaptureVisualState,
}
```

### 4.1 Token Budget Enforcement

```
TOKEN BUDGET SYSTEM:
═════════════════════

  Every perception request has a HARD token budget.
  If the task can be completed within budget: proceed.
  If not: escalate to user or fail gracefully.

  BUDGET TIERS:
  
  Tier 1 — Surgical (0-50 tokens)
  ─────────────────────────────────
  Intent: single-field data extraction
  Examples: price, stock, title, URL
  Approach: Grammar → DOM query → result
  
  Tier 2 — Focused (50-300 tokens)  
  ──────────────────────────────────
  Intent: multi-field extraction or simple interaction
  Examples: search result list, form fill, status check
  Approach: Grammar → DOM multi-query → result
  
  Tier 3 — Contextual (300-800 tokens)
  ──────────────────────────────────────
  Intent: page section analysis or complex navigation
  Examples: read article, find matching products, compare options
  Approach: DOM section extraction → text summary
  
  Tier 4 — Visual (800-2000 tokens)
  ────────────────────────────────────
  Intent: genuinely visual content
  Examples: chart, diagram, image, scanned document
  Approach: scoped screenshot → vision model
  
  Tier 5 — Full Page (2000+ tokens)   ← OLD DEFAULT. NOW: LAST RESORT
  ──────────────────────────────────────
  Intent: unknown site, completely unstructured content
  Examples: novel custom web app, blocked DOM, first visit to niche site
  Approach: full screenshot → vision model
  NOTE: Must be explicitly requested. Never automatic.

  ADAPTIVE ROUTING:
  The system automatically selects the cheapest tier
  that can satisfy the stated intent.
  The agent never overpays.
```

---

## 5. SOLVING THE "EVERY WEBSITE IS DIFFERENT" PROBLEM

This is the real challenge. Twitter and Amazon have completely different DOM structures, interaction patterns, and state systems. The current approach says: "Yes they're different, so take a screenshot and let the LLM figure it out." The Perception Revolution says: **"Yes they're different — and we store exactly how each one is different."**

```
THE BEHAVIORAL SCHEMA (The invention that doesn't exist anywhere else):
═══════════════════════════════════════════════════════════════════════

  Traditional approach:
    Different site → screenshot → expensive → works eventually
  
  AgenticVision approach:
    Different site → grammar → stored permanently → free forever

  TWITTER vs AMAZON vs GITHUB:
  
  Twitter Grammar:
  ────────────────
  tweet_compose:   [data-testid="tweetTextarea_0"]
  tweet_submit:    [data-testid="tweetButtonInline"]
  tweet_feed:      [aria-label="Timeline"] > article
  tweet_author:    article [data-testid="User-Name"]
  pagination_type: infinite-scroll
  state_loading:   [data-testid="cellInnerDiv"][style*="height"]
  
  Amazon Grammar:
  ────────────────
  product_price:   .a-price-whole
  search_input:    [name="field-keywords"]
  add_to_cart:     [id="add-to-cart-button"]
  pagination_type: numbered-pages
  state_loading:   .loading-spinner
  
  GitHub Grammar:
  ────────────────
  repo_files:      [role="gridcell"] .react-directory-filename-column
  file_content:    [data-testid="blob-content"]
  pr_status:       .State--merged | .State--open | .State--closed
  pagination_type: infinite-scroll
  state_loading:   .loading
  
  ─────────────────────────────────────────────────────────
  
  Result: The agent operates on EACH SITE through its unique
  grammar. Not through screenshots. Not through LLM guessing.
  Through learned, stored, verified behavioral knowledge.
  
  The "every website is different" problem becomes:
  "we know how each website is different, permanently."
```

---

## 6. THE COLLECTIVE GRAMMAR (Shared Learning)

The most powerful multiplier in the Perception Revolution is that grammar learning is collective. One agent learns Twitter — every agent gets Twitter for free.

```
COLLECTIVE GRAMMAR ARCHITECTURE:
══════════════════════════════════

  INDIVIDUAL AGENT (Current):
  ──────────────────────────────
  Agent A visits Twitter → learns grammar → stores locally
  Agent B visits Twitter → learns grammar → stores locally
  Agent C visits Twitter → learns grammar → stores locally
  
  1,000 agents × 1,500 tokens × 100 common sites = 150,000,000 tokens
  
  ─────────────────────────────────────────────────────────────
  
  COLLECTIVE GRAMMAR (AgenticVision + Evolve Sister):
  ──────────────────────────────────────────────────────
  Agent A visits Twitter → learns grammar → contributes to Collective
  Agent B looks up Twitter → Grammar exists → 0 learning cost
  Agent C looks up Twitter → Grammar exists → 0 learning cost
  
  First agent (per site): 1,500 tokens
  All subsequent agents:  0 tokens
  
  Grammar quality: improves with each contribution
  (multi-agent verification of selectors → higher confidence)
  
  GRAMMAR TIERS (by verification level):
  ────────────────────────────────────────
  
  ★ Community Grammar     → Contributed by single agent, unverified
  ★★ Verified Grammar     → Confirmed by 3+ agents independently
  ★★★ Canonical Grammar   → Certified by Agentra Labs, monthly updates
  
  COVERAGE:
  Top 1,000 websites: Canonical Grammars (ships with AgenticVision)
  Top 100,000 websites: Verified Grammars (community-contributed)
  Everything else: Agent-learned on first visit
  
  RESULT: 95%+ of web traffic already has a grammar.
          Agents almost never need to pay learning cost.
```

---

## 7. SOLVING CONVERSATION COMPACTION — FOR VISION

AgenticMemory's compaction problem is: LLMs forget conversation history. Vision has an equivalent problem: agents re-perceive pages they've already seen, paying full cost each time, because **visual perception has no memory**.

**Our solution: Perception happens once. Results persist. Grammar is the memory.**

```
TIMELINE OF VISUAL SESSIONS:
═════════════════════════════

  Session 1:
  ───────────
  Visit amazon.com/laptop → Grammar not found
  → Learn grammar (1,500 tokens, one-time)
  → Extract price (15 tokens)
  → Store: grammar + perception result
  
  Session 2 (same day, different conversation):
  ──────────────────────────────────────────────
  Visit amazon.com/laptop → Grammar FOUND ✓
  → Extract price (15 tokens)
  → Delta check: price changed? (5 tokens)
  → Total: 20 tokens
  
  Session 100 (3 months later):
  ──────────────────────────────
  Visit amazon.com/laptop → Grammar FOUND ✓
  → Grammar drift check: 2 selectors changed → partial re-learn (200 tokens)
  → Extract price (15 tokens)
  → Total: 215 tokens (vs 2,000+ tokens every time with screenshot approach)
  
  CUMULATIVE SAVINGS OVER 100 VISITS:
  Visit 1:      1,500 tokens (learning)
  Visits 2-100: 20 tokens × 99 = 1,980 tokens
  Total:        3,480 tokens
  
  Screenshot approach: 2,000 tokens × 100 visits = 200,000 tokens
  
  SAVINGS: 94.3% reduction over 100 visits
  SAVINGS: 99%+ reduction after grammar is stable
```

---

## 8. THE ZERO-SCREENSHOT TASK MATRIX

Every common agent task mapped to the cheapest layer that can handle it:

```
TASK CATEGORY              LAYER    TOKEN COST    OLD COST    REDUCTION
─────────────────────────────────────────────────────────────────────────
Price lookup               L0       ~15 tokens    ~2,000      98.5%
Stock check                L0       ~10 tokens    ~2,000      99.5%
Page title extraction      L0       ~5 tokens     ~2,000      99.8%
Link finding               L0       ~20 tokens    ~2,000      99.0%
Form field location        L0/L1    ~15 tokens    ~2,000      99.3%
Login detection            L1       ~10 tokens    ~2,000      99.5%
Search submission          L1       ~25 tokens    ~2,000      98.8%
Pagination (page 2→N)      L1       ~10 tokens    ~2,000      99.5%
Feed monitoring            L3       ~50 tokens    ~2,000      97.5%
Price monitoring           L3       ~15 tokens    ~2,000      99.3%
Read article text          L2       ~200 tokens   ~2,000      90.0%
Find matching products     L2       ~300 tokens   ~3,000      90.0%
Fill complex form          L1/L2    ~100 tokens   ~2,500      96.0%
Analyze chart              L4       ~400 tokens   ~2,000      80.0%
Read scanned document      L4       ~600 tokens   ~2,500      76.0%
Verify visual layout       L4       ~300 tokens   ~2,000      85.0%
Debug unknown new site     L4       ~1,500 tokens ~2,000      25.0%
```

**The last row is the old default. It is now the exception.**

---

## 9. WHAT THIS REQUIRES FROM AGENTICVISION

The current `.avis` format stores visual captures and diffs. It needs a new section: **the Grammar Store**.

```
NEW .avis FILE SECTIONS (V2):
══════════════════════════════

  EXISTING (V1):
  ├── captures/         # Screenshot captures with metadata
  ├── diffs/            # Visual diff records
  ├── contexts/         # Multi-context visual states
  └── grounding/        # Element grounding coordinates

  NEW (V2 — Perception Revolution):
  ├── grammars/         # Site Grammar Records (the big addition)
  │   ├── {domain}.sgr  # Site Grammar Record per domain
  │   └── index.sgridx  # Grammar index (domain → file)
  ├── intents/          # Intent-scoped extraction results (cacheable)
  │   └── {hash}.ier    # Intent Extraction Result (keyed by url+intent+hash)
  ├── deltas/           # DOM structural diffs (not pixel diffs)
  │   └── {url_hash}.dsr # DOM Structural Record
  └── budgets/          # Token budget history (for analytics)
      └── {session}.tbr  # Token Budget Record
```

---

## 10. IMPLEMENTATION PRIORITY (IMMEDIATE)

This is not "V3 someday." Token waste is happening RIGHT NOW on every task an agent performs with AgenticVision.

```
SPRINT 1 (2 weeks): LAYER 0 — SEMANTIC DOM EXTRACTION
══════════════════════════════════════════════════════
□ Implement accessibility tree extraction (headless Chromium via CDP)
□ Implement CSS selector query engine against extracted tree
□ Implement typed field extraction (text, number, URL, boolean)
□ Add layer: L0 routing to vision_extract MCP tool
□ Test: 50 common tasks on 10 popular sites — zero screenshots taken

SPRINT 2 (2 weeks): LAYER 1 — SITE GRAMMAR SYSTEM
════════════════════════════════════════════════════
□ Implement Grammar Learning Pipeline (structural scan → LLM inference)
□ Implement Site Grammar Record (.sgr format)
□ Implement Grammar Store in .avis
□ Implement Grammar Drift Detection (structural hash comparison)
□ Implement Partial Re-learning on drift
□ Test: visit Twitter 100 times — only 1st visit has learning cost

SPRINT 3 (2 weeks): LAYER 2/3 — INTENT ROUTING + DELTA VISION
══════════════════════════════════════════════════════════════
□ Implement PerceptionIntent declaration (mandatory, typed)
□ Implement Intent → Layer routing engine
□ Implement DOM structural diff (not pixel diff)
□ Implement Delta Vision for monitoring use cases
□ Implement Token Budget enforcement with hard caps
□ Test: 100-visit monitoring task, verify token cost scales with changes

SPRINT 4 (2 weeks): COMMUNITY GRAMMAR + TOP SITES
════════════════════════════════════════════════════
□ Build canonical grammars for top 100 sites (ships with AgenticVision)
□ Implement grammar contribution protocol (agent → community pool)
□ Implement grammar verification scoring
□ Build grammar update pipeline (monthly canonical updates)
□ Test: fresh install, no visits needed — 100 sites already known

TOTAL: 8 weeks from "screenshots for everything" to "screenshots for nothing"
```

---

## 11. CODE BUDGET REALITY CHECK

```
CURRENT CODEBASE:
  agentic-vision core:      ~12K lines Rust
  agentic-vision-mcp:       ~4K lines Rust
  Python SDK:               ~2K lines Python
  Tests:                    ~6K lines

WHAT WE NEED TO ADD:
  CDP browser integration:  ~8K lines Rust   (headless Chrome protocol)
  A11y tree extractor:      ~4K lines Rust
  CSS selector engine:      ~3K lines Rust
  Site Grammar Record:      ~5K lines Rust   (data model + serialization)
  Grammar Learning Pipeline:~6K lines Rust
  Grammar Drift Detection:  ~3K lines Rust
  Grammar Store (.avis v2): ~4K lines Rust
  Intent routing engine:    ~5K lines Rust
  DOM structural diff:      ~6K lines Rust
  Token Budget enforcer:    ~2K lines Rust
  Scoped screenshot system: ~3K lines Rust
  Community Grammar layer:  ~4K lines Rust
  New MCP tools (L0-L3):    ~5K lines Rust
  New CLI commands:         ~3K lines Rust
  Tests for all above:      ~25K lines Rust

TOTAL NEW CODE:             ~86K lines Rust
TOTAL WITH EXISTING:        ~110K lines

Every line exists to ensure that NO AGENT EVER takes a 2,000-token
screenshot to find a price that could have been extracted for 15 tokens.
```

---

## 12. THE ESCALATION CONTRACT

The Perception Revolution establishes a sacred ordering. The agent MUST try cheaper layers before escalating:

```
ESCALATION CONTRACT:
══════════════════════

  Rule 1: NEVER take a screenshot when a DOM query can answer the question.
  Rule 2: NEVER run a full DOM scan when a grammar query can answer it.
  Rule 3: NEVER re-extract what was just extracted and cached.
  Rule 4: NEVER pay full-page cost when scoped extraction is sufficient.
  Rule 5: ALWAYS declare intent BEFORE choosing perception method.
  Rule 6: ALWAYS respect the token budget hard cap.
  Rule 7: A site visited before MUST use its stored grammar, not screenshots.
  Rule 8: A screenshot taken for layout MUST be scoped to the relevant region.
  Rule 9: Grammar drift MUST trigger partial re-learning, not full re-screenshot.
  Rule 10: Token cost MUST be logged for every perception call (audit trail).

  If any of these rules is violated:
  → Bug. Not optimization. Not acceptable.
  
  The test is simple:
  "What does this laptop cost on Amazon?"
  If the answer requires a screenshot: we failed.
  If the answer costs more than 20 tokens after the first visit: we failed.
  If the answer is wrong: we failed.
  Only if it's correct AND costs ≤20 tokens AND used no screenshots: we passed.
```

---

## 13. THE SACRED COVENANT

```
TO EVERY AGENT THAT USES AGENTICVISION:

  We promise that you will never pay 2,000 tokens to find a price.
  We promise that the second time you visit a site, it costs less.
  We promise that the hundredth time you visit a site, it's nearly free.
  We promise that every other agent's learning benefits you.
  We promise that your perception scales with task complexity, not page complexity.
  We promise that "every website is different" becomes "we know how each one is different."
  We promise that screenshots are reserved for things that are actually visual.
  We promise that a chart requires a screenshot. A price does not.
  We promise that you will never be asked to overpay to understand the web.
  
  We promise that in a world where every other tool screenshots first
  and asks questions later, you will query first and never screenshot
  if a query was sufficient.
  
  This is not a performance optimization. This is a covenant.
  The web should not be opaque to an intelligent agent.
  We are making it legible. Permanently. For free.
```

---

*Addendum: THE PERCEPTION REVOLUTION*
*Fixing the fundamental flaw that makes vision expensive and dumb.*
*Nothing matters if every task costs a screenshot.*
