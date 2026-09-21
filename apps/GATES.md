# GATES — Composer CRUD pass (2026-08-20)

- [ ] G25 Controls CRUD ;; control select over the whole catalog; draft new control (id validated `FAM-N`, uniqueness fault `⊭ ids are identities`, enters as ○ claimed on the map's drafting strip + outline + counts); generic document editor (title, statements add/edit/remove, owner assign); removal via destructive decision naming orphaned deps — receipts generated/transformed/resolved
- [ ] G26 Mappings CRUD ;; bridge mappings card in Composer: rows per framework, new/edit via form (practice, target, relationship form, rationale, conf/cov clamped 0–1); editing an attested edge reopens the claim as ○ draft (seal discarded, said in the gate); removal gated; Bridge diagram/ledger + all counts reflect deltas live

# GATES — "responsive + no vaporware" pass (2026-08-20)

- [ ] G18 Bridge legible ;; edge plates 144×72 opaque, curves drawn in a separate pass under all plates; meta shortened to `rationale · conf/cov`; grid gets overflow-x scroll + 180px min side columns
- [ ] G19 Bridge ledger representation ;; diagram·ledger segment; real .ck-ledger table (from · relationship glyph · to · rationale · conf · cov · method · claim-state) with tfoot subtotal; rows keyboard-operable; lens drives default (low/zero exposure → ledger)
- [ ] G20 Responsive shell ;; vw-tracked: <1360 margin auto-collapses (user toggle wins), <940 nav → 64px icon rail, <720 stacked single column (nav strip, margin bar, wrapping masthead/footer)
- [ ] G21 Env + promotion governed ;; env segment dev/staging/prod in identity strip; authority footer derives level (open/restricted/sealed) from env; promote → prod is a sealed decision listing live gates, blocked on the constraint fault, confirm seals envelope + violet receipt
- [ ] G22 Composer closes the loop ;; assign owner (clears ⊭, receipt transformed) → publish (rust primary, receipt approved); commit resolution → version 15+, baseline rotates so BEFORE/AFTER diff is real
- [ ] G23 Pipeline runs ;; gates data-driven from app state (constraint fault ⇄ owner, coverage Δ from alters); re-run pipeline writes receipts + run history; CLI transcript flips fault→pass
- [ ] G24 Root fabric ;; DotBg v3 drafting surface behind the Atlas map, theme-aware, cached per theme

# GATES — enhancement pass (2026-08-19)

- [x] G11 Governed export gate ;; margin "export snapshot" → sealed decision (.ck-decision--sealed) with real sha-256 envelope digest; confirm writes violet `exported` receipt + proof toast; Esc closes it first in the chain — verifier pass clean
- [x] G12 Copy-for-AI hand-off ;; Composer "copy for AI" emits a ckodex-context bundle (live content_digest + two-pass bundle_digest, internal banner) to clipboard; plain `exported` receipt; body tracks params/alters/derived — verifier pass clean
- [x] G13 Chrome refactor ;; nav grouped into territory/record/operations with ink rail active state + keys 1–6; two-tier header (masthead + workspace identity strip, operable urn); repo block redesigned; map gains drag-pan + zoom cluster with click-guard — verifier pass clean (ledger)
- [x] G14 HC opacity floor ;; [data-theme="hc"] * {opacity:1 !important} defeated all opacity-hidden layers; posture labels, node layer, node id labels now presence-based (sc-if) — verifier probe: no % foreignObjects in controls zoom
- [x] G15 Nav overflow at short viewports ;; nav scrollHeight 509 vs client 379 pushed repo block under footer; fixed overflow-y:auto — load check clean
- [ ] G16 HC-safe overlays ;; freshness re-encoded as fill-level gauge (geometry, not fill-opacity); drift/POA&M out-of-register nodes render outline-only in fg-mute (hue, not opacity alone)
- [ ] G17 Time machine under HC ;; future receipts now removed from the derived view with a ◌ "N receipts ahead of cursor" pending row (presence, not opacity dim)

# GATES — Atlas "more + unlazy" pass (2026-08-16) — CLOSED 10/10

- [x] G1 Lens operable in-app ;; ;; probe per persona (fresh load)
  menuOpen:true · author → "lens author · exposure low", tiers in inspector = 2 (no Raw) · ciso → posture tiles rendered (.ck-bento true), "exposure zero", edit acts absent (editBtn:false) · engineer → raw pre visible (engineerRaw:true) · architect restore ok. Screenshots captured.
- [x] G2 Map nodes keyboard-operable ;; ;; DOM probe
  kbAttrs:true (tabindex=0, role=button, aria-label = full tip), Enter on AC-2 opened the inspector (kbOpens:true).
- [x] G3 Zero console errors after full sweep ;; ;; webview logs
  Sweep executed (6 surfaces, node select, edge select, CSF switch, chip open, alter toggle, decision gate open/Esc, ledger filter+sample, docket sort+row-click→Atlas, lens+theme switches) — get_webview_logs: "(no webview logs)".
- [x] G4 Themes legible on inherited text ;; ;; computed-style probe
  vault: nav+main color rgb(234,229,218) on bg rgb(10,19,34) · hc: rgb(255,255,255). No ledger-ink bleed. Vault/hc screenshots captured.
- [x] G5 No horizontal overflow incl. stateful layouts ;; ;; probe
  atlas 0 · atlas+inspector 0 · bridge 0 · bridge+edge 0 · composer 0 · composer+chip 0 · ledger 0 · docket 0 · docket+due 0 · pipeline 0. Re-measured after row-pitch change: 0.
- [x] G6 Rust budget ≤2 per surface ;; ;; probe
  atlas 0 · bridge 0 · composer 0 · ledger 0 · docket 1 · pipeline 0 (inline-style accent count; selection ring + pulse add ≤2 transient).
- [x] G7 Bridge label collisions = 0 ;; ;; bbox intersection probe
  Was 2/2 (meta↔rel grazes at Δmid 56–64px). Fix: row pitch 68→84px (≥64px label block), label boxes 150→140w, cluster threshold 40→64. Re-measured fresh: collIso 0 · collCsf 0 · ov 0.
- [x] G8 Decay bars monotonic with age ;; ;; probe
  (6min,90)(55min,90)(3h,90)(26h,90)(12d,87)(74d,72)(160d,50)(335d,7) — non-increasing with age ✓.
- [x] G9 Composer diff reacts + decision gate intercepts ;; ;; DOM probe
  a1 toggle: AFTER line c "90 days"→"30 days", "1 alterations", 401 ms · a3 toggle: .ck-decision opened, alterations stayed 0 (nothing applied), Esc closed it, receipt line names "resolved · profile MER-MOD · remove AC-2(9)".
- [x] G10 Esc chain order ;; ;; scripted probe
  decision open → Esc closed (decideEscClosed:true) · chip + lens menu both open → Esc closed menu, chip stayed (esc1) · Esc closed chip (esc2) · earlier: Esc closed inspector last. Order: decision > lens > chip > edge > inspector ✓.

## ABANDON
none

## Known probe artifacts (not defects)
- afterPre/Post lines appear duplicated in probe output (nested span matching); rendered text is single.
- Hot-edited previews can serve a stale template; all evidence above re-measured on fresh loads.
