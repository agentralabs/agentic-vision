# Capability Scenarios: AgenticVision

*What happens when an AI agent can see, remember what it saw, and reason about visual change over time?*

---

## Persistent Visual Evidence Store — The .avis Format

You're debugging a CSS layout issue that only appears when a user has more than 5 items in their shopping cart. You reproduce it, describe it to the agent, and the agent says "I understand." But the agent doesn't actually see the problem. It has your text description, which omits the subtle 3-pixel overlap between the cart badge and the notification icon. You spend 10 minutes describing a visual problem in words, and the fix the agent proposes addresses the wrong element.

Without persistent visual storage, every screenshot the agent takes is ephemeral. It exists for one conversation turn, then vanishes. The next session, the agent has no visual history. You can't say "remember that layout bug from Tuesday?" because the agent has no visual memory. Every visual debugging session starts from scratch. Evidence is lost the moment the conversation ends.

With the `.avis` binary format, every captured screenshot becomes a permanent visual observation: a JPEG thumbnail (max 512x512, quality 85), a 512-dimensional CLIP ViT-B/32 embedding vector for semantic search, a quality score, metadata labels, and a timestamp — all packed into roughly 4.26 KB per capture. A gigabyte holds 250,000 visual observations. The agent can reference visual evidence from any prior session, compare current state against historical baselines, and build a visual timeline of how your UI evolved. The file is portable, requires no external database, and opens in under 5 milliseconds.

> Loaded visual memory: 847 observations across 23 sessions. Oldest capture: January 15th (login page redesign). Most recent: today's cart badge regression. Storage: 3.6 MB.

**In plain terms:** The `.avis` format gives the agent a photo album with perfect recall. Instead of forgetting every screenshot between conversations, it keeps a searchable, timestamped visual history — like security camera footage for your UI.

---

## vision_query — Retrieving Past Visual States

You're reviewing a design change your team made to the dashboard two weeks ago. You want to see what the dashboard looked like before the change, but nobody took screenshots. The Git history shows CSS modifications, but you can't visualize the before state from a diff of `margin-top: 16px` to `margin-top: 24px`.

Without visual querying, the past is opaque. You have code diffs but no visual diffs. You might revert the CSS change locally to see what it looked like, but that disrupts your current work and doesn't account for other changes that happened simultaneously. The visual state of "two weeks ago" is unrecoverable.

With `vision_query`, the agent filters its visual evidence store by time range, labels, quality score, and description. You ask "show me the dashboard from two weeks ago" and the agent queries `after: 1708300800, before: 1709510400, labels: ["dashboard"]` and returns 12 captures sorted by recency. The highest-quality capture (score 0.82 — good resolution, labeled, described) shows the dashboard with the old 16px margins. You now have a visual reference without reverting any code.

> Found 12 dashboard captures from February 1-14. Best quality capture (0.82): session 15, February 8th, labeled "dashboard main view," 1280x720 resolution. Showing the pre-redesign layout with the compact header and 16px card margins.

**In plain terms:** vision_query is a search engine for your agent's visual memory. Instead of hoping someone took a screenshot, the agent retrieves exactly the visual state you need — filtered by time, quality, and labels — like searching a photo library by date and tag.

---

## vision_capture — Structured Screenshot with Typed Extraction

Your agent is monitoring a long-running deployment. The deployment dashboard shows progress bars, status indicators, log streams, and timing information. You step away for 30 minutes. When you return, you ask "what happened during the deployment?"

Without structured capture, the agent either describes what it sees in text (lossy, missing visual details like color-coded status indicators) or takes a raw screenshot that's discarded after the conversation. The deployment log scrolled past critical error messages that appeared for 10 seconds and then were replaced by subsequent output. The visual evidence is gone.

With `vision_capture`, the agent takes a structured screenshot from any of four sources — file, base64, live screenshot (full screen or region), or clipboard — and processes it through a rigorous pipeline. The image gets a JPEG thumbnail for storage efficiency. A CLIP ViT-B/32 neural network generates a 512-dimensional semantic embedding in 47 milliseconds. Quality scoring evaluates resolution (35% weight), metadata completeness (40%), and model confidence (25%). Metadata is sanitized: email addresses become `[redacted-email]`, API keys become `[redacted-secret]`, filesystem paths become `[redacted-path]`.

During your 30-minute absence, the agent captured 4 snapshots: deployment start (all green), a warning state at minute 12 (yellow status bar), an error at minute 18 (red log entry about failed health check), and recovery at minute 23 (green again). Each capture has labels, a description, and a quality score. The error capture has quality 0.78 — good resolution, labeled "deployment-error," described as "Health check failure on pod-3."

> Captured 4 deployment states while you were away. Key event: health check failure on pod-3 at minute 18 (capture #1247, quality 0.78). System recovered automatically at minute 23. Full visual timeline available.

**In plain terms:** vision_capture is a structured witness. It doesn't just take a photo — it indexes it, scores its quality, strips sensitive data, and files it where you can find it later. Your agent builds a visual evidence chain, not a pile of screenshots.

---

## UI-State Blindness Coverage — Seeing What the Model Can't Describe

Your e-commerce checkout page has a subtle visual bug. The "Place Order" button's disabled state looks almost identical to its enabled state — the background color shifts from `#2563eb` to `#93c5fd`, a difference that's visible to humans but nearly impossible for a language model to distinguish from a text description alone. The model says "I see a blue button" in both cases.

Without visual evidence storage, the agent relies entirely on its text-based understanding of the page. It reads the DOM: the button has `disabled="true"`. But it can't see that the visual feedback is insufficient — the color contrast ratio between enabled and disabled is only 1.3:1, well below WCAG guidelines. The accessibility problem is invisible to text-based reasoning.

With vision_capture, the agent takes a screenshot of the button in both states and stores them as observations with quality scores. Even though the agent can't articulate the exact color difference in text, the CLIP embedding captures the visual distinction — the two states produce embeddings with 0.94 cosine similarity (nearly identical, confirming the problem). A `vision_compare` between the enabled and disabled button captures confirms: `similarity: 0.94, is_same: false` — visually almost identical. The agent can now reason about the visual evidence numerically, even when it can't describe the colors precisely.

> Captured enabled and disabled button states. Visual similarity: 0.94 — these states are nearly indistinguishable visually. The disabled state lacks sufficient visual contrast. This may be an accessibility issue (color contrast ratio likely below WCAG 2.1 AA threshold of 3:1 for UI components).

**In plain terms:** Vision capture covers the gap between what the model can describe in words and what actually appears on screen. When a UI problem is visual rather than structural, the agent has photographic evidence to reason about — even when it can't put the colors into words.

---

## vision_compare — Side-by-Side State Comparison

You've just merged a CSS refactoring PR. The PR description says "no visual changes — only reorganized selectors." You want to verify that claim before deploying to production.

Without visual comparison, you trust the PR description. You might manually check a few pages, but with 47 unique routes in your application, comprehensive visual regression testing by eye is impractical. A subtle 2-pixel shift in the navigation bar, caused by a specificity change in the reorganized selectors, goes unnoticed until a user reports it.

With `vision_compare`, the agent captures the current state of key pages and compares them against pre-merge captures from its visual memory. For each page, it computes cosine similarity on the 512-dimensional CLIP embeddings. The home page: 0.99 similarity (identical). The product page: 0.98 (identical). The checkout page: 0.97 (identical). The user profile page: 0.89 (changed). The agent flags the profile page immediately — 0.89 is well below the 0.95 "is_same" threshold.

> Compared 12 pages pre-merge vs post-merge. 11 pages visually identical (similarity > 0.95). 1 regression detected: user profile page (similarity 0.89). The profile page header has shifted — likely a CSS specificity change affecting `.profile-header` margins.

**In plain terms:** vision_compare is an automated before-and-after check. Instead of trusting that a CSS refactor is purely cosmetic, the agent verifies it by comparing visual snapshots — like overlaying two photos to find what moved.

---

## vision_diff — Detecting What Changed Between Captures

The profile page regression from the CSS refactor has been flagged. Now you need to know exactly *where* on the page the change occurred. The page is complex — a header, avatar, bio section, activity feed, and settings sidebar. Which region changed?

Without pixel-level diff, you're doing visual spot-the-difference by eye on a 1280x720 page. The change might be a 2-pixel margin shift in the header that's only visible if you know where to look. You could spend 10 minutes staring at two screenshots before you identify the region.

With `vision_diff`, the agent loads both captures' JPEG thumbnails, resizes them to matching dimensions, converts to grayscale, and computes per-pixel absolute difference. Pixels with |diff| > 30 are marked as changed. The image is divided into an 8x8 grid, and cells where more than 10% of pixels changed are flagged as changed regions. Result: `pixel_diff_ratio: 0.034` (3.4% of pixels changed), with 2 changed regions identified: `{x: 120, y: 24, w: 340, h: 48}` (the profile header) and `{x: 120, y: 80, w: 340, h: 16}` (the space between header and bio). The diff took under 1 millisecond.

> Pixel diff between pre-merge and post-merge profile page: 3.4% of pixels changed. 2 regions affected: profile header area (120, 24, 340x48) and header-bio gap (120, 80, 340x16). The header appears to have shifted down, increasing the gap above the bio section. This is consistent with a margin-top change on `.profile-header`.

**In plain terms:** vision_diff is a magnifying glass that pinpoints exactly what changed between two visual states. Instead of playing spot-the-difference, the agent highlights the changed regions and tells you precisely where to look.

---

## vision_similar — Finding Visually Similar Past States

Your user reports: "The login page looks weird. It looked like this before." They don't send a screenshot of "before." They just remember it was different.

Without similarity search, you'd scroll through Git history looking for login page changes, or manually browse visual captures by date hoping to find a match. If the login page changed three months ago and you have 500 captures, this is a needle-in-a-haystack problem.

With `vision_similar`, the agent captures the current login page and searches its visual memory for the most similar past states. The current page's CLIP embedding is compared against all 847 stored observations via cosine similarity. The top match (similarity: 0.96) is from February 3rd, session 8 — before the authentication flow redesign. The second match (0.91) is from January 20th. The agent now has a precise visual baseline for comparison.

> Found 3 similar visual states to the current login page. Closest match: capture #412 from February 3rd (similarity 0.96, pre-redesign). The current page closely resembles the pre-redesign version. This suggests the recent CSS deployment may have accidentally reverted login page styles.

**In plain terms:** vision_similar is visual déjà vu with precision. When something looks familiar, the agent searches its entire visual history to find the closest match — like a facial recognition system, but for UI states.

---

## quality_score — Visual Evidence Reliability

Your agent has 200 visual captures from the last month. Some are full-resolution screenshots with detailed labels and descriptions. Others are quick 320x240 crops with no metadata, captured in fallback mode when the CLIP model wasn't loaded. You're about to make a critical decision about a UI redesign based on visual comparisons. Which captures can you trust?

Without quality scoring, all captures are treated equally. A blurry, unlabeled 320x240 thumbnail carries the same weight as a crisp, well-documented 1920x1080 full-page screenshot. Your visual regression comparison uses a low-quality capture as the baseline, and the similarity score is meaningless because the baseline itself was unreliable.

With quality_score, every capture receives a composite reliability metric from four weighted factors: resolution (35% — how much detail is preserved), label completeness (20% — is the capture tagged?), description presence (20% — does the capture explain what it shows?), and model confidence (25% — was CLIP loaded for proper embedding?). A full-resolution, well-labeled capture with CLIP embeddings scores 0.85+. A quick fallback capture without labels scores 0.35. When comparing captures, the agent automatically weights its confidence by the lower of the two quality scores.

> Visual comparison confidence adjusted: baseline capture quality 0.42 (low resolution, no labels, fallback mode). Recommend recapturing the baseline at full resolution before making design decisions. Current comparison reliability: low.

**In plain terms:** Quality scoring is a credibility rating for visual evidence. Like a court evaluating the reliability of a witness photograph — resolution, provenance, and context all matter. The agent tells you when its eyes aren't trustworthy.

---

## vision_health — Diagnostic Checks

Your visual memory has grown to 500 captures over 3 months. Is it healthy? Are captures being properly labeled? Are old captures going stale? Is the memory linked to cognitive reasoning?

Without health diagnostics, visual memory degrades silently. Unlabeled captures become unsearchable. Stale captures from deprecated UI versions pollute similarity searches. Captures that aren't linked to memory nodes represent visual evidence that's disconnected from the agent's reasoning — it saw something but never connected it to a decision.

With `vision_health`, the agent runs a comprehensive audit with configurable thresholds: captures below quality 0.45 are flagged as low-quality, captures older than 168 hours (7 days) are flagged as stale, captures without labels are flagged as unlabeled, and captures without `memory_link` are flagged as unlinked. The status logic is strict: more than 50% low-quality or 70% unlinked triggers a "fail." More than 25% low-quality or any stale captures triggers "warn."

> Vision health: WARN. 500 captures across 14 sessions. 32 low-quality (6.4%), 147 stale (29.4%), 89 unlabeled (17.8%), 312 unlinked (62.4%). Primary concern: 312 captures not linked to memory nodes — visual evidence exists but isn't connected to reasoning. Recommend linking key captures to decision nodes.

**In plain terms:** vision_health is a visual memory audit. It tells you how well the agent is maintaining its visual evidence — like a librarian checking that books are shelved, cataloged, and cross-referenced, not just piled on the floor.

---

## vision_link — Binding Visual Evidence to Cognitive Memory Nodes

The agent made a Decision in cognitive memory: "The new checkout flow is better than the old one." But where's the evidence? The decision is a text node. The visual comparison that motivated it is a separate capture. There's no connection between the conclusion and the evidence.

Without vision_link, visual captures and cognitive memory exist in parallel universes. The agent "decided" the new checkout was better, but the before/after screenshots that supported that decision are orphaned in visual memory with no traceable connection. If someone later asks "what evidence did you base that on?", the agent can't point to the visual comparison — it would need to re-search visual memory and hope it finds the right captures.

With `vision_link`, the agent creates an explicit connection: capture #834 (the before screenshot) is linked to the Decision node with relationship `evidence_for`. Capture #835 (the after screenshot) is also linked as `evidence_for`. Now the cognitive reasoning chain has visual grounding. The CAUSED_BY edges on the Decision point to text-based reasoning; the vision_links point to visual evidence. The relationship types — `observed_during`, `evidence_for`, `screenshot_of` — make the connection semantics explicit. And linked captures are protected from storage budget pruning — evidence that supports decisions is never automatically deleted.

> Linked capture #834 (old checkout, quality 0.87) and #835 (new checkout, quality 0.91) to Decision node "New checkout flow is better." Relationship: evidence_for. These captures are now protected from storage pruning and traceable from the decision's reasoning chain.

**In plain terms:** vision_link is a footnote that connects an argument to its visual evidence. Instead of saying "trust me, I saw it," the agent says "here's the exact screenshot that supports my conclusion" — visual evidence with a paper trail.

---

## Non-Text Signal Quality — Decisions Based on Visual Confidence

Your agent needs to decide whether a UI component regression is severe enough to block a release. The text description from the DOM looks fine — all elements are present, all text is correct. But the visual capture shows a clear overlap between two elements that only manifests at a specific viewport width.

Without visual confidence scoring, the agent can only reason about what it can describe in text. The DOM says everything is fine. The agent signs off on the release. The overlap ships to production, visible to every user on a 1366px-wide display.

With visual evidence quality integrated into decision-making, the agent captures the regression at the problematic viewport width (quality score: 0.83). It compares against the baseline capture (quality: 0.87). The `vision_diff` shows a `pixel_diff_ratio: 0.12` — 12% of pixels changed in a region where 0% should have changed. The agent combines the visual evidence quality (both captures above 0.80) with the diff severity (12% change in a supposedly unchanged region) to produce a confidence-weighted assessment.

> Visual regression severity: HIGH. Pixel diff ratio 12% in the header region at 1366px viewport. Both captures high quality (0.83 and 0.87). DOM inspection shows no structural issues — this is a purely visual regression invisible to text-based analysis. Recommend blocking release until resolved.

**In plain terms:** Visual confidence lets the agent weigh what it sees against how well it can see. A blurry screenshot gets low confidence; a crisp capture gets high confidence. The agent doesn't just look — it knows how much to trust its own eyes.

---

## Parameter Safety — Strict Validation for Capture/Query/Track Surfaces

A poorly formatted API call passes `min_quality: 1.5` to vision_query. Another passes negative coordinates to a screenshot region. A third requests `vision_similar` with both `capture_id` and `embedding` set simultaneously.

Without parameter validation, these malformed inputs produce undefined behavior. A quality filter of 1.5 might match nothing (correct but confusing) or be silently clamped (convenient but dishonest). Negative coordinates might crash the screenshot tool or capture garbage data. Dual similarity inputs might use one and silently ignore the other, with no indication of which was chosen.

With strict validation enforced across every tool surface, every parameter is checked before execution. `min_quality` must be in [0.0, 1.0] — 1.5 returns a clear error: "min_quality must be between 0.0 and 1.0." `after` must be less than or equal to `before` — reversed ranges return "after timestamp must be less than or equal to before." `vision_similar` requires exactly one of `capture_id` or `embedding` — providing both returns "provide exactly one of capture_id or embedding, not both." Region dimensions must have `w > 0` and `h > 0`. Max results must be positive. Sort modes must be "recent" or "quality."

Every validation error returns a specific, actionable message. No silent failures. No undefined behavior. No garbage-in-garbage-out.

> Error: vision_query parameter validation failed. `min_quality` value 1.5 is outside valid range [0.0, 1.0]. Adjust to a value between 0.0 and 1.0 (e.g., 0.80 for high-quality captures only).

**In plain terms:** Parameter safety means the visual memory system fails loudly and clearly rather than silently doing the wrong thing. Every input is validated at the gate — like a bouncer who checks IDs rather than letting everyone in and hoping for the best.

---

## All Together Now: Debugging a UI Regression Across Multiple States

It's Wednesday morning. A QA engineer reports: "The product detail page looks broken on mobile. The 'Add to Cart' button is hidden behind the image carousel. It was fine last week." You engage the agent.

**Step 1: Capture the Current Broken State**

The agent uses `vision_capture` with a screenshot source, targeting the product detail page at mobile viewport (375x812). The capture processes through the full pipeline: JPEG thumbnail generated, CLIP embedding computed in 47 milliseconds, quality score calculated at 0.84 (good resolution, labeled "product-detail-mobile-broken", described as "Add to Cart button overlapping with carousel on mobile viewport"). Metadata sanitized — the product URL containing a user session token is redacted to `[redacted-secret]`. Capture ID: #1089.

**Step 2: Find the Last Known Good State**

The agent uses `vision_similar` with `capture_id: 1089` to find visually similar past captures. It searches 847 observations in 1.5 milliseconds. Top results: capture #1034 (similarity: 0.88, from last Tuesday, labeled "product-detail-mobile"), capture #987 (similarity: 0.82, from two weeks ago), and capture #956 (similarity: 0.79, from three weeks ago). The agent selects #1034 as the most recent "good" state — it was captured before the regression appeared.

**Step 3: Pixel-Level Diff to Identify the Change**

The agent runs `vision_diff` between #1034 (good) and #1089 (broken). The diff completes in under 1 millisecond. Results: `pixel_diff_ratio: 0.18` — 18% of pixels changed. Three changed regions detected: `{x: 0, y: 380, w: 375, h: 120}` (the carousel area expanded), `{x: 20, y: 460, w: 335, h: 52}` (the Add to Cart button shifted down and now overlaps), and `{x: 0, y: 520, w: 375, h: 80}` (the product description section pushed down). The primary change is in the carousel area — it grew vertically by approximately 120 pixels, pushing everything below it.

**Step 4: Compare with Pre-Regression Baseline**

The agent runs `vision_compare` between the broken state (#1089) and the known-good state (#1034). Cosine similarity on CLIP embeddings: 0.88 — similar but clearly different. `is_same: false`. The agent then compares #1034 against #987 (the state from two weeks ago): similarity 0.97 — the page was stable between those two captures. The regression happened between Tuesday (#1034) and today (#1089).

**Step 5: Link Visual Evidence to Cognitive Memory**

The agent creates a Decision node in cognitive memory: "Product detail page has a mobile layout regression. Carousel height increase is pushing Add to Cart button below the fold." It uses `vision_link` to connect both captures to this decision: #1034 linked as `evidence_for` (the baseline showing correct layout), #1089 linked as `evidence_for` (the broken state). The decision node also gets a CAUSED_BY edge to a Fact node: "pixel_diff_ratio 0.18 between known-good and current state, concentrated in carousel region."

**Step 6: Quality-Weighted Confidence Assessment**

The agent evaluates its confidence in the diagnosis. Capture #1034 has quality 0.81. Capture #1089 has quality 0.84. Both are above the 0.80 threshold for high confidence. The pixel diff is unambiguous — 18% change concentrated in 3 contiguous regions. The CLIP similarity drop (0.97 → 0.88) between consecutive captures confirms a genuine visual regression, not a data artifact. The agent assigns confidence 0.92 to its diagnosis.

**Step 7: Health Check and Diagnosis**

The agent runs `vision_health` to confirm its visual memory is reliable for this diagnosis. Status: "pass" — 847 captures, 12 low-quality (1.4%), 0 stale (recent captures used), 534 linked to memory nodes (63%). The captures used for this diagnosis are both high-quality and properly linked. No reliability concerns.

The agent delivers its final assessment:

> **Diagnosis (confidence: 0.92):** Mobile product detail page regression. The image carousel height increased by ~120px between last Tuesday and today, pushing the "Add to Cart" button below the fold at 375px viewport width. Visual evidence: 18% pixel diff concentrated in carousel and button regions. The page was stable for the two weeks prior (0.97 similarity between weekly captures). Both evidence captures are high quality (0.81 and 0.84) and linked to this diagnosis. Recommend checking CSS changes to the carousel component between Tuesday and today — likely a `max-height` or `aspect-ratio` change on `.product-carousel`.

Seven steps. Two captures compared. Three changed regions identified. One diagnosis with 0.92 confidence, backed by quality-scored visual evidence linked to cognitive memory. The entire workflow — from capture to diagnosis — took under 200 milliseconds of computation time. The agent didn't just describe the bug. It saw it, measured it, found when it appeared, identified the exact regions affected, and grounded its diagnosis in visual evidence that's permanently linked to its reasoning chain.

**In plain terms:** This is the difference between a developer who says "something looks off" and one who says "here's the before screenshot, here's the after, here are the three regions that changed, and here's exactly when it broke." AgenticVision gives your agent eyes that remember, compare, and build evidence — turning visual debugging from guesswork into forensics.
