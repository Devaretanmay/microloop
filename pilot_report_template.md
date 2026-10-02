# Pilot Case Study: [Company Name]

## Overview
- **Company/Category:** [Company] / [Category]
- **Workload:** [Description of workload]
- **Traffic Scale:** [Total calls/day]
- **Problem:** [Cost/latency issues tied to repeated decisions]
- **DecisionSite:** [Name/Function of site]
- **Verifier:** [How outcomes were objectively qualified]

## Pilot Details
- **Qualification Period:** [Duration]
- **Weeks Live:** [Weeks]
- **Integration LOC:** [Number, e.g., 15 LOC]

## Results (Before / After)
- **% DecisionSite Calls Avoided:** [%]
- **% Whole-App Calls Avoided:** [%, benchmarks indicate 3.99–10.10%]
- **% Whole-App Spend Reduction:** [%, benchmarks indicate 3.51–8.87%]
- **Latency Impact:** [e.g., Model latency -> 0.18 ms local execution]

## Safety & Stability
- **Verified Errors:** [Number]
- **CI Width (Confidence Interval):** [Value]
- **Revocations (Deoptimizations):** [Number, e.g., 7-8 serves before drift detection]
- **Break-Even Point:** [Time/traffic required to offset compile cost]

## What Microloop Could NOT Optimize
- [List of sites or workload types correctly rejected due to high entropy or lack of objective verifiers]
