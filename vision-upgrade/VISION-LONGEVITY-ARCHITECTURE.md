# VISION LONGEVITY ARCHITECTURE
## From V1 Captures to 20-Year Persistent Web Intelligence

> **Status:** Architectural Design Document
> **Date:** March 2026
> **Scope:** Bridge the gap between AgenticVision V1 (shipped) and the vision of persistent, compressing, improving web knowledge
> **Principle:** A site grammar learned in 2026 must still work in 2036. A visual capture from 2026 must be navigable in 2046. Web intelligence accumulated over 20 years must remain searchable, accurate, and useful — not a bloated archive of stale screenshots.

---

## 1. WHERE WE ARE: V1 Reality Check

AgenticVision V1 (v0.3.0) is a shipped, production system:

**What exists and works:**
- `.avis` binary format with visual capture and metadata
- Visual diff engine (pixel-level and region-level comparison)
- Multi-context support (capture multiple visual states)
- Grounding system (element location in visual space)
- 16 inventions shipped
- MCP tools for capture, compare, ground, track

**What's missing for true longevity:**
- No Site Grammar system (every visit pays full cost — see ADDENDUM-PERCEPTION-REVOLUTION.md)
- No grammar compression hierarchy (grammars grow stale and accumulate without pruning)
- No significance scoring for visual captures (every screenshot treated equally)
- No forgetting protocol (capture archive grows linearly, 95% never queried again)
- No schema versioning or migration engine for .avis format
- No grammar drift tracking over time (sites change — no historical record of how)
- No storage budget management
- No DOM structural snapshot system (only pixel captures)
- No intent-result cache (same query on same page repeated endlessly)

**The gap is not in the foundation — the capture and diff infrastructure is solid. The gap is in the TEMPORAL INTELLIGENCE layer: the system that makes visual knowledge accumulate, improve, and compress over time rather than growing into an expensive archive of mostly-useless screenshots.**

---

## 2. THE LONGEVITY PROBLEM (Quantified)

### 2.1 Storage Growth Without Intelligence

| Use Case | Captures/Day | Raw Size/Year | 5 Years | 20 Years |
|----------|-------------|---------------|---------|----------|
| Personal agent (light browsing) | 20 | ~200 MB | ~1 GB | ~4 GB |
| Developer agent (heavy browsing) | 80 | ~800 MB | ~4 GB | ~16 GB |
| Research agent (monitoring) | 200 | ~2 GB | ~10 GB | ~40 GB |
| Enterprise agent (multi-site) | 500 | ~5 GB | ~25 GB | ~100 GB |

But storage size is not the real problem. The real problem is **signal-to-noise death**:
- Year 1: 95% of screenshots were taken for one-time use, never queried again
- Year 5: 99% of raw screenshots are irrelevant to current tasks
- Year 10: The search index for 80,000 screenshots is slower than just taking a new screenshot
- Year 20: The agent spends more time searching its visual archive than browsing the web

The archive becomes a liability, not an asset.

### 2.2 Storage With Intelligent Compression

| Use Case | Year 1 (Full) | Years 2-5 (Grammar+Episodes) | Years 6-10 (Summaries) | Years 11-20 (Patterns) | Total |
|----------|--------------|------------------------------|------------------------|------------------------|-------|
| Personal | 200 MB | 50 MB | 20 MB | 8 MB | ~278 MB |
| Developer | 800 MB | 200 MB | 80 MB | 30 MB | ~1.1 GB |
| Research | 2 GB | 500 MB | 200 MB | 75 MB | ~2.8 GB |
| Enterprise | 5 GB | 1.2 GB | 500 MB | 180 MB | ~6.9 GB |

The compression hierarchy doesn't just save storage. It makes visual knowledge SMARTER over time:
- Raw captures compress into behavioral patterns
- Patterns become grammars
- Grammars improve with use
- Old screenshots are replaced by grammar knowledge that weighs kilobytes

### 2.3 The Real Threats (Vision-Specific)

1. **Grammar staleness**: A site grammar from 2026 may be wrong by 2027 if the site redesigned. Without drift tracking, the agent silently fails on sites it thinks it knows.

2. **Screenshot format rot**: PNG/WebP captures from 2026 are fine. But metadata, grounding coordinates, and element references become meaningless when the site changes its DOM. A screenshot with no valid grammar reference is just a flat image.

3. **Intent-result cache explosion**: If we cache "what is the price on amazon.com/product/X at 2026-03-06T14:23:11Z" — 365 days × 100 products = 36,500 cache entries per year per user, most never queried again.

4. **Grammar version conflict**: A grammar learned by an agent using Chrome 124 may not match behavior seen by an agent using Chrome 130. Without versioning, grammar corruption is silent.

5. **Embedding drift for visual search**: If AgenticVision uses visual embeddings for "find similar pages" — the model used in 2026 won't exist or won't be compatible in 2031.

---

## 3. ARCHITECTURAL DESIGN: THE VISION LONGEVITY ENGINE

### 3.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                  AGENTICVISION LONGEVITY ENGINE                   │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                    V1 ENGINE (EXISTS)                      │   │
│  │  .avis Format · Visual Diff · Grounding · MCP Tools      │   │
│  └────────────────────────┬─────────────────────────────────┘   │
│                           │                                      │
│  ┌────────────────────────▼─────────────────────────────────┐   │
│  │              PERCEPTION LAYER (NEW - V2)                  │   │
│  │  (see ADDENDUM-PERCEPTION-REVOLUTION.md)                 │   │
│  │                                                           │   │
│  │  Adaptive Perception Stack · Site Grammar System          │   │
│  │  Intent-Scoped Extraction · Delta Vision                 │   │
│  └────────────────────────┬─────────────────────────────────┘   │
│                           │                                      │
│  ┌────────────────────────▼─────────────────────────────────┐   │
│  │              LONGEVITY BRIDGE (NEW - V3)                  │   │
│  │                                                           │   │
│  │  ┌─────────────┐  ┌──────────────┐  ┌────────────────┐  │   │
│  │  │ Grammar     │  │ Visual       │  │ Schema         │  │   │
│  │  │ Compression │  │ Archive Mgr  │  │ Versioning     │  │   │
│  │  └──────┬──────┘  └──────┬───────┘  └───────┬────────┘  │   │
│  │         │                │                   │            │   │
│  │  ┌──────▼──────┐  ┌─────▼────────┐  ┌──────▼─────────┐  │   │
│  │  │ Drift       │  │ Significance │  │ Intent Cache   │  │   │
│  │  │ History     │  │ Scorer       │  │ Management     │  │   │
│  │  └──────┬──────┘  └──────┬───────┘  └───────┬────────┘  │   │
│  │         │                │                   │            │   │
│  │  ┌──────▼──────┐  ┌─────▼────────┐  ┌──────▼─────────┐  │   │
│  │  │ Grammar     │  │ Forgetting   │  │ Storage        │  │   │
│  │  │ Evolution   │  │ Protocol     │  │ Budget         │  │   │
│  │  └─────────────┘  └──────────────┘  └────────────────┘  │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                           │                                      │
│  ┌────────────────────────▼─────────────────────────────────┐   │
│  │              PERSISTENCE LAYER (NEW - V3)                 │   │
│  │                                                           │   │
│  │  ┌─────────────────────────────────────────────────────┐ │   │
│  │  │  SQLite Backing Store (long-term structured data)    │ │   │
│  │  │  ├── site_grammars (versioned, drift-tracked)        │ │   │
│  │  │  ├── grammar_history (how each site changed over time│ │   │
│  │  │  ├── schema_versions (format migration chain)        │ │   │
│  │  │  ├── intent_cache (deduped extraction results)       │ │   │
│  │  │  ├── visual_significance (importance scoring)        │ │   │
│  │  │  └── compression_log (what was archived when)        │ │   │
│  │  └─────────────────────────────────────────────────────┘ │   │
│  │                                                           │   │
│  │  ┌─────────────────────────────────────────────────────┐ │   │
│  │  │  .avis File (hot path, binary, active grammars)      │ │   │
│  │  │  V2 format with grammar store + delta snapshots      │ │   │
│  │  └─────────────────────────────────────────────────────┘ │   │
│  │                                                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### 3.2 The Dual-Store Architecture Decision

**Critical design choice: `.avis` binary + SQLite, not one or the other.**

The `.avis` file is the HOT PATH — active grammars for currently-used sites, recent captures for active monitoring tasks, current intent cache. Sub-millisecond grammar lookups during active browsing sessions.

SQLite becomes the COLD PATH — grammar history (how Amazon's DOM evolved 2026-2046), compressed visual archives, schema migration records, significance scoring data, everything that needs to survive decades.

```
~/.agentic/vision/
├── {workspace}.avis          # Hot: active grammars, recent captures, intent cache
└── {workspace}.vision.db     # Cold: grammar history, archive, schemas, significance
```

The `.avis` file is the operational layer. The `.vision.db` is the memory. If `.avis` is lost, grammars can be reconstructed from `.vision.db` and from community grammar pool. If `.vision.db` is lost, `.avis` preserves current operations while the daemon rebuilds history.

---

## 4. THE VISUAL KNOWLEDGE HIERARCHY

Visual knowledge has natural compression levels. Raw screenshots at the bottom. Web intelligence at the top.

```
VISUAL KNOWLEDGE HIERARCHY:
════════════════════════════

Level 0: RAW CAPTURES
══════════════════════
  What: Full-page screenshots, scoped element screenshots
  Size: 200KB-2MB per capture
  Retention: Active session + 30 days
  Usage: Direct visual analysis, layout verification, visual debugging
  Query: By URL, timestamp, similarity
  Transition: After 30 days → compress to DOM Snapshot

Level 1: DOM SNAPSHOTS
═══════════════════════
  What: Structural snapshot of page (nodes, roles, text, selectors, coordinates)
       No pixel data. Pure structural map.
  Size: 2KB-20KB per snapshot (100× compression vs screenshot)
  Retention: 1 year after last visit
  Usage: Grammar learning, drift detection, structural diff
  Query: By URL, timestamp, structural similarity
  Transition: If site visited regularly → grammar extraction
              If site not revisited → after 1 year → archive summary

Level 2: SITE GRAMMARS
═══════════════════════
  What: Typed behavioral model of site (selectors, patterns, state indicators)
       Everything needed to operate on the site without any screenshot
  Size: 1KB-10KB per site
  Retention: PERMANENT (with versioning)
  Usage: Zero-token page interaction, intent routing, direct query
  Query: By domain, capability, grammar version
  Transition: Never expires. Versions accumulate. Drift patches applied.

Level 3: INTENT RESULTS (Cache)
════════════════════════════════
  What: The result of a specific intent on a specific page version
       "Price of product X on 2026-03-06" = "$1,299"
  Size: 50-500 bytes per result
  Retention: Until page structural hash changes (result becomes invalid)
  Usage: Zero-cost repeat queries (same URL, same page version, same intent)
  Query: By URL + intent + structural hash
  Transition: Invalidated when page changes (not time-based)

Level 4: SITE EVOLUTION RECORDS
═════════════════════════════════
  What: Historical record of how a site's grammar has changed over time
       "Amazon changed its price selector from .a-price to .a-price-whole
        in Q3 2027. Navigation changed in Q1 2029."
  Size: 5KB-50KB per site (full history)
  Retention: PERMANENT (this is web archaeology)
  Usage: Understanding site behavior patterns, grammar migration, debugging
  Query: By domain, date range, grammar version, drift event
  Transition: Never expires. New drift events are appended.

Level 5: WEB INTELLIGENCE PATTERNS
════════════════════════════════════
  What: Cross-site behavioral patterns distilled from thousands of grammars
       "E-commerce sites universally use: .price, [itemprop=price], or .a-price"
       "SPA navigations always use history.pushState"
       "Rate limiting always shows [class*=error] or HTTP 429"
  Size: 1KB-5KB per pattern
  Retention: PERMANENT (living knowledge)
  Usage: Cold-start on unknown sites (use patterns to guess initial selectors)
  Query: By site category, behavior type, pattern confidence
  Transition: Aggregated from Level 2 grammars. Updated monthly.
```

---

## 5. GRAMMAR COMPRESSION AND EVOLUTION

### 5.1 Grammar Lifecycle

```
GRAMMAR LIFECYCLE:
══════════════════

  STATE 1: LEARNING
  ──────────────────
  Site first visited → Grammar Learning Pipeline runs
  Selectors extracted → LLM classifies elements
  Intent routes mapped → Grammar Record created
  Status: "learning" — tentative, needs verification
  
  STATE 2: ACTIVE
  ────────────────
  Grammar used for successful extraction → confidence increases
  Each successful query: selector confidence += 0.1
  Each failed query: selector confidence -= 0.2
  Status: "active" when average confidence > 0.8
  
  STATE 3: DRIFTED
  ─────────────────
  Structural hash mismatch detected on visit
  Failed queries accumulate
  Status: "drifted" — partial re-learning triggered
  Only changed sections re-learned (not full re-learn)
  
  STATE 4: ARCHIVED
  ──────────────────
  Site not visited in 1+ years AND no community activity
  Grammar moved to cold storage (.vision.db)
  Still usable (loaded on demand) but not in hot .avis
  Status: "archived"
  
  STATE 5: HISTORICAL
  ────────────────────
  Grammar superseded by new version due to site redesign
  Old grammar version preserved as Site Evolution Record
  Historical value: understand how the web changed
  Status: "historical" — readable but never executed

  NEVER: Grammar deleted unless explicitly purged
  ALWAYS: Version chain preserved indefinitely
```

### 5.2 Grammar Significance Scoring

Not all grammars are equally valuable. The significance scorer determines retention priority, replication to community pool, and compression scheduling.

```
GRAMMAR SIGNIFICANCE FACTORS:
══════════════════════════════

  1. USAGE FREQUENCY (weight: 0.35)
     How many times this grammar has been used
     Normalized across all grammars
     
  2. SUCCESS RATE (weight: 0.25)
     Percentage of queries answered without fallback to screenshot
     High-quality grammar: > 95% success rate
     
  3. SITE IMPORTANCE (weight: 0.20)
     Alexa rank / domain authority proxy
     High-traffic sites worth preserving even if rarely queried
     
  4. UNIQUENESS (weight: 0.10)
     How different from Web Intelligence Patterns
     Generic e-commerce grammar: low uniqueness
     Niche specialized site: high uniqueness
     
  5. RECENCY (weight: 0.10)
     Last verified against live site
     Recently verified → higher score

  SIGNIFICANCE SCORE: 0.0 (noise) → 1.0 (critical)
  
  THRESHOLD ACTIONS:
  > 0.7: Active tier, replicated to community pool
  0.4-0.7: Standard tier, kept in .avis hot path
  0.2-0.4: Cold tier, moved to .vision.db SQLite
  < 0.2: Candidate for archival (user can override)
  
  USER OVERRIDE: Always available.
  Any grammar can be pinned as permanent regardless of score.
  ("I need this niche internal tool's grammar forever.")
```

---

## 6. THE FORGETTING PROTOCOL (VISUAL)

AgenticVision must know what to forget. Without forgetting, the archive grows forever and degrades in quality.

```
VISUAL FORGETTING PROTOCOL:
════════════════════════════

  WHAT GETS FORGOTTEN (by layer):
  
  Raw Captures:
  ──────────────
  • After 30 days: Convert to DOM Snapshot, delete pixel data
    Exception: User explicitly marked capture for retention
    Exception: Capture is linked to a significant agent decision
  • After 30 days with no DOM Snapshot possible: archive as image-only
  
  DOM Snapshots:
  ───────────────
  • After 1 year + site not revisited: archive to .vision.db
  • If grammar extracted from snapshot: raw snapshot can be deleted
    (grammar IS the compressed version of the snapshot)
  
  Intent Cache:
  ──────────────
  • Invalidated: when page structural hash changes
  • Time-limited: max 24-hour cache for dynamic content (prices, stock)
  • Preserved: for stable content (article text, documentation)
  
  Site Grammars:
  ───────────────
  • NEVER forgotten. Versioned. Archived. But never deleted.
  • Historical versions are compressed to minimal delta format.
    "V2 changed: product_price selector from X to Y. Everything else identical."
    (50 bytes instead of 10KB full grammar)
  
  WHAT IS NEVER FORGOTTEN:
  ──────────────────────────
  • Grammar evolution records (how the web changed)
  • Web Intelligence Patterns (cross-site behavioral knowledge)
  • User-pinned captures (explicit preservation)
  • Captures linked to significant agent decisions (evidence chain)
  • Site grammars in any form (the accumulated intelligence)
```

---

## 7. SCHEMA VERSIONING AND THE 20-YEAR PROMISE

The `.avis` format must outlive every browser version, every CSS framework trend, every major site redesign it has ever encountered.

```
SCHEMA VERSION CHAIN:
══════════════════════

  .avis v1 (shipped): captures, diffs, contexts, grounding
  .avis v2 (V2):      + grammars, intents, deltas, budgets
  .avis v3 (V3):      + longevity fields, significance scores,
                         compression metadata, schema version active
  .avis vN (future):  each version adds fields, never removes
  
  MIGRATION RULES (non-negotiable):
  ───────────────────────────────────
  1. Every .avis version can be read by all later versions
  2. Migration is automatic (never asks user)
  3. Original file backed up before migration
  4. Migration is reversible to N-1 version
  5. CI carries test files from every version ever shipped
     (the "forever test" — same as AgenticMemory)
  
  THE FOREVER TEST:
  ──────────────────
  A test file captured in 2026 MUST be:
  • Readable by AgenticVision in 2036 ✓
  • Fully queryable (grammars still accessible) ✓
  • Migratable to latest format automatically ✓
  • Grammar evolution record preserved ✓
  
  This test runs in CI forever.
  If it ever fails, we have broken the covenant.
```

### 7.1 Grammar Version Compatibility

Site grammars have their own versioning problem. A grammar written using 2026 CSS selector patterns must still be parseable in 2036, even if:
- CSS syntax evolved
- New structural patterns emerged (Web Components, shadow DOM expansion)
- The site itself changed (tracked via drift history)

```
GRAMMAR FORMAT VERSIONING:
════════════════════════════

  grammar_format: "sgr-v1"    // Selector grammar record, version 1
  
  Each grammar field includes:
  ├── selector: string         // The actual selector
  ├── selector_type: enum      // css | xpath | aria | text | shadow-dom
  ├── format_version: "sgr-v1" // Which format generated this selector
  ├── confidence: f32          // 0.0-1.0
  ├── verified_at: timestamp
  └── fallback_selectors: Vec<string>  // If primary fails, try these
  
  If primary selector fails AND all fallbacks fail:
  → Grammar for this field is marked "drifted"
  → Partial re-learning triggered for this field only
  → Old selector preserved as historical record
  → New selector added as primary
```

---

## 8. THE INTENT CACHE (ELIMINATE REDUNDANT PERCEPTION)

The Vision equivalent of AgenticMemory's Transport Capture — stop paying for what you already know.

```
INTENT CACHE ARCHITECTURE:
════════════════════════════

  CACHE KEY:
  ───────────
  {url_normalized} + {intent_type} + {structural_hash} + {content_hash}
  
  Structural hash:  hash of DOM structure (detects site redesign)
  Content hash:     hash of relevant content region (detects content change)
  
  CACHE HIT: If all four match → return cached result, 0 tokens
  CACHE MISS: If content hash changed → re-extract, update cache
  CACHE INVALID: If structural hash changed → grammar drift, re-learn
  
  EXAMPLE:
  ─────────
  Agent asks: "What is the price of MacBook Pro 16 on amazon.com?"
  
  Session 1 (10:00 AM):
    URL: amazon.com/dp/B09G9HD6PD
    Intent: find_price
    Structural hash: "a4f8..." (DOM structure)
    Content hash: "c2e1..." (price region content)
    → Cache miss → extract → "$1,299" → cache
    
  Session 2 (10:05 AM, same session or new session same day):
    Same URL, same structural hash, same content hash
    → Cache HIT → "$1,299" immediately → 0 tokens
    
  Session 3 (next day, price changed):
    Same URL, same structural hash, DIFFERENT content hash
    → Partial cache miss → re-extract price only → "$1,249" → update cache
    → Tokens used: ~15 (only re-extracted price region)
    
  Session 4 (after Amazon redesign):
    Same URL, DIFFERENT structural hash
    → Full cache miss → grammar drift → partial re-learn
    → Tokens used: ~300 (grammar update + extraction)
  
  CACHE RETENTION:
  ─────────────────
  Dynamic content (prices, stock, news): 1-hour TTL
  Semi-static content (reviews, descriptions): 24-hour TTL
  Static content (documentation, articles): 7-day TTL
  User-override: pin any result for custom duration
```

---

## 9. STORAGE BUDGET MANAGEMENT

```
DEFAULT STORAGE BUDGET:
════════════════════════

  Profile: Developer Agent (default)
  ────────────────────────────────────
  Hot .avis (active grammars + recent):    200 MB  (hard limit)
  Cold .vision.db (history + archive):     2 GB    (soft limit)
  Intent cache:                            50 MB   (circular buffer)
  Total budget:                            ~2.5 GB default
  
  LAYER ALLOCATIONS (within hot .avis):
  ───────────────────────────────────────
  Active grammars:    40% (frequently used sites)
  Recent captures:    30% (last 30 days, raw pixels)
  DOM snapshots:      20% (structural history)
  Intent cache:       10% (query result cache)
  
  GROWTH PROJECTION:
  ───────────────────
  Year 1:  ~300 MB total (learning phase, many grammars being built)
  Year 2:  ~400 MB (grammars stable, captures compressing)
  Year 5:  ~600 MB (mature archive, efficient compression)
  Year 10: ~800 MB (grammar evolution records accumulate)
  Year 20: ~1.2 GB (20 years of web archaeology, well within 2.5 GB)
  
  ALERTS:
  ────────
  80% of any layer → Warning (accelerate compression)
  95% of any layer → Critical (emergency forgetting pass)
  95% total → User notification (expand budget or review)
```

---

## 10. SITE EVOLUTION ARCHAEOLOGY (THE UNIQUE VALUE)

This is the capability that makes 20-year Vision longevity genuinely valuable beyond just "the grammars still work."

After 20 years, AgenticVision has captured how the web itself has changed:

```
WHAT 20 YEARS OF VISION LONGEVITY CONTAINS:
═════════════════════════════════════════════

  "How did Amazon's product page change between 2026 and 2046?"
  → Complete grammar version history
  → Every CSS selector change, timestamped
  → Every UI pattern that appeared or disappeared
  → Which selector patterns proved durable vs fragile
  
  "How did web accessibility evolve?"
  → ARIA adoption rates across 10,000 sites over 20 years
  → Which interactive patterns became universal standards
  → How screen-reader compatibility changed
  
  "When did SPAs stop being the dominant pattern?"
  → navigation_type field across all grammars, over time
  → Transition from spa_navigation: true → false
  → The rise of server-side rendering patterns
  
  "Which e-commerce sites survived?"
  → Grammars exist for sites that may no longer exist
  → Site Evolution Records document their last-known state
  → Competitive intelligence frozen in time
  
  This is WEB ARCHAEOLOGY. No other system captures it.
  
  The grammars are not just a performance optimization.
  They are a historical record of the web's evolution.
  That record belongs to the user. Permanently.
```

---

## 11. INTEGRATION WITH SISTERS

### AgenticMemory
When an agent makes a decision based on visual perception ("I chose ThinkPad because its Amazon price was $200 less than Dell"), that decision event stored in AgenticMemory LINKS to the Vision capture that informed it. Twenty years later, the decision chain is navigable: `decision → memory_event → vision_intent_cache → grammar_version → DOM_snapshot`.

### AgenticCodebase
When an agent analyzes a GitHub repository through Vision, the site grammar for GitHub is shared. Code-related intent routes in the GitHub grammar tie directly to Codebase's understanding of repository structure. The grammars inform each other.

### Evolve (Astral Sister)
The Collective Grammar pool lives in Evolve's Crystallization system. AgenticVision contributes grammars. Evolve verifies, canonicalizes, and distributes them. The Web Intelligence Patterns (Level 5 of the knowledge hierarchy) are computed by Evolve from thousands of Vision-contributed grammars. This is the collective learning flywheel.

### Hydra
Hydra uses Vision's grammar system as a browser automation primitive. When Hydra needs to interact with a website, it calls Vision's grammar layer first. If grammar exists: zero-token operation. If not: Vision's learning pipeline runs, grammar is stored, future Hydra calls are free. Hydra never pays the screenshot tax twice for the same site.

---

## 12. NEW MCP TOOLS (V2 + V3)

```
PERCEPTION TOOLS (V2):
  vision_dom_extract     ← Layer 0: DOM query without screenshot
  vision_grammar_query   ← Layer 1: Grammar-based extraction
  vision_intent_extract  ← Layer 2: Intent-scoped extraction (routes automatically)
  vision_delta_perceive  ← Layer 3: DOM diff-based change detection
  vision_scope_capture   ← Layer 4: Scoped screenshot (element bounds only)

GRAMMAR TOOLS (V2):
  vision_grammar_learn   ← Learn grammar for new site
  vision_grammar_get     ← Get grammar for known site
  vision_grammar_status  ← Check grammar confidence and drift status
  vision_grammar_update  ← Force partial or full re-learn
  vision_grammar_pin     ← Pin grammar version permanently

LONGEVITY TOOLS (V3):
  vision_longevity_stats     ← Storage budget, layer distribution, projections
  vision_longevity_project   ← Project storage needs for N years
  vision_longevity_health    ← Overall health score and recommendations
  vision_archive_search      ← Search visual history (all layers)
  vision_evolution_query     ← Query how a site has changed over time

CACHE TOOLS (V3):
  vision_cache_get       ← Retrieve intent cache result
  vision_cache_status    ← Cache hit rates, age, size
  vision_cache_invalidate ← Force re-extraction for URL
  
SIGNIFICANCE TOOLS (V3):
  vision_significance_get   ← Significance score for grammar/capture
  vision_significance_set   ← Override score, pin retention
  vision_compress_now       ← Trigger manual compression pass
```

---

## 13. IMPLEMENTATION PHASES

### Phase 1: Perception Foundation (V2.0) — The Adaptive Stack
**See ADDENDUM-PERCEPTION-REVOLUTION.md for full sprint plan.**
- CDP browser integration
- Accessibility tree extraction
- Site Grammar system
- Intent-scoped routing
- Token budget enforcement
- Ships as AgenticVision 0.4.0

**Success criteria:** 95% of common browsing tasks use no screenshots. Grammar system live for top 100 sites. Token costs logged per call.

### Phase 2: Longevity Foundation (V2.1) — The Dual Store
**Goal:** Introduce SQLite backing without breaking V2.

- Add `.vision.db` alongside `.avis`
- Nightly sync: grammar history, drift events, captures → SQLite
- Implement significance scorer (usage frequency + success rate only)
- Add grammar versioning infrastructure
- Add schema version field to `.avis`
- Ships as AgenticVision 0.5.0

**Success criteria:** All V2 tests pass. Grammar history accumulates. No user-facing behavior changes. New `vision longevity-stats` CLI command works.

### Phase 3: Compression (V2.2) — The Hierarchy
**Goal:** Implement visual knowledge compression.

- Implement Raw → DOM Snapshot compression (30-day schedule)
- Implement DOM Snapshot → Grammar extraction pipeline
- Implement Intent Cache with invalidation
- Implement Grammar significance scorer (full model)
- Implement storage budget management
- Ships as AgenticVision 0.6.0

**Success criteria:** After 30 days of use, raw captures compressed to DOM snapshots. Storage growth rate drops 80%+ vs V1 screenshot approach. Intent cache hit rate > 70% for repeated tasks.

### Phase 4: Survival (V2.3) — The Guarantees
**Goal:** Make the 20-year promise real.

- Implement .avis format versioning (migration chain active)
- Implement Visual Intelligence Pattern extraction
- Implement Site Evolution Record preservation
- Implement storage budget projection
- Implement safe forgetting protocol
- Carry forward V1 test files in CI ("forever test")
- Ships as AgenticVision 0.7.0

**Success criteria:** Can migrate V1 `.avis` to V4 format automatically. CI has test files from every previous format version. 20-year storage projection within 2.5 GB budget for developer use case.

### Phase 5: Intelligence (V2.4) — Web Archaeology
**Goal:** Visual knowledge that improves with time.

- Implement Web Intelligence Patterns (cross-site behavioral distillation)
- Implement Site Evolution Archaeology queries
- Implement Collective Grammar integration with Evolve sister
- Implement predictive grammar loading (pre-load grammars agent will likely need)
- Ships as AgenticVision 0.8.0

**This is where Vision stops being a perception tool and becomes a web intelligence system.**

---

## 14. WHAT STAYS PRIVATE (Hydra Integration Points)

The longevity engine is OPEN SOURCE. It ships with AgenticVision. But the following capabilities are reserved for Hydra and remain proprietary:

- **Omniscience Loop**: Cross-sister visual queries (Vision grammar + Memory decision + Codebase context in one query)
- **Collective Grammar Pool**: Full Evolve-backed community grammar network (open contribution, but Agentra Labs runs the canonical verification tier)
- **Grammar Prophecy**: Hydra's ability to predict which grammars will be needed for upcoming tasks and pre-load them (requires Planning + Time sisters)
- **Web Intelligence API**: Cross-user, anonymized web pattern intelligence sold as enterprise data product

The open-source longevity engine gives every AgenticVision user 20-year visual memory. The proprietary Hydra layer gives Agentra Labs customers visual intelligence that spans all agents.

---

## 15. RISK REGISTER

| Risk | Severity | Mitigation |
|------|----------|------------|
| Grammar confidently wrong (site changed silently) | High | Structural hash on every visit. Silent failures trigger drift detection. Query success rate monitoring. |
| CDP / browser engine changes | High | Abstract browser interface. Support CDP, Playwright protocol, and direct a11y API. Never lock to one browser engine. |
| Shadow DOM and Web Components break selectors | Medium | selector_type field supports shadow-dom traversal. Grammar Learning Pipeline handles shadow roots explicitly. |
| Intent cache returns stale data | Medium | Content hash + TTL double-protection. Dynamic content has 1-hour max TTL regardless. |
| Visual embedding model discontinued | Medium | Visual search uses grammar text features first. Pixel embeddings optional. Lazy re-embedding strategy. |
| SQLite grammar database corrupts | Low | WAL mode. Periodic backup. Grammar pool can reconstruct from community. |
| 20-year CSS selector patterns become obsolete | Low | selector_type enum extensible. Historical selectors preserved in grammar evolution record for archaeology value even if non-executable. |

---

## 16. SUCCESS METRICS

```
YEAR 1:
  □ 95%+ of common tasks: zero screenshots
  □ Grammar coverage: top 1,000 sites have canonical grammars
  □ Storage growth rate: < 20% of V1 screenshot approach
  □ Intent cache hit rate: > 70% for power users
  □ Zero data loss across 10,000 grammar update cycles

YEAR 5:
  □ Grammar coverage: top 100,000 sites covered (community + agent-learned)
  □ Storage within 30% of budget projection
  □ Site Evolution Records capture 5 years of web change history
  □ Grammar success rate: > 98% for canonical grammars
  □ At least one major site redesign successfully tracked and grammar updated

YEAR 20:
  □ All grammars from Year 1 still accessible (historical tier)
  □ Site Evolution Records cover 20 years of web archaeology
  □ Total storage: < 2.5 GB for developer use case
  □ Grammar query time: < 10ms at any age, any layer
  □ The V1 test file from 2026 still opens perfectly
```

---

*Document: VISION-LONGEVITY-ARCHITECTURE.md*
*The vision that never goes blind. The web intelligence that compounds.*
*Every site visited makes the next visit cheaper. That's the promise.*
