# Top 20 Design Partner Outbound Sequences

Deeply personalized first-touch emails for the top 20 ranked prospects.

**Rules:**
- 50–80 words per email.
- Specific company trigger → operational hypothesis → interesting question → low-friction offer.
- No feature dumping. No marketing hype. No "I saw you are an AI company."

---

### 1. Retell AI (Binghai Zhao, Founder)
**Trigger:** Recent release of real-time conversational voice API emphasizing sub-second voice latency budgets.  
**Hypothesis:** Certain stable call-flow decisions (intent confirmation, state transitions) repeatedly pay 150ms+ cloud inference latency even after becoming repetitive.  

**Email (62 words):**
> Hi Binghai,
>
> Saw the recent real-time voice API launch — sub-second latency is clearly your primary wedge.
>
> At that speed, I imagine paying full model round-trip latency on recurring call-flow transitions (like intent confirmation) gets painful.
>
> I've been studying how often production voice agents keep calling cloud models for decisions that have already become highly repetitive.
>
> Happy to analyze a sanitized trace sample and share what's compilable. Worth a look?

---

### 2. Bland AI (Isaiah Granet, CEO)
**Trigger:** $16M Series A to scale enterprise phone agents across millions of inbound/outbound calls.  
**Hypothesis:** High call volume creates massive token spend on repetitive routing nodes (e.g. transfer vs qualify vs reject).  

**Email (64 words):**
> Hi Isaiah,
>
> Congrats on the $16M Series A to scale enterprise phone agents.
>
> At millions of calls, running remote inference on predictable conversational routing nodes (qualify vs transfer vs reject) must chew into unit margins.
>
> We've been looking at compiling repeated agent decisions into verified local fast paths that execute in under 0.2ms.
>
> Happy to analyze a sanitized trace export and show you what looks compilable before touching production code. Worth exploring?

---

### 3. Vapi (Jordan D'Amato, Founder)
**Trigger:** Widespread community traction for ultra-low latency voice bot infrastructure.  
**Hypothesis:** Repeated conversational state transitions and turn-taking classifications can execute in-process rather than round-tripping to cloud models.  

**Email (62 words):**
> Hi Jordan,
>
> Love the traction Vapi is seeing — developer focus on raw latency is the right bet.
>
> In voice pipelines, every 50ms matters. I'm curious whether your bots are repeatedly paying remote LLM latency on predictable state transitions that rarely change.
>
> We compile repetitive, verified agent decisions to sub-millisecond local execution.
>
> Happy to run discovery on a sanitized trace sample and send you an opportunity breakdown. Worth a look?

---

### 4. Decagon (Jesse Zhang, CEO)
**Trigger:** $35M Series A for enterprise customer support agents handling transactional enterprise workflows.  
**Hypothesis:** Repetitive API-calling decisions (e.g. refund eligibility vs specialist handoff) can be safely compiled once verified by CRM receipts.  

**Email (66 words):**
> Hi Jesse,
>
> Congrats on the $35M Series A for enterprise support automation.
>
> In transactional enterprise workflows, agents often repeat the same bounded API routing decisions (like refund vs specialist handoff) thousands of times.
>
> We build a Behavior JIT that compiles repeated, verified decisions into local fast paths, deoptimizing automatically when policies drift.
>
> Curious if you've measured your repeated decision share? Happy to inspect a sanitized trace sample offline if helpful.

---

### 5. Cursor / Anysphere (Aman Sanger, Co-Founder)
**Trigger:** $60M round and massive developer adoption where tab-completion and inline edits require sub-100ms latency.  
**Hypothesis:** Standard syntactic completion and linter correction dispatch can be served locally in <0.2ms instead of remote model calls.  

**Email (63 words):**
> Hi Aman,
>
> The developer velocity Cursor unlocked is incredible.
>
> With autocomplete and edit loops requiring extreme responsiveness, paying remote inference latency on repeated boilerplate patterns or standard linter fixes seems like an obvious bottleneck.
>
> We compile repeated, verified agent decisions into sub-millisecond local execution paths.
>
> Curious if this is something you've measured internally? Happy to run trace discovery on a sanitized sample and share what we find.

---

### 6. Cognition / Devin (Scott Wu, CEO)
**Trigger:** Devin autonomous coding agent scaling to complex customer repos with tight debug/test loops.  
**Hypothesis:** Tight compile/test debugging loops repeatedly make the same bounded linter/import fixes where test exit codes act as natural verifiers.  

**Email (64 words):**
> Hi Scott,
>
> Devin's autonomous capabilities have been fascinating to watch.
>
> In long agentic coding loops, agents repeatedly cycle through the same bounded linter, import, and syntax error decisions where test exit codes provide immediate verification.
>
> We compile verified decision sites into local fast paths that bypass the model entirely.
>
> Curious if you're tracking token burn across repeated debugging loops? Happy to review a sanitized trace sample.

---

### 7. Sierra (Bret Taylor, Co-Founder)
**Trigger:** Scaling enterprise conversational AI across Fortune 500 brands with strict brand and policy guardrails.  
**Hypothesis:** Repeated action decisions in sensitive workflows require strict drift deoptimization when business policies change.  

**Email (65 words):**
> Hi Bret,
>
> Sierra's focus on enterprise-grade reliability and governance for customer agents stands out.
>
> When business policies change, static caching and small models risk serving stale actions. We built a local runtime that qualifies repeated decisions against real outcomes and autonomously deoptimizes back to the model the moment drift occurs.
>
> Curious if policy drift governance is an active engineering focus? Happy to share our benchmark data or analyze a trace sample.

---

### 8. Augment Code (Scott Dietzen, CEO)
**Trigger:** $227M launch for enterprise AI coding assistant optimizing codebase comprehension.  
**Hypothesis:** Enterprise codebases have repetitive internal API patterns where suggestions can be verified by compiler success.  

**Email (63 words):**
> Hi Scott,
>
> Congrats on Augment's launch and momentum in enterprise developer tooling.
>
> In large enterprise codebases, developers repeatedly invoke the same internal libraries and patterns where compiler feedback provides an objective verifier.
>
> We compile repetitive, verified agent decisions into sub-millisecond local fast paths without touching model weights.
>
> Curious if you've explored compiling repetitive codebase decisions? Happy to evaluate a sanitized trace sample offline.

---

### 9. Forethought (Deon Nicholas, CEO)
**Trigger:** SupportGPT expansion and large customer wins for automated customer triage.  
**Hypothesis:** Top 20% of repetitive ticket types account for majority of inference spend with clear downstream resolution verifiers.  

**Email (64 words):**
> Hi Deon,
>
> Noticed SupportGPT's recent enterprise customer expansion.
>
> Support triage typically has a long tail of unique issues, but the top 20% of repetitive actions (like password reset routing or status checks) consume substantial inference budget.
>
> We compile verified decision sites into local fast paths that execute in 0.18ms while safely deoptimizing on drift.
>
> Happy to run discovery on a sanitized trace file and share what looks compilable. Worth exploring?

---

### 10. Intercom (Eoghan McCabe, CEO)
**Trigger:** Fin AI agent handling millions of enterprise customer support conversations.  
**Hypothesis:** Deflection and routing decisions for top recurring FAQ categories can run locally at sub-millisecond latency.  

**Email (63 words):**
> Hi Eoghan,
>
> Fin's scale across millions of customer support conversations is impressive.
>
> At that volume, paying frontier model inference costs for recurring triage and deflection decisions represents significant annual compute.
>
> We compile repeated, verified agent decisions into sub-millisecond local fast paths that fall back to the model on novelty.
>
> Curious if you've measured the percentage of Fin's decisions that are truly novel? Happy to share findings from similar workloads.

---

### 11. Relevance AI (Daniel Vassilev, Co-Founder)
**Trigger:** $10M Series A for multi-agent workforce platform where users build thousands of autonomous agents.  
**Hypothesis:** Custom user agents execute repetitive step routing that burns platform margin unless compiled.  

**Email (64 words):**
> Hi Daniel,
>
> Congrats on the $10M Series A for Relevance.
>
> As teams build recurring multi-agent workforces, their agents run the exact same tool-routing and step-selection decisions thousands of times, eating into platform compute margins.
>
> We build a local Behavior JIT that turns verified repeated decisions into 0.2ms fast paths.
>
> Curious if repeated agent step costs are becoming noticeable at scale? Happy to analyze a sanitized trace sample.

---

### 12. CrewAI (João Moura, Founder)
**Trigger:** Rapid open-source adoption for multi-agent orchestration frameworks.  
**Hypothesis:** Multi-agent delegation loops repeat similar task assignments where task completion acts as verifier.  

**Email (63 words):**
> Hi João,
>
> CrewAI's adoption has been incredible to watch.
>
> In multi-agent orchestration, delegation decisions between agents often burn significant token budgets on repeated task patterns where completion signals provide immediate verification.
>
> We compile repeated agent decisions into verified local fast paths, saving model calls without changing agent architecture.
>
> Curious if this is something CrewAI users frequently ask about? Happy to run discovery on a sample trace log.

---

### 13. Rootly (JJ Tang, CEO)
**Trigger:** Deepening AI capabilities for automated incident response and remediation.  
**Hypothesis:** Standard incident runbook selection can be compiled into deterministic fast paths verified by successful execution.  

**Email (63 words):**
> Hi JJ,
>
> Rootly's auto-remediation features are a huge step forward for SRE workflows.
>
> During production incidents, speed and determinism matter. Selecting standard runbooks for known alert signatures shouldn't wait on remote LLM latency when the path is already verified.
>
> We compile repeated, verified decisions into local fast paths that execute in 0.18ms.
>
> Curious if deterministic execution for standard runbooks is on your radar? Happy to compare notes.

---

### 14. PagerDuty AI (Dan Alexandru, VP Engineering)
**Trigger:** Launch of PagerDuty Copilot for automated incident triage and remediation.  
**Hypothesis:** Alert severity classification and on-call routing follow repetitive incident signatures verified by human acknowledgement.  

**Email (62 words):**
> Hi Dan,
>
> PagerDuty Copilot is an exciting addition to modern incident response.
>
> In high-volume monitoring pipelines, alert classification and escalation decisions repeatedly match known patterns where operator acknowledgements provide ground-truth verifiers.
>
> We compile verified decision sites into local fast paths that execute in under 0.2ms with formal safety invariants.
>
> Curious if you've measured the repetition rate across your alert triage streams? Happy to share our benchmark data.

---

### 15. Skyvern (Suhas, Co-Founder)
**Trigger:** Open-source browser automation platform gaining traction for workflow execution.  
**Hypothesis:** Repetitive DOM navigation decisions (e.g. login buttons, standard form fields) repeatedly invoke costly vision/text LLM calls.  

**Email (65 words):**
> Hi Suhas,
>
> Skyvern's approach to browser automation via visual grounding is super compelling.
>
> For recurring workflows, agents repeatedly call vision and LLM models on the exact same DOM actions (e.g. login, form submit) where successful DOM mutation verifies the outcome.
>
> We compile repeated agent decisions into verified local fast paths, eliminating remote model latency.
>
> Curious if you're looking at compiling recurring web flows? Happy to run trace discovery.

---

### 16. Browser-use (Greg Schlom, Founder)
**Trigger:** Rapid viral traction in agentic browser automation.  
**Hypothesis:** Standard web workflow steps (form completion, navigation) can be compiled into local fast paths verified by page state changes.  

**Email (63 words):**
> Hi Greg,
>
> Browser-use has caught fire in the developer community — great work on the launch.
>
> Running browser agents through repeated web tasks burns tokens fast, even when 60%+ of navigation actions are identical across runs.
>
> We compile verified agent decisions into sub-millisecond local fast paths that deoptimize when page structures change.
>
> Curious if you've explored compiling common browser agent trajectories? Happy to share what we've seen.

---

### 17. Reducto (Aditya, Co-Founder)
**Trigger:** Document extraction API scaling across enterprise PDFs and complex forms.  
**Hypothesis:** Extracting standard structured forms (e.g. invoices, W2s) repeatedly calls models for identical table layouts.  

**Email (64 words):**
> Hi Aditya,
>
> Reducto's document parsing benchmarks are impressive.
>
> In enterprise document processing, models frequently process thousands of standard layout variants where schema validation provides an immediate programmatic verifier.
>
> We build a Behavior JIT that compiles repeated structured decisions into local fast paths, eliminating remote inference costs on known templates.
>
> Curious if you're exploring ways to compile repeated extraction structures? Happy to analyze a sanitized trace sample.

---

### 18. Adept AI (David Luan, CEO)
**Trigger:** Enterprise browser agent deployments across CRM and ERP platforms.  
**Hypothesis:** Repetitive enterprise data-entry tasks repeat identical DOM actions verified by form submission success.  

**Email (63 words):**
> Hi David,
>
> Adept's focus on enterprise process automation addresses a massive workflow need.
>
> In repetitive enterprise data entry (Salesforce, Workday), agents repeatedly make the same bounded UI decisions where successful submission provides verifiable outcomes.
>
> We compile verified decision sites into local fast paths that execute in under 0.2ms with automatic drift revocation.
>
> Curious if compiling repeated enterprise workflows is something you've considered? Happy to share our findings.

---

### 19. Portkey (Rohit Agarwal, CEO)
**Trigger:** AI gateway and observability processing billions of tokens for enterprise AI teams.  
**Hypothesis:** Observability layer identifies recurring prompt signatures that could execute locally rather than hitting model providers.  

**Email (65 words):**
> Hi Rohit,
>
> Portkey's momentum in the AI gateway and observability space is remarkable.
>
> Processing billions of tokens gives you visibility into which agent decisions repeat constantly. Rather than static semantic caching that breaks under drift, we compile repeated decisions into verified local fast paths with automatic deoptimization.
>
> Curious if gateway-level decision compilation is on your roadmap? Happy to share our competitive frontier benchmarks comparing JIT execution against semantic caches.

---

### 20. Martian (Yash Shevdatia, Co-Founder)
**Trigger:** $9M seed for dynamic model router optimizing cost and performance trade-offs.  
**Hypothesis:** The routing layer itself has recurring classification patterns that can be fast-pathed locally without calling classifier endpoints.  

**Email (63 words):**
> Hi Yash,
>
> Martian's work on dynamic model routing tackles one of the biggest problems in AI systems.
>
> Interestingly, the model routing decision itself often repeats across stable traffic segments. We compile bounded decision sites into local fast paths that run in 0.18ms while maintaining statistical safety guarantees.
>
> Curious if you've looked at compiling the routing decision itself? Happy to compare benchmark methodologies if of interest.

---

## Weekly Tracking Discipline
1. Log every sent email in `design_partners.csv`.
2. Follow-up 1 after 3–4 business days if no reply.
3. Once a positive reply arrives, reply within 1 hour: offer a 15-minute call or send the 75-second sales demo recording.
