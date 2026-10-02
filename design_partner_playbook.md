# Design Partner Playbook

## The Offer

> Send us a sanitized trace sample. We will tell you which model calls appear worth compiling before you change production code.

No contract. No SaaS account. No dashboard. No credit card.

First deliverable: a one-page Microloop Opportunity Report.

---

## Ideal Customer Profile

All must be true:

- Production AI workload
- High model volume
- Bounded recurring decisions
- Measurable downstream outcomes
- Latency and/or inference cost matters
- Low-to-moderate decision risk
- Engineering team controls model invocation

### Strong verticals

Voice agents, support automation, AI workflow platforms, tool-using enterprise agents, document-processing agents, incident/ops agents, browser/workflow automation, coding/CI agents.

### Weak initial targets

Pure creative generation, research-heavy agents, low-volume internal copilots, mostly free-form text generation, teams without measurable outcomes.

---

## Qualification Scoring

### Strong

- Bounded traffic ≥ ~20% of meaningful workload
- Repeat rate ≥ ~20%
- Clear independent verifier
- High call volume
- Model latency ≥ ~100ms OR meaningful cost
- Policy changes occur

### Medium

One or two dimensions weak.

### Reject

- Mostly free-form generation
- No downstream verifier
- Very low traffic
- Model already effectively free/instant
- Extremely high-risk irreversible decisions

Tell prospects when Microloop is not useful. That builds trust.

---

## First Call: Discovery, Not Demo

### Questions to ask

1. What does the agent/workflow actually do?
2. How many model calls do you make per day/week?
3. Which model calls are expensive or slow?
4. Which decisions recur with similar inputs?
5. Which outputs are bounded (finite set of choices)?
6. What happens after those decisions?
7. Can you later determine whether they were correct?
8. How often do policies/prompts/business rules change?
9. What have you already tried? (caching, cheap models, model routing, classifiers, rules?)
10. Which metric matters most: cost, latency, reliability, capacity?

Do not force Microloop onto a bad workload.

---

## Trace Pilot

1. Receive sanitized historical traces (JSONL, OpenTelemetry, LangSmith, LiteLLM exports)
2. Run `microloop discover`
3. Generate the Opportunity Report (see `opportunity_report_template.md`)
4. If no valuable site exists: tell the prospect. Record `NO FIT`.

No production changes yet.

---

## Live Pilot

### Setup

- Select **1 DecisionSite** (not five)
- Integration target: ≤ 1 file, ≤ 20 LOC where possible
- Never start directly ACTIVE

### Lifecycle

```text
OBSERVE → SHADOW → ACTIVE
```

### Duration

2–4 weeks of genuine traffic variation.

### Required telemetry

For every partner, measure:

- Weeks live
- Total application LLM calls
- DecisionSite calls
- Bounded/verifiable share
- Microloop ACTIVE serves
- Model calls avoided
- Whole-application calls avoided
- Whole-application spend reduction
- p50/p95 fast-path latency
- Fallback latency
- Qualification cost
- Comparison overhead
- Break-even point
- Verified local errors (with 95% CI)
- Revocations
- False revocations
- Requalifications
- Missing outcomes
- Policy changes observed

Never report only DecisionSite coverage.

---

## Retention Signal

After 2–4 weeks, ask:

> If I removed Microloop tomorrow, would you care?

Record: YES (strong) / MAYBE (weak) / NO (failed)

Also record:
- Still enabled?
- Expanded to another site?
- Asked for another feature?
- Willing to introduce us to another team?

Three users who would be upset if Microloop disappeared > ten pilots.

---

## Follow-up Sequence

**Follow-up 1** (3–4 business days):

> Quick bump — happy to do the trace analysis entirely on my side first, so there is nothing to integrate unless the numbers are actually interesting.

**Follow-up 2** (4–6 business days later):

Add one new useful observation specific to their company.

**Follow-up 3** (close loop):

> I'll leave this here. If reducing repeated model decisions becomes relevant later, happy to compare notes.

Do not send endless sequences.

---

## Pricing Discovery

Do not implement billing. Ask every serious partner:

1. How would you prefer this priced? (flat fee / usage-based / share of savings / per-site / enterprise annual)
2. What would this need to save or improve for \$1k/month to be trivial?
3. What about \$5k/month?

Record actual responses.

---

## Product Feedback Rule

Before adding a substantial feature: **3 independent customers** must ask for approximately the same thing.

Exceptions: critical correctness bugs, security issues, data-loss issues, integration blockers affecting all users.

---

## Weekly Operating Rhythm

**Monday:** Review funnel, select 25 high-fit prospects, research triggers.

**Tuesday–Thursday:** Send personalized outbound, hold calls, run trace discovery, onboard pilots.

**Friday:** Review — what messages got replies? What problems repeated? Which DecisionSites appeared? What integration friction repeated? What customers asked for? What did they NOT care about? Update product only from repeated evidence.
