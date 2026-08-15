---
name: external-discovery
description: Evaluate relevant external prior art before substantial new engineering while preserving qualified existing work.
---

# External Discovery and Reuse

## Activation

Use this skill before substantial work on a new capability, subsystem, algorithm, infrastructure component, model, memory system, agent mechanism, protocol, evaluation system, or other significant technical decision. Also activate after repeated failed attempts, entry into a new problem domain, a major course correction, or when technical novelty is being claimed. Routine fixes and established architecture do not require an extensive landscape check.

## Governing principle

Preserve what works. Search before building what comes next. Reinvent only when there is a reason.

## Existing work

External discoveries do not automatically justify changing stable or qualified work. Do not re-architect, replace working components, reopen completed stages, invalidate evidence, introduce unnecessary dependencies, or migrate merely because another implementation exists. Propose change only when the material benefit justifies migration, integration, regression, revalidation, dependency, governance, and maintenance costs.

## Discovery

Search for the underlying problem, not only project terminology. Use sources appropriate to the decision, such as GitHub, GitLab, package registries, Hugging Face, arXiv, research literature, academic projects, frameworks, SDKs, APIs, standards, protocols, and adjacent technical disciplines. Scale research depth to the cost and importance of the decision; do not turn research into an expensive distraction.

## Evaluation

For serious candidates, consider functional overlap, architecture, license, maintenance, security posture, tests, documentation, dependencies, community maturity, integration complexity, performance limits, and long-term maintenance. For future work, evaluate whether to ADOPT, WRAP, EXTEND, FORK, COMPOSE, REFERENCE, or BUILD.

## Mid-implementation discovery

Do not automatically stop active work when an external candidate is found. Reconsider only when it is likely to eliminate substantial remaining effort or materially improve the architecture. Balance completed work, remaining work, migration cost, validation cost, compatibility, dependency risk, and long-term benefit.

## Reporting

When discovery materially affects a future decision, record a concise note in the appropriate `.agent/` record without rewriting existing data: what was found, the problem addressed, the source, approximate overlap, the proposed disposition, and the rationale. This skill is additive and does not supersede active scope, governance, qualification criteria, architecture, directives, or completed work.
