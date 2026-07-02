# Synaptic retention scoring

OpenTerm ranks history for cleanup without deleting it automatically. The model
is deterministic, local, and intentionally explainable.

## Model

Freshness follows exponential half-life decay:

```text
freshness = 2 ^ (-age_days / 30)
```

Repeated evidence access saturates smoothly instead of growing without bound:

```text
access = 1 - exp(-access_count / 3)
```

The final keep probability is a logistic transform:

```text
z = -1.4
  + 2.2 * freshness
  + 1.1 * access
  + 0.8 * diagnostic_value
  + 0.4 * compactness

keep_score = 1 / (1 + exp(-z))
```

Failed commands receive higher diagnostic value because their evidence is
often useful for unresolved work. Compactness prevents very large, never-used
results from dominating storage indefinitely. Pinned results always score 1.

`ai-clean-history` shows keep, freshness, use, and diagnostic components. It
only suggests candidates. Deletion still requires `ai-forget RESULT`.

## Mathematical constraints

- Every component is bounded to `[0, 1]`.
- Freshness decreases monotonically with age.
- Access value increases monotonically but saturates.
- Failure evidence cannot reduce retention.
- Pinning dominates every other signal.
- Integer inputs are bounded before floating-point conversion.

Property-oriented tests enforce the important monotonic relationships. Future
signals such as reference graph degree and duplicate similarity must preserve
these constraints and ship with calibration benchmarks.
