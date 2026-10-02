# Microloop Opportunity Report

**Company:** [Company Name]  
**Analysis Date:** [Date]  

## Trace Summary
- **Total Model Calls:** [Number]
- **Time Period:** [Duration]
- **Agent Architecture Overview:** [Brief Description]

## Candidate DecisionSites

| Site Name | Traffic Share | Repeat Rate | Entropy | Bounded Choices | Verifier Availability | Recommendation |
|---|---|---|---|---|---|---|
| [Site 1] | [%] | [%] | [Low/Med/High] | [Yes/No] | [Description] | [Compile/Shadow/Ignore] |

## Rejected Sites

| Site | Reason |
|---|---|
| [Site A] | High-entropy generation; no objective verifier. |

## Estimated Opportunity
- **Qualified Traffic Share:** [%]
- **Potential Fast-Path Coverage:** [% of decision calls]
- **Estimated Whole-System Call Reduction:** [Typically 3.99–10.10%]
- **Estimated Latency Impact:** [e.g., local execution at ~0.19ms vs model latency]

## Risks and Limitations
- Drift may require frequent fallback if decision distributions shift rapidly.
- Integration requires accurately capturing outcome telemetry (`record_outcome`).

## Next Steps
- Select 1 Candidate DecisionSite for a Live Pilot.
- Integrate the `client.decide()` wrapper (≤20 LOC).
- Begin the OBSERVE phase in staging/production.
