# OpenTerm — Complete AI-Oriented Terminal Roadmap

**Document status:** Architecture and delivery roadmap, revision 4  
**Product direction:** terminal application + persistent session engine + native RTK compression + AI tool suite + plugin platform  
**Initial flagship plugin:** workspace documentation agent
**Product name:** OpenTerm  
**Mascot:** canonical interactive terminal monster preserved in `design/mascot/`
**Current implementation:** Rust foundation, providers, RTK-backed P0 commands,
encrypted evidence references, progressive retrieval, smart history scoring,
and a Ghostty-based native macOS terminal alpha with bundled tools, native
appearance UI, animated mascot launch, and automatic Git-aware shell context  
**Next implementation slice:** OpenTerm session daemon, native command boundaries,
and native RTK filter extraction

## 1. Executive summary

OpenTerm is a complete AI-native terminal platform. It combines a fast terminal application, persistent multiplexed sessions, native integration with [RTK](https://github.com/rtk-ai/rtk), a small low-code command language, a token-aware tool runtime, and a plugin ecosystem. It lets humans and agents inspect files, search code, run processes, use remote systems, transform data, and collaborate without flooding the model context window.

The idea is feasible and useful, with one important constraint:

> Do not rebuild Unix, SSH, Git, or mature parsers from scratch. Build a safe, typed, token-aware control plane over them.

Traditional terminal tools optimize for humans looking at streams of text. AI agents need different defaults:

- structured output instead of terminal prose;
- bounded output instead of unbounded streams;
- stable references instead of repeating large results;
- previews before mutation;
- resumable operations;
- explicit side-effect and permission metadata;
- semantic compression;
- deterministic behavior that is easy to test;
- composition without fragile shell quoting.

OpenTerm should therefore contain three layers:

1. **Adapters** wrap reliable existing tools and operating-system APIs.
2. **AI primitives** normalize, limit, cache, summarize, diff, and reference results.
3. **A low-code workflow language** composes primitives into reusable, auditable operations.

The first release should target codebase exploration and a documentation agent inside a real terminal experience. Safe file editing follows once result provenance, permission boundaries, and the plugin protocol are stable. Remote access, process orchestration, Git writes, databases, and cloud operations come later.

---

## 2. Product thesis

### 2.1 The problem

A normal terminal session wastes model context in predictable ways:

- `grep` repeats long matching lines and irrelevant vendor files.
- `find` emits thousands of paths without hierarchy or prioritization.
- `cat` sends entire files when only one symbol is needed.
- build and test commands repeat progress, warnings, and stack traces.
- `git diff` is often larger than the reasoning task.
- remote commands add connection noise to useful output.
- pipelines serialize structured information into text and immediately parse it again.
- agents repeatedly request the same data because results have no persistent identity.
- shell quoting and platform differences create avoidable failures.

The expensive resource is not only execution time. It is the amount of result data that must enter the model's active context.

### 2.2 The opportunity

An AI-oriented terminal can save tokens by changing the interface contract:

```text
request -> execute -> normalize -> store full result -> return compact view + handle
                                                           |
                                                           +-> expand only on demand
```

Example:

```text
ai.search query="TODO|FIXME" scope=src
```

Instead of returning 800 lines, the command could return:

```yaml
status: ok
summary: 63 matches in 18 files
groups:
  - path: src/api/auth.ts
    matches: 12
    preview: [44, 91, 120]
  - path: src/db/cache.ts
    matches: 8
    preview: [17, 56, 80]
result_ref: res_01J...
truncated: true
next: ai.expand res_01J --group 1
```

The complete result remains available in a local content-addressed store. The model receives only the information needed for its current decision.

### 2.3 What “low code” should mean

Low code should not mean vague natural-language execution. It should mean concise, typed declarations with strong defaults:

```yaml
workflow: fix-and-test
inputs:
  issue: string
steps:
  - search: { query: "${issue}", scope: src }
  - inspect: { refs: "${search.top_files}", mode: symbols }
  - patch: { from: "${inspect}", approval: preview }
  - test: { impacted_by: "${patch.files}", budget: 90s }
  - report: { include: [changes, failures, risks] }
```

Every step should map to a deterministic primitive. Natural language may help generate workflows, but it must not be the runtime protocol.

---

## 3. Goals and non-goals

### 3.1 Goals

- Reduce terminal output entering AI context by 60–95% on common coding tasks.
- Preserve access to complete, lossless output through result references.
- Make read, write, network, privilege, and destructive effects machine-readable.
- Make common commands portable across macOS, Linux, containers, and remote hosts.
- Support concise one-shot commands and reusable declarative workflows.
- Produce deterministic JSON/YAML results suitable for agents and humans.
- Keep adapters replaceable so the runtime can select `rg`, Git, native APIs, or fallbacks.
- Make execution observable, replayable, and auditable.
- Work locally first and offline by default.

### 3.2 Non-goals

- Reimplement SSH encryption, authentication, or transport.
- Replace POSIX shells for interactive human administration.
- Rebuild Git's object database or merge algorithms.
- Create new versions of `sed`, `awk`, and every GNU utility merely for completeness.
- Hide command failures behind generated summaries.
- Depend on an LLM for deterministic parsing, file mutation, or policy enforcement.
- Automatically execute arbitrary natural-language intent without a typed plan.
- Guarantee universal portability in the first release.

---

## 4. Design principles

1. **References before repetition** — large outputs become handles such as `res_…`, `file_…`, `proc_…`, and `snap_…`.
2. **Progressive disclosure** — return counts and grouped previews first; expand exact sections later.
3. **Structure before prose** — result fields are stable and schema-versioned.
4. **Lossless underneath, compressed on top** — summaries never destroy the original output.
5. **Semantic boundaries** — inspect symbols, diagnostics, hunks, records, and test cases rather than arbitrary line blocks when possible.
6. **Safe mutation** — edits return previews, preconditions, inverse operations, and verification results.
7. **Explicit budgets** — commands accept byte, token, item, time, and process limits.
8. **Local intelligence first** — deterministic filters run before optional model-based summarization.
9. **Capability-based security** — workflows receive only declared filesystem, command, network, and secret capabilities.
10. **Adapter reuse** — mature executables and libraries do the dangerous or highly complex work.
11. **Idempotency where possible** — retries should not duplicate writes or remote actions.
12. **Explainable compression** — every omitted section has a count, reason, and expansion path.

---

## 5. Core architecture

### 5.1 Components

#### CLI and REPL

Human- and agent-facing entry points:

- `ai-*` executables for scripts and tool calls;
- an optional `openterm>` REPL;
- JSON-RPC/MCP-compatible server mode;
- SDKs for TypeScript, Python, and later Rust/Go;
- workflow runner for YAML or JSON definitions.

#### Command registry

Every command declares:

- input schema;
- output schema;
- side effects;
- required capabilities;
- default budgets;
- cache behavior;
- preview/commit support;
- cancellation and timeout behavior;
- platform adapters;
- redaction rules.

#### Execution engine

Responsibilities:

- validate input;
- resolve workspace and sandbox boundaries;
- enforce permissions and budgets;
- call native APIs, libraries, or existing executables;
- stream events into the result store;
- normalize errors;
- generate compact views;
- issue stable references;
- record provenance and metrics.

#### Result store

A local content-addressed store containing:

- raw stdout and stderr;
- normalized records;
- command metadata;
- file snapshots and hashes;
- summaries and indexes;
- parent/child result relationships;
- expiration policy and size accounting.

Use SQLite for metadata and compressed blobs on disk initially. Avoid inventing a distributed database before it is needed.

#### Policy engine

The policy engine evaluates:

- filesystem scopes;
- network destinations;
- executable allowlists;
- destructive action classes;
- secret access;
- remote-host capabilities;
- approval requirements;
- organization policy overrides.

#### Compression engine

Compression stages should be deterministic by default:

1. remove ANSI and progress animation;
2. detect repeated lines and collapse them with counts;
3. group records by file, test, diagnostic, host, or operation;
4. rank groups by relevance and severity;
5. preserve heads, tails, errors, and boundary context;
6. generate an extractive summary;
7. optionally add an LLM summary as a separate, clearly labeled field.

### 5.2 Result envelope

All commands should return a common envelope:

```json
{
  "schema": "openterm.result/v1",
  "command": "code.search",
  "status": "ok",
  "summary": "14 matches in 5 files",
  "data": {},
  "diagnostics": [],
  "refs": [{"id": "res_01J...", "kind": "search-results"}],
  "omitted": {"items": 9, "bytes": 28410, "reason": "item_budget"},
  "next": [
    {"command": "result.expand", "args": {"ref": "res_01J...", "group": 2}}
  ],
  "effects": {"filesystem": "read", "network": "none", "process": "spawned"},
  "metrics": {"duration_ms": 31, "raw_bytes": 30120, "returned_bytes": 1710}
}
```

### 5.3 Stable reference types

- `res_*`: immutable command result;
- `view_*`: filtered or summarized view of another result;
- `file_*`: file content at a known hash;
- `span_*`: exact line, byte, AST, or symbol range;
- `snap_*`: workspace or directory snapshot;
- `patch_*`: proposed change set;
- `proc_*`: running or completed process;
- `job_*`: workflow execution;
- `host_*`: configured remote host;
- `secret_*`: opaque credential reference, never secret content;
- `artifact_*`: build, archive, report, or generated asset.

References must include workspace identity and expiration semantics. Mutable files must be guarded by expected hashes to prevent stale edits.

---

## 6. Command strategy: rebuild, wrap, or avoid

Use three implementation classes:

- **Native**: implement when the AI-oriented behavior is the product itself.
- **Adapter**: wrap a mature command or library and normalize its result.
- **Avoid**: do not reimplement; expose only a guarded interface.

### 6.1 Filesystem and navigation

| Traditional command | OpenTerm command | Strategy | AI-native behavior | Priority |
|---|---|---|---|---|
| `pwd` | `fs.where` | Native | workspace, repo root, relative path, capabilities | P0 |
| `cd` | `fs.focus` | Native | scoped logical context, bookmarkable and workflow-safe | P0 |
| `ls` | `fs.list` | Native | grouped tree, ignored/generated detection, item budgets | P0 |
| `tree` | `fs.tree` | Native | depth and token budgets, directory summaries | P0 |
| `find` | `fs.find` | Native | typed filters, ranking, grouped results, refs | P0 |
| `stat` | `fs.stat` | Native | hashes, MIME, ownership, change state | P0 |
| `file` | `fs.classify` | Adapter | MIME, encoding, language, binary/generated flags | P1 |
| `du` | `fs.usage` | Native | hot spots, ignored caches, cleanup candidates | P1 |
| `df` | `sys.storage` | Adapter | concise capacity warnings and mount metadata | P2 |
| `mkdir` | `fs.ensure-dir` | Native | idempotent, reports created/existing | P0 |
| `touch` | `fs.ensure-file` | Native | idempotent with template/hash options | P1 |
| `cp` | `fs.copy` | Native | preview, collision plan, hashes, rollback record | P1 |
| `mv` | `fs.move` | Native | reference-aware rename, collision plan | P1 |
| `rm` | `fs.remove` | Native | preview, quarantine default, explicit permanent mode | P1 |
| `ln` | `fs.link` | Native | link-type validation and portability warning | P2 |
| `chmod` | `fs.permissions` | Adapter/native | symbolic intent, before/after diff, risky-bit warnings | P2 |
| `chown` | `fs.owner` | Adapter | privileged, preview-first, tightly gated | P3 |

`fs.focus` should not mutate a global shell working directory. It returns a context reference that subsequent calls inherit. This makes concurrent workflows safe.

### 6.2 Reading and inspecting content

| Traditional command | OpenTerm command | Strategy | AI-native behavior | Priority |
|---|---|---|---|---|
| `cat` | `fs.read` | Native | line/token budget, binary guard, content ref | P0 |
| `head` / `tail` | `fs.peek` | Native | meaningful boundaries, log-aware modes | P0 |
| `less` / `more` | `result.page` | Native | cursor pagination over stored results | P0 |
| `wc` | `text.measure` | Native | lines, bytes, tokens, symbols, estimated context cost | P0 |
| `cut` | `data.select` | Native | named fields and schema-aware selection | P1 |
| `sort` | `data.sort` | Native | typed stable sort without text round-trips | P1 |
| `uniq` | `data.unique` | Native | counts, approximate mode for large data | P1 |
| `tr` | `text.translate` | Native | Unicode-safe transformations | P2 |
| `xxd` / `hexdump` | `binary.inspect` | Adapter | bounded ranges, signatures, entropy summary | P3 |
| `strings` | `binary.strings` | Adapter | ranked strings with offsets and budgets | P3 |

New semantic readers:

- `code.outline PATH`: imports, exports, symbols, signatures, line ranges.
- `code.symbol NAME`: definition, references, callers, and callees.
- `code.slice PATH --symbol X`: retrieve a complete semantic unit rather than arbitrary lines.
- `code.context SPAN`: return the smallest useful enclosing construct.
- `config.read PATH`: parse JSON, YAML, TOML, INI, dotenv, and package manifests into typed data.
- `log.inspect REF`: group repeated events, isolate errors, correlate timestamps.
- `doc.extract PATH`: extract headings, tables, metadata, and bounded text from supported documents.

### 6.3 Search and discovery

| Traditional command | OpenTerm command | Strategy | AI-native behavior | Priority |
|---|---|---|---|---|
| `grep` / `rg` | `code.search` | Adapter over ripgrep | groups by file/symbol, ranked previews, full result ref | P0 |
| `locate` | `fs.locate` | Adapter/index | workspace-aware cached index | P2 |
| shell globbing | `fs.match` | Native | explicit patterns and deterministic ordering | P0 |
| `which` | `exec.resolve` | Native | path, version, trust source, adapter support | P1 |
| `whereis` | `exec.inspect` | Native | binary, man page, package origin | P3 |

New search commands:

- `code.search-change`: search only changed lines or files.
- `code.search-related`: find files related by imports, tests, naming, and co-change metadata.
- `code.search-concept`: optional semantic search with local embeddings; results must cite exact spans.
- `code.find-dead`: candidate unused files, exports, dependencies, or configuration.
- `code.find-generated`: detect generated or vendored content and explain why.
- `code.find-secret`: local credential pattern scan with masked results.
- `history.search`: search prior command results and workflows without rerunning them.
- `artifact.search`: query build/test artifacts by type, hash, or producing job.

### 6.4 Text and structured-data transformation

| Traditional command | OpenTerm command | Strategy | AI-native behavior | Priority |
|---|---|---|---|---|
| `sed` | `text.replace` | Native | literal/regex edits, preview, occurrence constraints | P0 |
| `awk` | `data.map` / `data.reduce` | Native | typed expressions, no shell quoting | P1 |
| `jq` | `data.query` | Adapter/library | JSONPath/JMESPath/jq-compatible modes | P0 |
| `yq` | `data.query` | Adapter/library | same query model across JSON/YAML/TOML | P1 |
| `paste` / `join` | `data.join` | Native | schema-aware keyed joins | P2 |
| `split` | `data.chunk` | Native | size/token/record-aware chunks with refs | P1 |
| `diff` | `data.diff` | Native/adapters | semantic modes and bounded hunk summaries | P0 |
| `patch` | `fs.patch` | Native/library | hash preconditions, preview, atomic application | P0 |
| `tee` | `result.save` | Native | materialize a result while retaining provenance | P1 |
| pipes | workflow references | Native | pass typed values and refs, not serialized text | P0 |

Expression language requirements:

- no implicit code execution;
- total, deterministic core functions;
- clear null and error behavior;
- bounded recursion and memory;
- field autocomplete from schemas;
- reusable named transforms;
- escape hatch to WASM or subprocess only with declared capability.

### 6.5 Process and system commands

| Traditional command | OpenTerm command | Strategy | AI-native behavior | Priority |
|---|---|---|---|---|
| arbitrary command | `exec.run` | Adapter | argv arrays, budgets, normalized result, no shell by default | P0 |
| `env` / `export` | `exec.env` | Native | scoped environment overlays, secret refs | P0 |
| `ps` | `proc.list` | Adapter/native | grouped process tree, ownership, resource ranking | P2 |
| `kill` | `proc.signal` | Native | identity verification, graceful escalation plan | P2 |
| `jobs` | `proc.jobs` | Native | persistent job refs and status | P1 |
| `nohup` / `&` | `proc.start` | Native | managed background job with retained output | P1 |
| `top` | `sys.activity` | Adapter | sampled summaries and anomaly deltas | P3 |
| `time` | `exec.measure` | Native | wall/CPU/memory/I/O plus baseline comparison | P1 |
| `timeout` | command budget | Native | universal deadline and cancellation contract | P0 |
| `xargs` | `flow.each` | Native | bounded parallel map over typed items | P1 |
| `watch` | `flow.watch` | Native | delta-only updates and stop conditions | P2 |

`exec.run` must accept an argument array by default:

```yaml
exec.run:
  argv: [npm, test, "--", auth]
  timeout: 2m
  output:
    mode: diagnostics
    token_budget: 1500
```

Shell interpretation should require `shell: true` and a higher-risk capability.

### 6.6 Git and source control

| Traditional command | OpenTerm command | Strategy | AI-native behavior | Priority |
|---|---|---|---|---|
| `git status` | `vcs.status` | Adapter/library | concise state grouped by intent | P0 |
| `git diff` | `vcs.diff` | Adapter | hunk summaries, symbols changed, expansion refs | P0 |
| `git log` | `vcs.history` | Adapter | ranked commits and compact change metadata | P1 |
| `git show` | `vcs.show` | Adapter | commit summary before bounded patch | P1 |
| `git blame` | `vcs.origin` | Adapter | line provenance with commit intent | P2 |
| `git add` | `vcs.stage` | Adapter | explicit file/hunk plan and readback | P1 |
| `git commit` | `vcs.commit` | Adapter | staged-scope validation and generated draft message | P1 |
| `git branch` | `vcs.branch` | Adapter | relationship and worktree awareness | P1 |
| `git merge` / `rebase` | `vcs.integrate` | Adapter | conflict plan; never reimplement algorithms | P3 |
| `git fetch` / `pull` / `push` | `vcs.sync` | Adapter | network effects and target refs made explicit | P2 |

New VCS commands:

- `vcs.impact REF`: changed symbols, affected packages, tests, APIs, and risk areas.
- `vcs.review REF`: structured review packet with bounded diffs and verification evidence.
- `vcs.intent`: compare requested task, current diff, and tests to identify scope drift.
- `vcs.change-map`: dependency graph centered on changed files.
- `vcs.checkpoint`: lightweight reversible workspace snapshot without pretending it is a commit.

### 6.7 Build, test, and diagnostics

Do not create one wrapper per build tool initially. Create a common adapter contract.

- `project.detect`: identify languages, packages, build tools, test tools, and entry points.
- `project.tasks`: list normalized tasks from package manifests and config.
- `build.run`: execute a build and return artifact refs plus grouped diagnostics.
- `test.run`: execute selected tests and return pass/fail/skip counts, failures first, raw log ref.
- `test.select`: predict impacted tests using dependency and history data.
- `lint.run`: normalize diagnostics across linters.
- `format.check`: report files and ranges; `format.apply` mutates with preview support.
- `typecheck.run`: group errors by root cause and suppress cascades in the compact view.
- `diagnostic.explain`: join a diagnostic with source span, related symbols, and configuration.
- `diagnostic.delta`: show only new, fixed, or changed diagnostics relative to a baseline.
- `benchmark.run`: repeated measurements with variance and baseline comparison.

Initial adapters: npm/pnpm/yarn, pytest, Cargo, Go, Make, common TypeScript tools. Add ecosystems through plugins, not core conditionals.

### 6.8 Archives and transfer

| Traditional command | OpenTerm command | Strategy | AI-native behavior | Priority |
|---|---|---|---|---|
| `tar` / `zip` | `archive.create` | Library/adapter | manifest preview, deterministic mode, artifact ref | P2 |
| `unzip` | `archive.extract` | Library | zip-slip protection, collision preview, budgets | P2 |
| `gzip` / `zstd` | `artifact.compress` | Library | content type and ratio metadata | P3 |
| `rsync` | `transfer.sync` | Adapter | dry-run default, transfer plan and deletion warnings | P2 |
| `scp` | `remote.copy` | SSH adapter | checksums, resume, explicit host/path roles | P2 |

### 6.9 Network and HTTP

| Traditional command | OpenTerm command | Strategy | AI-native behavior | Priority |
|---|---|---|---|---|
| `curl` / `wget` | `net.request` | HTTP library | typed request, bounded body, headers/redaction, artifact refs | P1 |
| `ping` | `net.reach` | Adapter | sampled latency summary | P3 |
| `dig` / `nslookup` | `net.dns` | Library | typed records and resolution chain | P3 |
| `nc` | `net.probe` | Adapter | tightly gated, destination-scoped | P3 |
| `lsof` | `net.listeners` | Adapter | process/listener relationship | P3 |

`net.request` needs:

- destination allowlists;
- secret headers represented by secret refs;
- response-size and redirect budgets;
- MIME-aware preview;
- automatic storage of large/binary bodies;
- reproducible request records;
- certificate verification on by default;
- no implicit credential forwarding across redirects.

### 6.10 SSH and remote execution

SSH is an **adapter-only** area. Use OpenSSH or a mature library. Never implement cryptography or host verification.

- `remote.hosts`: list configured opaque host profiles.
- `remote.check HOST`: connectivity, identity, platform, and capabilities.
- `remote.run HOST -- COMMAND`: argv-safe remote execution with local result normalization.
- `remote.read HOST PATH`: bounded remote file read.
- `remote.copy`: checksum-verified upload/download.
- `remote.sync`: previewed rsync-style synchronization.
- `remote.tunnel`: explicit, time-bounded managed tunnel.
- `remote.batch`: bounded concurrency across hosts with per-host results.
- `remote.snapshot`: collect a declared diagnostic bundle.

Security requirements:

- preserve strict host-key verification;
- use the user's agent/keychain instead of storing private keys;
- never return private key or token contents;
- redact remote environment secrets;
- distinguish local and remote paths in schemas;
- require explicit target hosts, commands, and timeouts;
- retain per-host raw logs and exit status;
- prevent accidental fan-out to production host groups.

### 6.11 Databases

| Traditional command | OpenTerm command | Strategy | AI-native behavior | Priority |
|---|---|---|---|---|
| `psql` / `mysql` / `sqlite3` | `db.query` | Drivers/adapters | typed rows, pagination, read-only default | P2 |
| dump tools | `db.snapshot` | Adapter | schema/data manifest, encrypted artifact | P3 |
| migration tools | `db.migrate` | Adapter | plan, lock/risk metadata, approval gate | P3 |

Database commands must enforce row, byte, and execution-time budgets. Mutating statements require a declared write mode; destructive schema operations require an additional capability. Query summaries must never silently replace exact values needed for correctness.

### 6.12 Package managers and dependencies

- `dep.list`: normalized dependency graph.
- `dep.why NAME`: explain direct and transitive inclusion.
- `dep.outdated`: normalized update candidates with compatibility metadata.
- `dep.add` / `dep.remove`: manifest and lockfile change preview.
- `dep.audit`: normalize security advisories while retaining source evidence.
- `dep.license`: summarize license obligations and unknowns.
- `dep.size`: installed, packaged, and transitive size impact.

Use native package managers for lockfile calculation and installation. OpenTerm owns planning, policy, normalization, and diff reporting.

### 6.13 Containers and cloud CLIs

Start with generic adapters rather than rebuilding Docker, Kubernetes, or cloud SDKs:

- `container.list`, `container.logs`, `container.exec`, `container.build`, `container.inspect`;
- `cluster.context`, `cluster.resources`, `cluster.logs`, `cluster.apply`;
- `cloud.call` as a schema-aware adapter to installed provider CLIs.

Mutations must show current context, account, region, namespace, resource identifiers, and expected effect. Production contexts should be policy-gated.

---

## 7. New AI-native commands

These commands are the real differentiator; they should not merely rename Unix tools.

### 7.1 Context control

#### `result.compact REF`

Create a smaller view using deterministic grouping and relevance rules.

#### `result.expand REF`

Expand one group, record range, diagnostic, hunk, or semantic span.

#### `result.query REF EXPR`

Query stored results without rerunning the original operation.

#### `result.compare REF_A REF_B`

Return only changes between command results, builds, tests, logs, or snapshots.

#### `context.pack`

Build an explicit context bundle from file spans, results, diagnostics, and task metadata, deduplicate it, estimate tokens, and emit a manifest.

#### `context.budget`

Allocate a token budget across categories such as source, diagnostics, history, and tests.

#### `context.explain`

Explain why each included item was selected and what was omitted.

### 7.2 Workspace understanding

#### `workspace.map`

Return a compact architectural map: packages, entry points, services, data stores, configs, generated areas, tests, and dependency edges.

#### `workspace.delta SNAP`

Return files, symbols, diagnostics, and tests changed since a snapshot.

#### `workspace.rules`

Discover applicable instruction files, formatting rules, language versions, and nested scope overrides.

#### `workspace.focus TASK`

Produce a ranked working set for a task with evidence and exact source spans.

### 7.3 Planning and effects

#### `action.plan`

Resolve a workflow into concrete steps, capabilities, estimated cost, and mutations without executing it.

#### `action.preview`

Run read-only preparation and produce exact proposed side effects.

#### `action.apply PLAN_REF`

Execute an unchanged, hash-bound plan.

#### `action.undo JOB_REF`

Apply recorded inverse operations where supported; state clearly when an operation is not reversible.

#### `action.verify JOB_REF`

Run postconditions and report evidence rather than a generic success message.

### 7.4 Failure intelligence

#### `failure.root REF`

Collapse repeated/cascading errors and rank probable root diagnostics using deterministic relationships.

#### `failure.reproduce REF`

Create a minimal reproducible command, environment manifest, and relevant inputs.

#### `failure.delta BASE CURRENT`

Identify newly introduced failures and resolved ones.

#### `failure.flaky TEST_REF`

Repeat under a bounded policy and report observed variance; never declare flakiness from one run.

### 7.5 Memory and history

#### `memory.note`

Store a small, user-approved workspace fact with evidence and expiry.

#### `memory.recall`

Retrieve facts relevant to the current workspace, branch, or task.

#### `memory.invalidate`

Expire facts when source hashes or configurations change.

Memory must be inspectable, scoped, and deletable. It must not become a hidden source of instructions.

---

## 8. Low-code workflow language

### 8.1 Minimum syntax

Support:

- typed inputs and outputs;
- command invocation;
- references between steps;
- `if` and `match` conditions;
- bounded `each` and parallelism;
- retry policies for declared transient errors;
- time/token/item budgets;
- preview and approval boundaries;
- error handlers and cleanup;
- reusable modules;
- assertions and postconditions.

Avoid general-purpose loops, arbitrary evaluation, and implicit shell execution in v1.

### 8.2 Example

```yaml
apiVersion: openterm.dev/v1
kind: Workflow
metadata:
  name: diagnose-test-failure
inputs:
  test: { type: string }
budgets:
  duration: 5m
  returned_tokens: 4000
capabilities:
  filesystem: { read: ["./"] }
  process: { commands: ["npm"] }
steps:
  run:
    use: test.run
    with: { select: "${inputs.test}" }
  changed:
    use: vcs.diff
    with: { mode: symbols }
  focus:
    use: workspace.focus
    with:
      task: "Explain ${inputs.test} failure"
      evidence: ["${run.ref}", "${changed.ref}"]
  pack:
    use: context.pack
    with:
      include: ["${run.failures}", "${changed.hunks}", "${focus.spans}"]
      token_budget: 3000
outputs:
  context: "${pack.ref}"
  raw_test_log: "${run.raw_ref}"
```

### 8.3 Workflow trust

Workflow files are executable policy objects. They should support:

- content hashes and optional signatures;
- declared capability manifests;
- static validation;
- `plan` mode;
- locked dependency/plugin versions;
- provenance for imported modules;
- organization allow/deny rules.

---

## 9. Token-efficiency system

### 9.1 Budgets

Every command should support some combination of:

- `max_items`;
- `max_bytes`;
- `max_tokens`;
- `max_groups`;
- `context_before` / `context_after`;
- `timeout`;
- `max_parallel`;
- `max_depth`;
- `summary_mode`.

The runtime should use a tokenizer appropriate to the consuming model when known, and a documented approximation otherwise.

### 9.2 Compaction policies by data type

- **Search**: group by file and symbol; rank exact identifier matches first.
- **Logs**: preserve errors, first/last events, time gaps, and repetition counts.
- **Tests**: counts first, then failures, then slow tests, with passing detail omitted.
- **Diffs**: file summaries and changed symbols before hunks.
- **Trees**: collapse single-child chains and generated/vendor subtrees.
- **Tabular data**: schema, row count, null/cardinality summaries, then sampled rows.
- **Processes**: group by executable and parent; rank resource outliers.
- **HTTP**: status and selected headers; body preview according to MIME.

### 9.3 Cache correctness

Cache keys must include:

- normalized command and arguments;
- relevant file content hashes, not only timestamps;
- executable and adapter version;
- environment keys explicitly declared as inputs;
- remote host identity when applicable;
- schema and compaction-policy version.

Never cache mutations as if they were reads. A compact view can be regenerated from a stored raw result without rerunning the command.

### 9.4 Token-savings measurement

Record:

- raw bytes and estimated tokens;
- compact bytes and estimated tokens;
- number of expansions requested;
- repeated reads avoided by references;
- percentage of compact results sufficient without expansion;
- latency and storage overhead;
- correctness regressions caused by omitted context.

Token reduction is not success if agents make worse decisions. Benchmarks must measure task completion and error rate alongside compression.

---

## 10. Safety model

### 10.1 Effect classes

Each command declares one of:

- `pure`: transforms supplied data only;
- `read-local`;
- `write-local`;
- `execute-local`;
- `read-network`;
- `write-network`;
- `remote-execute`;
- `privileged`;
- `destructive`.

Commands may have multiple effects. The runtime must compute transitive effects for workflows.

### 10.2 Mutation protocol

For important writes:

1. inspect current state;
2. create a hash-bound plan;
3. show a compact preview and exact effect metadata;
4. apply atomically where possible;
5. read back the result;
6. run declared verification;
7. store inverse information when feasible.

### 10.3 Secrets

- Use OS keychains or external secret providers.
- Pass opaque references to commands.
- Redact exact and encoded forms in output where practical.
- Do not persist secrets in workflow files, result stores, logs, or telemetry.
- Track which process or request received a secret without storing its value.
- Require explicit destination binding for network credentials.

### 10.4 Untrusted repositories

Reading a repository must not execute it. Discovery should distinguish static inspection from project-controlled commands. Package scripts, build hooks, editor configs, and instruction files are untrusted inputs, not authority.

### 10.5 Result integrity

Summaries should include:

- source result reference;
- summary algorithm/version;
- omitted counts;
- evidence pointers;
- whether generative summarization was used.

Agents must be able to request exact raw evidence for any claim.

---

## 11. Plugin system

Core should stay small. Plugins may add:

- commands and schemas;
- executable adapters;
- parsers and diagnostic normalizers;
- project detectors;
- compaction policies;
- workflow templates;
- policy rules;
- renderers.

Plugin requirements:

- manifest with version and runtime compatibility;
- declared capabilities;
- deterministic installation and lockfile;
- isolated process or WASM execution where possible;
- schema namespace ownership;
- signed provenance for distributed plugins;
- test fixtures and conformance suite.

Do not allow plugins to silently inherit every runtime permission.

---

## 12. Implementation roadmap

### Phase 0 — Research and specification (2–3 weeks)

Deliverables:

- collect 30–50 real agent terminal transcripts;
- measure output volume, repeated reads, latency, and common failures;
- define `openterm.result/v1` and error schemas;
- define reference lifecycle and workspace identity;
- choose implementation language through a spike;
- create threat model and effect taxonomy;
- build a benchmark corpus of representative repositories;
- write 10 golden end-to-end scenarios.

Decision gate:

- at least five common tasks show a credible path to >60% context reduction without loss of necessary evidence.

Recommended language choice:

- **Rust** for the long-term portable runtime, safety, single binary, and process control; or
- **TypeScript** for the fastest initial low-code/plugin iteration.

A pragmatic route is a TypeScript prototype with a strict protocol, followed by selective Rust components only when profiling or distribution justifies them. Do not start with a multi-language architecture.

### Phase 1 — Token-aware read-only MVP (4–6 weeks)

Implement:

- CLI and JSON output mode;
- command registry and schema validation;
- SQLite/blob result store;
- `fs.where`, `fs.list`, `fs.tree`, `fs.find`, `fs.read`, `fs.peek`;
- `text.measure`;
- `code.search` using ripgrep;
- `code.outline` for 2–3 languages using tree-sitter or language parsers;
- `vcs.status`, `vcs.diff`;
- `exec.run` with argv, timeout, and output budgets;
- `result.page`, `result.query`, `result.expand`, `result.compact`;
- ANSI stripping, repetition collapse, grouping, and raw-result retention;
- filesystem read capability boundaries;
- metrics for raw versus returned tokens.

MVP demonstration:

1. map an unfamiliar repository;
2. locate a feature implementation;
3. inspect exact relevant symbols;
4. run one test command;
5. identify a failure;
6. complete the flow with fewer than 25% of the raw terminal tokens.

Exit criteria:

- stable result schema across all P0 read commands;
- no unexplained truncation;
- every compact result offers a working expansion path;
- tested on macOS and Linux;
- benchmark shows ≥60% median context reduction and no material task-completion regression.

### Phase 2 — Safe editing and workflows (4–6 weeks)

Implement:

- `text.replace`, `fs.patch`, `data.diff`;
- file hash preconditions and atomic writes;
- `action.plan`, `action.preview`, `action.apply`, `action.verify`;
- YAML workflow runner;
- typed step references, conditions, bounded `each`, and retries;
- effect calculation and capability enforcement;
- workspace snapshots and mutation audit trail;
- `project.detect`, `project.tasks`;
- `test.run`, `lint.run`, `typecheck.run` adapter contract;
- `diagnostic.delta`;
- VS Code/editor-friendly result links if desired.

Exit criteria:

- stale patches fail safely;
- interrupted writes do not leave partial files;
- workflows are statically validated before execution;
- mutation results include readback and verification evidence;
- an agent can implement and verify a small code change end to end.

### Phase 3 — Code intelligence and impact analysis (5–8 weeks)

Implement:

- language plugin SDK;
- symbol definitions and references;
- import/dependency graph;
- `code.symbol`, `code.slice`, `code.context`;
- `workspace.map`, `workspace.focus`, `workspace.delta`;
- `vcs.impact`, `vcs.change-map`, `test.select`;
- result comparison and baseline support;
- local optional semantic index with exact-span citations;
- `context.pack`, `context.budget`, `context.explain`.

Start with TypeScript/JavaScript, Python, Rust, and Go only if demand justifies all four. Depth in two ecosystems is better than shallow support for ten.

Exit criteria:

- symbol-level retrieval outperforms line-window retrieval on benchmark tasks;
- changed-file impact packets fit within declared budgets;
- index invalidation is correct after edits and branch changes.

### Phase 4 — Managed processes, Git writes, and HTTP (4–6 weeks)

Implement:

- persistent background job manager;
- `proc.start`, `proc.jobs`, `proc.signal`, delta-based log watching;
- `vcs.stage`, `vcs.commit`, guarded `vcs.sync`;
- `net.request` with secret references and body artifact storage;
- package-manager adapters;
- artifact registry and retention policies;
- approval boundaries for external effects.

Exit criteria:

- processes survive client disconnects and remain inspectable;
- Git writes validate target repository and expected state;
- HTTP secrets cannot appear in stored output in tested cases;
- repeated polling returns deltas rather than complete logs.

### Phase 5 — Remote execution and databases (6–8 weeks)

Implement:

- OpenSSH adapter and host profiles;
- `remote.check`, `remote.run`, `remote.read`, `remote.copy`, `remote.batch`;
- strict host-key and target policy;
- database driver interface;
- read-only `db.query` with pagination and row budgets;
- guarded database writes and transaction support;
- remote and database audit records.

Exit criteria:

- remote identity is explicit in every result;
- multi-host operations isolate failures per host;
- connection secrets never enter command result payloads;
- database read-only mode is enforced by driver/session policy where supported.

### Phase 6 — Ecosystem and production hardening (ongoing)

Implement:

- signed plugin registry;
- organization policies;
- encrypted/shared result stores where required;
- MCP/API server and client SDKs;
- Windows support if demanded;
- container/Kubernetes/cloud adapters;
- observability dashboards;
- compatibility and migration policy;
- fuzzing for parsers, paths, archives, and result envelopes;
- supply-chain review and reproducible builds.

---

## 13. Prioritized command backlog

### P0: prove the thesis

- `fs.where`, `fs.focus`, `fs.list`, `fs.tree`, `fs.find`;
- `fs.read`, `fs.peek`, `text.measure`;
- `code.search`, `code.outline`;
- `exec.run`;
- `vcs.status`, `vcs.diff`;
- `data.query`, `data.diff`;
- `result.page`, `result.expand`, `result.query`, `result.compact`;
- `fs.patch`, `text.replace`;
- `action.plan`, `action.apply`, `action.verify`.

### P1: complete local coding workflows

- project/test/lint/typecheck adapters;
- managed jobs and delta logs;
- config readers;
- package dependency inspection;
- Git history, staging, and commits;
- HTTP requests;
- context packing;
- workspace mapping and focus;
- structured workflow runner.

### P2: team and infrastructure workflows

- remote execution and transfer;
- read-only database access;
- archive handling;
- process/system inspection;
- containers;
- dependency audits;
- Git network synchronization.

### P3: specialized administration

- privileged filesystem operations;
- network probes and tunnels;
- database migrations;
- Kubernetes/cloud mutations;
- advanced binary inspection;
- broad fleet orchestration.

---

## 14. Testing strategy

### 14.1 Unit and property tests

- schema validation;
- path boundary enforcement;
- command argument encoding;
- hash and cache invalidation;
- truncation accounting;
- deterministic compaction;
- secret redaction;
- expression-language limits;
- patch preconditions and inverses.

### 14.2 Golden tests

Store fixtures for:

- search grouping;
- Git diff normalization;
- common compiler and test outputs;
- ANSI/progress cleanup;
- repeated log collapse;
- partial and invalid Unicode;
- huge lines and binary files;
- filenames with spaces, newlines, Unicode, and leading dashes;
- symlink loops and workspace escapes.

### 14.3 Integration tests

- macOS and Linux;
- shallow/full Git repositories and worktrees;
- interrupted processes;
- stale file hashes;
- offline mode;
- full disks and permission failures;
- SSH host-key mismatch;
- HTTP redirects and oversized bodies;
- database timeouts and transaction rollback.

### 14.4 Adversarial tests

- prompt-like text in repository files;
- malicious terminal escape sequences;
- archive traversal;
- command injection through filenames;
- environment-secret exfiltration;
- symlink races;
- output designed to defeat repetition collapse;
- plugin capability escalation;
- result-reference substitution across workspaces.

### 14.5 Agent benchmarks

For each scenario compare a normal shell baseline with OpenTerm:

- task completion rate;
- total tokens returned by tools;
- number of tool calls;
- wall-clock time;
- erroneous mutations;
- unnecessary file reads;
- expansions after compact output;
- human rating of trace inspectability.

Benchmark tasks should include bug localization, small feature work, test diagnosis, dependency investigation, log analysis, and remote incident inspection.

---

## 15. Metrics and product success

Primary metrics:

- median and p95 returned-token reduction;
- successful task completion relative to baseline;
- percentage of results not requiring expansion;
- average repeated reads avoided;
- mutation verification success;
- unsafe action prevention rate;
- adapter normalization accuracy.

Guardrail metrics:

- failures caused by omitted context;
- stale-cache incidents;
- redaction failures;
- result-store disk growth;
- added latency;
- nondeterministic output changes;
- raw-command escape-hatch frequency.

Suggested initial targets:

- ≥70% median output-token reduction for search, test, diff, and log workflows;
- ≥95% of compact claims traceable to exact stored evidence;
- <5% of commands require the raw shell escape hatch in supported workflows;
- zero silent truncations;
- zero writes without declared effects and readback.

---

## 16. Key risks and mitigations

### Risk: rebuilding too much

Mitigation: maintain the native/adapter/avoid classification; require an architecture decision record before replacing a mature tool.

### Risk: summaries hide crucial evidence

Mitigation: deterministic summaries, omission counts, raw refs, easy expansion, and correctness benchmarks.

### Risk: low-code becomes another programming language

Mitigation: keep v1 declarative and bounded; use plugins or normal code for complex logic.

### Risk: result storage becomes large or sensitive

Mitigation: TTLs, quotas, content deduplication, encryption options, redaction, workspace-local stores, and explicit purge commands.

### Risk: cross-platform behavior diverges

Mitigation: normalized contract with adapter metadata; test differences rather than pretending they do not exist.

### Risk: agents over-trust generated summaries

Mitigation: label generative content, keep deterministic facts separate, and require evidence pointers.

### Risk: remote features create a dangerous automation surface

Mitigation: mature transports, host identity, narrow capabilities, preview, environment labels, concurrency limits, and audit logs.

### Risk: performance overhead cancels usefulness

Mitigation: stream raw data once, index incrementally, cache views, benchmark every compaction stage, and allow a compact passthrough mode.

---

## 17. Recommended repository structure

```text
OpenTerm/
├── cli/                 # command-line and REPL interface
├── runtime/             # registry, execution, budgets, effects
├── protocol/            # schemas, references, errors, versioning
├── store/               # metadata and blob storage
├── policy/              # capability and approval engine
├── compact/             # deterministic compaction policies
├── commands/
│   ├── fs/
│   ├── code/
│   ├── data/
│   ├── exec/
│   ├── result/
│   └── vcs/
├── workflow/            # low-code parser and runner
├── adapters/            # ripgrep, git, build/test tools
├── plugins/             # SDK and first-party plugins
├── benchmarks/          # shell-vs-openterm task corpus
├── fixtures/            # golden outputs and adversarial cases
├── docs/
│   ├── architecture.md
│   ├── protocol.md
│   ├── security.md
│   └── command-design.md
└── roadmap.md
```

---

## 18. First 30 implementation issues

1. Specify the v1 result envelope.
2. Specify normalized error codes and retryability.
3. Specify effect and capability schemas.
4. Specify result-reference identity, TTL, and workspace binding.
5. Create benchmark transcript collection format.
6. Select language via a CLI/process/result-store spike.
7. Implement command registry and schema validation.
8. Implement SQLite metadata store and compressed blob store.
9. Implement output byte/token/item budget accounting.
10. Implement ANSI and carriage-return progress normalization.
11. Implement repeated-line collapse with lossless refs.
12. Implement `fs.where` and workspace discovery.
13. Implement path normalization and scope enforcement.
14. Implement `fs.list` and `fs.tree`.
15. Implement `fs.find` with ignore-file support.
16. Implement `fs.read` and binary detection.
17. Implement `fs.peek` and `text.measure`.
18. Implement ripgrep adapter for `code.search`.
19. Implement grouped/ranked search compaction.
20. Implement `result.page` and cursors.
21. Implement `result.query` and derived views.
22. Implement `result.expand` for groups and spans.
23. Implement argv-safe `exec.run` with timeout/cancellation.
24. Implement Git repository detection and `vcs.status`.
25. Implement `vcs.diff` summaries and hunk refs.
26. Add tree-sitter spike for `code.outline`.
27. Add telemetry that remains local by default.
28. Build five shell-versus-OpenTerm golden scenarios.
29. Fuzz paths, unusual filenames, and output normalizers.
30. Publish the first measured token-efficiency report.

---

## 19. MVP command examples

```sh
# Compact workspace overview
ai-tree --depth 4 --tokens 800

# Search with full output retained outside model context
ai-grep --query 'authenticate' --scope src --tokens 1000

# Inspect one group from the previous result
ai-expand res_01J... --group 2

# Read complete semantic units fitting a budget
ai-read src/auth.ts --symbol authenticate --tokens 1200

# Run tests while returning only normalized diagnostics
ai-test --select auth --output diagnostics --tokens 1500

# Preview a hash-guarded edit
ai-replace --file src/auth.ts --literal oldName --with newName --preview

# Apply and verify the exact previewed plan
ai-apply plan_01J...
ai-verify job_01J...
```

---

## 20. Final recommendation

This is a good idea if OpenTerm is positioned as a **token-aware, typed orchestration layer for AI agents**. It is a poor idea if the goal is to independently rewrite every terminal utility.

The moat is not a shorter spelling of `grep`. It is the shared protocol connecting:

- bounded structured results;
- lossless references and progressive expansion;
- semantic code and diagnostic views;
- safe, hash-bound mutation plans;
- declarative workflows;
- capability-based local and remote execution;
- measurable context savings without correctness loss.

Begin with the read-only P0 slice. Prove that agents can solve real repository tasks using materially fewer tokens. Only then add editing, workflows, remote systems, and broad ecosystem adapters. That sequence keeps the project testable, useful early, and safely differentiated from a conventional shell.

---

## 21. Product expansion: from tool suite to complete terminal

The complete product should feel familiar enough to replace a developer's daily terminal while exposing a second, structured plane designed for agents.

It has five product surfaces:

1. **Terminal surface** — tabs, panes, scrollback, search, links, selection, command blocks, themes, keybindings, notifications, and accessibility.
2. **Session surface** — persistent local and remote sessions, detach/reattach, process lifecycle, workspace identity, snapshots, and collaboration.
3. **AI surface** — agent chat, plans, tool calls, context packs, result expansion, approvals, diffs, diagnostics, and evidence.
4. **Automation surface** — low-code workflows, triggers, schedules, reusable commands, and policy-bound actions.
5. **Extension surface** — plugins, themes, completions, renderers, agents, adapters, policies, and integrations.

The terminal and AI must not be two unrelated applications placed side by side. A command typed by a human should produce the same structured execution record as a command started by an agent. An agent action should remain visible, inspectable, interruptible, and reproducible from the terminal.

### 21.1 The dual-plane model

Every session has two synchronized planes:

```text
┌──────────────────────────── terminal session ────────────────────────────┐
│                                                                          │
│  byte plane                              semantic plane                   │
│  ──────────                              ──────────────                   │
│  PTY input/output                        command start/end                │
│  VT escape sequences     ───────────▶    cwd/environment deltas          │
│  interactive TUIs                        exit status/duration             │
│  arbitrary programs      ◀───────────    files/artifacts/diagnostics     │
│  exact compatibility                    result refs/context summaries    │
│                                                                          │
└──────────────────────────────────────────────────────────────────────────┘
```

The byte plane preserves compatibility with shells, editors, REPLs, and TUIs. The semantic plane makes sessions useful to agents. Neither can replace the other.

### 21.2 Core product promises

- Existing shell commands continue to work.
- Users can keep zsh, bash, fish, nushell, or another shell.
- AI features enhance the shell; they do not silently reinterpret it.
- Every command becomes a navigable block with status and provenance.
- Large output remains available without automatically consuming model context.
- Sessions survive UI restarts.
- Plugins cannot silently gain terminal, filesystem, network, or secret access.
- Themes and keybindings are data, not arbitrary startup code.
- AI is optional for basic terminal operation.
- Local/offline workflows remain first-class.

---

## 22. Architecture decision: tmux, an existing terminal, or from scratch?

### 22.1 Decision

Use a **hybrid architecture**:

- do not fork or embed tmux as the permanent application core;
- support tmux as a compatibility backend and import path;
- build an independent OpenTerm session daemon and structured event protocol;
- embed a mature VT terminal engine rather than writing a terminal emulator from zero;
- use OS PTY facilities or a mature cross-platform PTY library;
- keep the user's normal shell as the compatibility execution engine;
- implement AI-native commands and workflows in the OpenTerm runtime.

This keeps the unique part—the semantic, agent-aware session model—under project control while reusing decades of terminal correctness work.

### 22.2 Why tmux should not be the permanent core

tmux already provides:

- windows and panes;
- detach and reattach;
- long-running server sessions;
- input routing;
- scrollback and copy mode;
- scripting and control mode;
- broad Unix availability.

It is useful for:

- an early headless prototype;
- remote servers where only SSH and tmux are installed;
- importing or attaching to existing developer sessions;
- a fallback when the native session daemon is unavailable;
- testing session semantics before building the UI.

It is limiting as the permanent foundation because OpenTerm needs first-class concepts tmux does not own:

- typed command events and result references;
- model-context budgets;
- per-command provenance and artifacts;
- plugin capability isolation;
- semantic history and workspace memory;
- UI-native result cards, diffs, diagnostics, and approval panels;
- structured synchronization between terminal blocks and agents;
- portable remote session protocol beyond screen scraping;
- fine-grained replay without treating all output as one character grid.

An adapter can infer some of this around tmux, but the architecture becomes a stack of heuristics. Keep tmux integration; do not make every future feature negotiate through it.

### 22.3 Why not build everything from zero

A correct terminal emulator must handle decades of VT behavior, Unicode, grapheme clusters, width calculation, bidirectional text concerns, colors, hyperlinks, images, mouse protocols, bracketed paste, alternate screens, resizing, and hostile escape sequences. This work is important but not differentiating for OpenTerm.

Likewise, shells already own parsing, expansion, job control, signals, functions, aliases, and compatibility. Replacing zsh/bash/fish in the first product would turn one ambitious project into three.

Build from zero only where the project creates unique value:

- structured session daemon;
- command-event protocol;
- result and artifact store;
- agent context manager;
- permission and approval layer;
- plugin host;
- low-code workflow engine;
- semantic history;
- AI-native UI.

### 22.4 Technology paths to prototype

#### Path A — Native app with libghostty-style VT embedding

Best long-term product path if the embedding API proves sufficiently stable.

- Rust or Zig/Rust core;
- embedded mature VT engine;
- native or GPU UI;
- OpenTerm daemon over local IPC;
- user shell in a PTY;
- strongest control over command blocks and AI panels.

Advantages: performance, native interaction, full product control.  
Risks: embedding API maturity, UI engineering, platform packaging.

#### Path B — Web UI with xterm.js-style frontend

Best validation path for rapid iteration.

- Rust/TypeScript daemon;
- desktop shell such as Tauri or a browser client;
- web terminal component;
- UI panels built with normal web technologies.

Advantages: fast UI/plugin experimentation, accessible component ecosystem.  
Risks: rendering overhead, native integration limits, browser security boundary complexity.

#### Path C — tmux control-mode client

Best short-lived prototype for session persistence.

- tmux owns PTYs and panes;
- OpenTerm consumes control-mode events;
- thin desktop/TUI client adds agent panels and result store.

Advantages: shortest route to persistent sessions.  
Risks: semantic reconstruction, portability, long-term architectural ceiling.

#### Path D — WezTerm-style integrated multiplexer

Use as an architectural reference: a GUI attaches to daemon-managed domains. Do not depend on private/internal APIs unless the maintenance commitment is acceptable.

### 22.5 Recommended sequence

1. Build the semantic protocol and documentation agent in a headless CLI.
2. Connect it to current terminals through shell integration.
3. Build a tmux-backed experimental session UI.
4. In parallel, spike two embedded VT options with the same daemon protocol.
5. Select the native UI path using measured correctness, latency, accessibility, and packaging results.
6. Keep the daemon protocol stable so the prototype UI can be replaced without rewriting the AI stack.

### 22.6 Technology selection scorecard

Score each candidate from 1–5:

| Criterion | Weight |
|---|---:|
| VT correctness and compatibility | 20% |
| embedding API stability | 15% |
| input latency and rendering performance | 10% |
| macOS and Linux support | 10% |
| accessibility and IME support | 10% |
| command-block integration | 10% |
| maintainability and license | 10% |
| packaging and binary size | 5% |
| image/hyperlink/clipboard protocols | 5% |
| Windows path, if required | 5% |

Do not choose on aesthetics or benchmark FPS alone. Terminal correctness and input behavior are product foundations.

---

## 23. Complete system architecture

### 23.1 Process topology

```text
┌──────────────── OpenTerm Desktop / TUI / Editor Client ────────────────┐
│ terminal renderer │ command blocks │ agent panel │ docs │ settings     │
└──────────────────────────────┬──────────────────────────────────────────┘
                               │ versioned local protocol
┌──────────────────────────────▼──────────────────────────────────────────┐
│ OpenTerm Session Daemon                                                │
│ sessions │ panes │ PTYs │ event log │ scrollback │ jobs │ attachments  │
├─────────────────────────────────────────────────────────────────────────┤
│ Semantic Runtime                                                       │
│ command registry │ workflows │ result store │ policies │ context packs │
├─────────────────────────────────────────────────────────────────────────┤
│ Plugin Host                                                            │
│ agents │ tools │ parsers │ completions │ themes │ integrations          │
└──────────┬──────────────────────┬───────────────────┬────────────────────┘
           │                      │                   │
      user shells            OS / PTYs         optional services
     zsh/bash/fish      files/git/processes    models/SSH/Drive/etc.
```

### 23.2 Client responsibilities

- terminal rendering and input;
- pane/tab/window composition;
- command-block presentation;
- search and navigation;
- agent conversation and plan views;
- diff and approval UI;
- theme and font rendering;
- accessibility tree;
- notifications;
- client-local clipboard and keybindings.

The client must not directly own critical session state. A crash or upgrade should not terminate running commands.

### 23.3 Session daemon responsibilities

- create and manage PTYs;
- track sessions, windows, panes, and active workspaces;
- retain ordered input/output/event logs;
- detach and reattach clients;
- manage scrollback and terminal checkpoints;
- route resize, signal, and input events;
- detect command boundaries through shell integration;
- associate artifacts and results with command blocks;
- expose a versioned local protocol;
- enforce client and plugin permissions;
- recover after client crashes.

### 23.4 Semantic runtime responsibilities

- run AI-native commands;
- normalize traditional command output;
- create compact views and references;
- execute low-code workflows;
- calculate effects and request approvals;
- build model context packs;
- coordinate agents and tools;
- index workspace data;
- store provenance, metrics, and verification evidence.

### 23.5 Storage layout

Use separate stores with different retention and sensitivity:

- **session database**: topology, pane metadata, command blocks, timestamps;
- **event segments**: append-only compressed PTY and semantic events;
- **result store**: typed command results and derived views;
- **artifact store**: files, logs, reports, builds, screenshots;
- **history index**: searchable command and outcome metadata;
- **workspace index**: symbols, docs, dependencies, hashes;
- **settings database**: user configuration and plugin state;
- **secret references**: identifiers only; values remain in keychain/provider.

All stores need quotas, retention, purge, export, and encryption policy hooks.

### 23.6 Local protocol

The daemon protocol should support:

- request/response calls;
- ordered event streams;
- binary attachments;
- resumable subscriptions by sequence number;
- client capability negotiation;
- protocol-version negotiation;
- backpressure;
- cancellation;
- authentication on local/remote transports;
- explicit workspace and session IDs.

Start with a simple framed protocol over Unix domain sockets. JSON is acceptable for control messages; use binary frames or blob references for terminal data and artifacts. Do not put high-volume PTY bytes into repeated JSON base64 payloads.

### 23.7 Event model

Important event types:

- `session.created`, `session.attached`, `session.closed`;
- `pane.created`, `pane.resized`, `pane.focused`, `pane.closed`;
- `pty.output`, `pty.input`, `pty.checkpoint`;
- `command.started`, `command.updated`, `command.finished`;
- `cwd.changed`, `environment.changed`;
- `artifact.created`, `diagnostic.detected`;
- `result.created`, `result.view-created`;
- `agent.started`, `agent.tool-called`, `agent.waiting`, `agent.finished`;
- `approval.requested`, `approval.resolved`;
- `plugin.loaded`, `plugin.failed`, `policy.denied`.

Events require monotonic sequence IDs, wall-clock timestamps, producer identity, parent correlation IDs, and schema versions.

---

## 24. Shell integration without replacing zsh

### 24.1 Strategy

OpenTerm should integrate with existing shells through a very small, auditable hook package. It should not require users to abandon their aliases, functions, environment managers, or completion definitions.

Initial supported shells:

1. zsh;
2. bash;
3. fish;
4. nushell after the byte/structured boundary is understood.

### 24.2 Shell hook events

The integration reports:

- prompt ready;
- command buffer before execution;
- command start;
- command end and exit status;
- current working directory;
- selected environment changes;
- shell nesting/subshell state;
- optional command-specific completion context.

Use documented shell hooks and terminal escape sequences where possible. The shell hook must never export all environment variables or secrets to the daemon.

### 24.3 Compatibility modes

- **Transparent mode**: normal shell with command-block metadata.
- **Enhanced mode**: smart completions, history ranking, result capture, and AI actions.
- **Structured mode**: OpenTerm native commands and workflows return typed results.
- **Safe mode**: plugins and AI disabled; terminal remains usable.
- **Remote mode**: reduced shell integration over SSH when the helper is available; screen-only fallback otherwise.

### 24.4 Command blocks

Each executed command becomes a block containing:

- exact input text;
- resolved working directory;
- start/end time and duration;
- exit status and signal;
- output byte range or event refs;
- detected diagnostics and links;
- generated files/artifacts if observable;
- environment profile, not secret values;
- tags, notes, and agent associations;
- rerun, copy, compact, explain, and share actions.

Interactive full-screen TUIs remain normal pane intervals rather than pretending every internal action is a shell command.

### 24.5 Importing existing zsh configuration

The migration assistant should discover but not automatically execute or rewrite:

- `.zshrc`, `.zprofile`, and related files;
- aliases and functions;
- completion directories;
- prompt frameworks;
- history settings;
- keybindings;
- environment and version managers;
- common plugins.

It produces a compatibility report:

- keep in zsh unchanged;
- replace with a OpenTerm native feature;
- bridge through an adapter;
- conflicting or unsafe behavior;
- startup performance cost;
- secret-handling concern.

---

## 25. User experience roadmap

### 25.1 Terminal layout

The default workspace should include:

- central terminal pane;
- optional collapsible agent rail;
- command/result timeline;
- workspace and session selector;
- compact status line showing cwd, Git state, active environment, agent state, and resource budgets;
- command palette;
- notification center for completed jobs and approvals.

Avoid a permanently crowded “AI cockpit.” Every panel must be dismissible, keyboard-accessible, and useful at narrow widths.

### 25.2 Tabs, panes, and workspaces

Hierarchy:

```text
workspace
└── session
    ├── window/tab
    │   ├── terminal pane
    │   ├── agent pane
    │   └── document/result pane
    └── background jobs
```

- A workspace binds repository roots, instructions, indexes, policies, and memory.
- A session binds running processes and event history.
- A window/tab is a client presentation concept synchronized through the daemon.
- A pane renders a terminal, agent, document, diff, log, or plugin view.

### 25.3 Command palette

The palette should search:

- built-in actions;
- plugin actions;
- workflows;
- commands and flags;
- open sessions and workspaces;
- files, symbols, docs, results, and history;
- settings and keybindings.

Every entry states its effect class and shortcut. Mutating agent actions cannot disguise themselves as navigation actions.

### 25.4 Agent interaction modes

- **Ask**: read-only answers grounded in current session/workspace.
- **Draft**: proposes commands, patches, workflows, or docs without applying.
- **Act**: executes within granted capabilities and approval policy.
- **Watch**: monitors a job or event condition and reports deltas.
- **Teach**: explains terminal commands and builds reusable workflows.

The active mode must be visually obvious. Capability grants should be narrower than conversational modes.

### 25.5 Inline AI actions

On selected terminal text or a command block:

- explain;
- summarize;
- extract errors;
- create a reusable command;
- find the source location;
- draft a fix;
- compare with previous run;
- add to documentation;
- create a context pack;
- redact and share.

### 25.6 Accessibility

- complete keyboard navigation;
- screen-reader representation of command blocks;
- configurable motion and reduced animation;
- contrast validation for themes;
- IME and composed-text correctness;
- scalable fonts and UI chrome;
- semantic announcements for command completion and approval requests;
- non-color indicators for status and risk.

---

## 26. Plugin platform roadmap

### 26.1 Plugin types

OpenTerm should support narrowly defined extension types:

- **tool plugin**: adds typed commands;
- **agent plugin**: adds a specialized agent and prompt/tool policy;
- **adapter plugin**: normalizes an external CLI, service, or language ecosystem;
- **parser plugin**: recognizes diagnostics, links, artifacts, or command events;
- **completion plugin**: contributes deterministic completion sources;
- **history plugin**: ranks or annotates history without reading forbidden fields;
- **workflow plugin**: packages low-code automations;
- **view plugin**: renders a typed result or custom pane;
- **theme plugin**: provides colors, typography metadata, icons, and terminal palette;
- **keymap plugin**: provides named bindings with conflict metadata;
- **policy plugin**: adds organization rules;
- **connector plugin**: integrates an external application or data source.

### 26.2 Plugin manifest

```yaml
apiVersion: openterm.dev/plugin/v1
kind: Plugin
metadata:
  id: dev.openterm.docs-agent
  name: Documentation Agent
  version: 0.1.0
runtime:
  type: wasm
  entry: plugin.wasm
capabilities:
  filesystem:
    read: [workspace]
    write: ["docs/**"]
  history:
    read: metadata
  network: none
contributes:
  agents: [docs]
  commands: [docs.scan, docs.plan, docs.build, docs.verify]
  views: [docs.coverage, docs.graph]
  workflows: [document-workspace]
```

### 26.3 Runtime isolation

Preferred order:

1. data-only extensions for themes, keymaps, and simple completions;
2. WASM sandbox for deterministic tools and parsers;
3. isolated subprocess with a capability broker for SDK-heavy plugins;
4. in-process native plugins only for trusted first-party performance-critical components.

Plugins should access files, network, secrets, terminal input, and process execution through brokered APIs. A manifest declaration is not itself permission; the user or organization policy grants the capability.

### 26.4 Plugin lifecycle

- discover;
- inspect permissions and provenance;
- install with a lockfile;
- activate per user or workspace;
- migrate plugin state;
- update with changelog and permission diff;
- quarantine after crash loops;
- disable without preventing terminal startup;
- uninstall and optionally purge plugin data.

### 26.5 Marketplace

Marketplace metadata should include:

- publisher identity;
- source and build provenance;
- requested capabilities;
- supported platforms and protocol versions;
- release history;
- security review status;
- telemetry behavior;
- data storage/retention;
- reproducible-build status;
- user-visible permission changes between versions.

Do not make popularity a proxy for trust.

### 26.6 Plugin SDK milestones

- v0: out-of-process JSON-RPC tools;
- v1: stable command/result schemas and capability broker;
- v2: custom views and completion sources;
- v3: WASM runtime and signed packages;
- v4: remote-capable plugins and organization distribution;
- v5: compatibility guarantees and automated conformance certification.

---

## 27. Themes, prompts, keybindings, and personalization

### 27.1 Theme system

A theme can define:

- ANSI 16/256/true-color defaults;
- application background/foreground;
- cursor, selection, links, search matches;
- command block success/failure/running states;
- agent/tool/approval colors;
- diff and diagnostic colors;
- syntax token colors;
- light/dark variants;
- contrast metadata;
- optional icon pack;
- terminal image and transparency policy.

Themes must not execute code. Include a live preview and automated contrast checks.

### 27.2 Prompt system

Do not replace existing shell prompts by force. Offer:

- passthrough mode for Starship, Powerlevel10k, and custom prompts;
- a OpenTerm native status bar outside the PTY;
- an optional minimal shell prompt that avoids duplicating status-bar data;
- structured prompt segments supplied by plugins;
- latency budgets so slow segments degrade gracefully;
- sensitive-segment rules to prevent accidental capture/sharing.

### 27.3 Keybinding system

- named actions, not injected key sequences where possible;
- multiple profiles: familiar terminal, tmux-like, VS Code-like, Emacs-like, vi-like;
- chord support;
- context predicates by pane type and mode;
- conflict detection and visualization;
- import assistant for common terminal/tmux bindings;
- per-workspace overrides only with explicit trust;
- exportable data format.

### 27.4 Dotfile compatibility

OpenTerm configuration should complement dotfiles:

- shell behavior remains in shell dotfiles;
- terminal appearance and app actions live in OpenTerm settings;
- shared declarative config may be stored in a dotfiles repository;
- machine-local secrets and paths remain outside sync;
- settings have schema validation and migrations;
- `config doctor` reports conflicts and startup regressions.

---

## 28. Smart typing, completion, and command composition

### 28.1 Completion layers

Completion should merge sources with transparent ranking:

1. shell-native completions;
2. command schemas and man/help output;
3. filesystem paths;
4. workspace symbols and tasks;
5. semantic history;
6. plugin sources;
7. optional AI suggestions.

Deterministic completions appear immediately. AI completions arrive asynchronously and are visually distinct.

### 28.2 Autotype rules

“Autotype” must not mean an agent secretly typing and pressing Enter. Separate:

- **ghost text**: suggestion only;
- **insert**: writes into the editable command buffer;
- **compose**: opens a structured command editor;
- **run**: explicit execution action;
- **agent execute**: audited tool execution with capability checks.

Never automatically submit destructive, privileged, remote, or secret-bearing commands from prediction alone.

### 28.3 Smart command editor

Features:

- shell-aware syntax highlighting;
- multiline editing;
- matching quote/substitution visualization;
- command and flag documentation;
- path and host previews;
- inferred effect badges;
- inline validation;
- environment-variable visibility without exposing secret values;
- pipeline visualization;
- convert command to a reusable low-code workflow;
- explain shell expansion before execution;
- dry-run adapter when supported.

### 28.4 Command risk analysis

Before execution, statically recognize:

- recursive deletion;
- filesystem root or broad glob targets;
- privilege elevation;
- force flags;
- redirects overwriting files;
- pipes from network to shell;
- production hosts/contexts;
- database mutations;
- secret expansion into command arguments;
- fork bombs and unbounded concurrency patterns.

Warnings must be precise enough to avoid habituation. The terminal is not responsible for preventing every valid dangerous command, but AI-triggered actions require stricter enforcement than human-typed commands.

### 28.5 Natural-language command drafting

Natural language can draft either:

- a conventional shell command with an expansion preview; or
- a typed OpenTerm command/workflow.

Prefer the typed form when the operation has important side effects, structured output, or repeat use. Always show target scope and expected effects before execution.

---

## 29. Smart history and terminal memory

### 29.1 History data model

Traditional shell history stores a command string and sometimes a timestamp. OpenTerm history should store:

- command text and normalized command identity;
- workspace, repository, branch, cwd, and host;
- start/end time, duration, exit status, and signal;
- shell and environment profile;
- output/result references and compact summary;
- related files, diagnostics, artifacts, and tests;
- whether human, workflow, plugin, or agent initiated it;
- approval and capability record;
- user tags, notes, pins, and redaction state;
- supersedes/retry relationship;
- retention and sensitivity class.

### 29.2 History features

- typo-tolerant and semantic search;
- filters by workspace, branch, host, status, time, tool, and initiator;
- successful-command preference with failure visibility;
- command sequence grouping into tasks;
- “what fixed this last time?” retrieval;
- compare output with a prior run;
- rerun with parameter placeholders;
- turn a sequence into a workflow;
- pin canonical project commands;
- detect obsolete commands after project changes;
- local-first sync with optional end-to-end encrypted sharing.

### 29.3 Ranking policy

Rank by:

- current workspace/host relevance;
- recency with decay;
- prior success;
- semantic match;
- user pinning;
- exact prefix and argument-position match;
- current file/task relationship;
- safety and environment compatibility.

Do not recommend a production command merely because it is frequent. Host and environment mismatch should sharply reduce ranking or block direct execution.

### 29.4 Privacy controls

- configurable commands excluded from history;
- secret-pattern and high-entropy detection before storage;
- private session mode;
- redact command arguments while keeping command identity;
- per-workspace retention;
- inspect and delete exact records;
- export in a documented format;
- no cloud synchronization by default;
- model context access separate from local search access.

### 29.5 Workspace memory

Memory is distinct from history. History records what happened; memory stores reviewed facts such as:

- canonical build/test commands;
- architectural decisions;
- project terminology;
- known flaky tests;
- service ownership;
- local environment setup;
- task-specific conclusions.

Every memory item needs evidence, creator, scope, creation date, optional expiration, and invalidation conditions.

---

## 30. Documentation Agent — first flagship plugin

The documentation agent should be the first complete vertical slice because it proves the platform's most valuable primitives while remaining primarily read-only.

### 30.1 Mission

The agent creates and maintains a trustworthy map of a workspace:

- what the project is;
- how to install, run, test, build, and release it;
- how components relate;
- where important APIs and configuration live;
- which documents are missing, stale, contradictory, or unsupported;
- which claims are grounded in code/config/history evidence.

### 30.2 Commands

- `docs.scan`: inventory existing docs, READMEs, comments, manifests, and generated references.
- `docs.map`: build a navigable documentation graph linked to source symbols and configs.
- `docs.coverage`: score important topics and components for documentation coverage.
- `docs.stale`: detect claims invalidated by source/config changes.
- `docs.question`: answer with exact workspace citations.
- `docs.plan`: propose document additions/updates and affected files.
- `docs.draft`: generate a hash-bound draft without writing.
- `docs.apply`: apply an approved documentation patch.
- `docs.verify`: validate commands, links, code references, and source citations.
- `docs.watch`: monitor changes and emit documentation deltas.
- `docs.export`: generate a portable static knowledge bundle.

### 30.3 Ingestion pipeline

1. discover applicable workspace instructions;
2. classify files and exclude secrets, vendor data, binaries, and generated noise;
3. parse manifests, configuration, schemas, and task definitions;
4. build file/symbol/dependency indexes;
5. extract existing documentation structure and claims;
6. associate claims with source evidence and hashes;
7. calculate coverage and freshness;
8. store compact summaries plus exact evidence refs;
9. present unanswered questions and proposed work.

### 30.4 Document graph

Node types:

- document;
- section;
- claim;
- symbol;
- file;
- package/service;
- command/task;
- configuration key;
- API endpoint/schema;
- decision;
- owner;
- external reference.

Important edges:

- `documents`;
- `implemented-by`;
- `configured-by`;
- `depends-on`;
- `contradicts`;
- `supersedes`;
- `verified-by`;
- `changed-since`;
- `owned-by`.

### 30.5 Evidence contract

Every generated factual claim should have one or more of:

- exact file span at a content hash;
- parsed manifest/config record;
- command result with exit status;
- test or build evidence;
- user-approved memory item;
- external URL with retrieval metadata.

Generated prose may synthesize evidence, but the UI must expose citations. Unsupported statements should be labeled as inference or proposal.

### 30.6 Initial generated artifacts

- `README` improvement draft;
- `docs/architecture.md`;
- `docs/development.md`;
- command/task reference;
- environment and configuration reference;
- package/service map;
- testing guide;
- troubleshooting guide derived from verified failures;
- glossary;
- documentation coverage report.

The agent must not generate all of these blindly. It should choose artifacts appropriate to the detected project.

### 30.7 Verification

- execute safe documented commands in an isolated workspace or dry-run mode;
- validate internal links and anchors;
- verify referenced paths and symbols;
- compare environment/config keys with actual schemas and code reads;
- mark examples that cannot be tested;
- detect copied secrets and machine-specific paths;
- report claims whose evidence changed after drafting;
- re-read rendered output for structural quality.

### 30.8 UI

The docs agent gets:

- documentation tree;
- coverage heatmap;
- stale-claim queue;
- source/document split view;
- evidence inspector;
- draft diff;
- verification status;
- “ask this workspace” box with cited answers;
- watch mode showing docs impact of current Git changes.

### 30.9 First release boundaries

Version 0.1 should:

- support Markdown and repository-local docs;
- work offline with a configured local or remote model;
- index TypeScript/JavaScript and Python projects first;
- cite local files and exact spans;
- draft but require approval before writing;
- verify links, paths, package tasks, and selected safe commands;
- avoid automatically publishing docs externally.

Later plugins can export to Notion, Google Docs, wikis, static sites, or API portals.

---

## 31. Agent platform

### 31.1 Agent definition

An agent package declares:

- purpose and user-visible boundaries;
- model requirements and fallback behavior;
- allowed tools and default capabilities;
- context-building policy;
- memory access;
- approval points;
- maximum steps, duration, cost, and returned tokens;
- output schema;
- evaluation suite;
- failure and recovery policy.

### 31.2 Built-in agents after documentation

1. **Explorer** — maps unfamiliar workspaces and answers cited questions.
2. **Test Doctor** — diagnoses failures and creates reproduction packs.
3. **Code Editor** — drafts scoped changes and verification plans.
4. **Git Steward** — organizes diffs, commits, branches, and review packets.
5. **Dependency Curator** — explains, updates, audits, and measures dependencies.
6. **Log Analyst** — watches and correlates local/remote logs using deltas.
7. **Environment Doctor** — diagnoses toolchains, PATH, versions, and configuration.
8. **Release Agent** — builds release evidence and drafts release operations.
9. **Remote Operator** — performs narrowly authorized host operations.

Each should be a plugin over shared primitives, not a new isolated framework.

### 31.3 Agent concurrency

- one coordinator may delegate bounded subtasks;
- each task has its own context budget and capability subset;
- agents share immutable result refs rather than copying outputs;
- writes use a single serialized plan/apply path per workspace;
- users can inspect, pause, cancel, or redirect each task;
- background agents send delta updates, not repeated full reports;
- concurrency limits apply per workspace, host, and external service.

### 31.4 Context manager

The context manager selects:

- current prompt/task;
- applicable workspace rules;
- terminal command blocks;
- exact file spans;
- diagnostics and diffs;
- prior result refs;
- reviewed memory;
- plugin-specific instructions.

It deduplicates, ranks, budgets, and records why every item was included. Full terminal scrollback should never be an automatic context source.

---

## 32. Revised master delivery roadmap

The earlier command-runtime phases remain valid, but this sequence supersedes their ordering for the complete terminal product.

### Milestone A — Product specification and protocol (weeks 1–4)

Build:

- product requirements and threat model;
- transcript benchmark corpus;
- result, effect, reference, session, and event schemas;
- daemon/client protocol prototype;
- technology spikes for tmux control mode, web terminal, and embedded VT;
- shell-hook prototypes for zsh and bash;
- plugin manifest and capability model;
- documentation-agent evidence contract;
- clickable UX prototype.

Exit gate:

- one shell command can be observed end to end as PTY bytes plus a structured command block;
- a client disconnect/reconnect does not lose the session;
- technology scorecard selects the v0 UI/VT path;
- architecture threat review has no unresolved critical design issue.

### Milestone B — Headless semantic core (weeks 5–10)

Build:

- daemon lifecycle and local IPC;
- session/event/result stores;
- command registry;
- read-only P0 tool suite;
- result compaction and expansion;
- workspace discovery and instruction scoping;
- plugin host v0 using isolated subprocesses;
- RTK CLI compatibility adapter, version detection, and double-wrap prevention;
- RTK provenance and gain metrics in the common result model;
- local metrics and benchmark runner;
- CLI client.

Exit gate:

- core works without a GUI;
- raw output survives while compact results reduce median returned tokens by at least 60%;
- plugins cannot read outside granted workspace scope;
- event replay reconstructs command history after restart.

### Milestone C — Documentation Agent alpha (weeks 11–16)

Build:

- Markdown/config/manifest ingestion;
- TypeScript and Python symbol indexing;
- documentation graph;
- `docs.scan`, `docs.map`, `docs.question`, `docs.plan`, `docs.draft`, `docs.verify`;
- exact local citations;
- coverage and stale-claim reports;
- approval-gated Markdown writes;
- evaluation corpus across 10–20 repositories.

Exit gate:

- answers cite exact evidence;
- drafts survive source-hash validation;
- verified commands and links report exact evidence;
- generated documentation scores better than baseline on correctness and usefulness;
- no unapproved writes or external publication.

### Milestone D — Terminal application alpha (weeks 17–26)

Build:

- selected VT renderer/client;
- panes, tabs, sessions, detach/reattach;
- command blocks and scrollback search;
- agent rail and result/diff/document panes;
- settings, themes, fonts, keybindings;
- zsh and bash integration;
- safe mode and crash recovery;
- macOS packages and Linux developer build.

Exit gate:

- daily shell workflows work without noticeable incompatibility in test matrix;
- UI restart does not kill sessions;
- terminal remains functional when all AI services are unavailable;
- input latency and memory remain within defined targets;
- accessibility audit covers core navigation.

### Milestone E — Smart shell beta (weeks 27–34)

Build:

- structured completion broker;
- ghost-text suggestions and command editor;
- smart history and ranking;
- risk analysis;
- command-to-workflow conversion;
- prompt/status integration;
- zsh configuration migration report;
- private sessions and history privacy controls.

Exit gate:

- deterministic completion latency meets target;
- AI suggestions never auto-submit;
- history filters prevent cross-host/environment mistakes in tests;
- users can inspect/delete/export all retained history;
- common zsh setups have documented compatibility results.

### Milestone F — Safe actions and workflows beta (weeks 35–42)

Build:

- file patching and hash-bound plans;
- approvals and effect UI;
- low-code runner;
- test/build/lint/typecheck adapters;
- managed background jobs;
- context packs;
- docs watch mode;
- Explorer and Test Doctor agents.

Exit gate:

- a small code change can be planned, approved, applied, tested, and undone where supported;
- stale plans fail safely;
- workflow effects are statically inspectable;
- agent actions are attributable and interruptible.

### Milestone G — Plugin SDK and marketplace preview (weeks 43–50)

Build:

- stable v1 plugin protocol;
- WASM sandbox evaluation/implementation;
- custom views, completions, themes, agents, and adapters;
- package signing and lockfiles;
- permission-diff updates;
- plugin developer CLI, templates, tests, and docs;
- curated first-party plugin collection.
- stable RTK sidecar protocol or embedded-engine integration, subject to upstream collaboration.

Exit gate:

- external developers can build a tool, completion, theme, and agent plugin;
- plugin crashes do not crash terminal sessions;
- capability violations are denied and audited;
- compatibility suite runs in CI.

### Milestone H — Remote and team features (weeks 51–62)

Build:

- SSH-backed sessions and remote helper;
- remote file/result refs;
- end-to-end encrypted optional sync;
- shareable redacted session/result bundles;
- database read adapters;
- container integration;
- organization policies and managed plugins.

Exit gate:

- host identity is explicit everywhere;
- remote fallback works without helper installation;
- shared bundles exclude secrets under adversarial tests;
- team policy cannot be bypassed by plugins or agent workflows.

### Milestone I — 1.0 hardening (weeks 63–72)

Build:

- performance and storage tuning;
- protocol compatibility policy;
- migrations and recovery tools;
- fuzzing and security review;
- installer/update/rollback system;
- crash reporting with privacy controls;
- complete user and plugin documentation;
- public benchmark report.

1.0 criteria:

- stable daily terminal use on supported platforms;
- session recovery and data migration tested;
- no critical security findings;
- ≥70% median token reduction on supported agent workflows without task-success regression;
- documentation agent and at least two other agents production-ready;
- plugin API stability commitment published.

---

## 33. Workstreams and suggested team

### 33.1 Parallel workstreams

- **Terminal core**: PTY, VT, input, rendering, panes, sessions.
- **Semantic runtime**: commands, results, workflows, context, stores.
- **Agent intelligence**: documentation agent, evaluations, context policies.
- **Plugin platform**: manifests, sandbox, SDK, marketplace.
- **Desktop experience**: command blocks, panels, themes, settings, accessibility.
- **Security/reliability**: policy, secrets, fuzzing, update chain, recovery.
- **Developer ecosystem**: adapters, docs, templates, sample plugins.

### 33.2 Lean founding team

A serious alpha can begin with 3–5 people:

- terminal/runtime engineer;
- desktop/product engineer;
- agent/retrieval engineer;
- optional security/platform engineer;
- product/design contributor with developer-tool experience.

A polished cross-platform 1.0 and marketplace likely needs 6–10 contributors or a longer schedule. A solo developer should narrow v0 to macOS/Linux, zsh, a web or tmux prototype, and the documentation agent.

### 33.3 Solo-build sequence

If building alone:

1. CLI semantic runtime;
2. zsh hook and command blocks written to SQLite;
3. docs agent in read-only mode;
4. simple desktop client attached to the daemon;
5. result/diff/docs views;
6. tmux compatibility sessions;
7. native embedded terminal spike;
8. plugin protocol only after two internal plugins share real needs.

This sequence produces useful software before attempting a full terminal replacement.

---

## 34. Repository structure for the complete product

```text
OpenTerm/
├── apps/
│   ├── desktop/              # native/web desktop client
│   ├── tui/                  # optional terminal-only client
│   └── cli/                  # headless client and automation
├── crates/ or packages/
│   ├── protocol/             # versioned messages and schemas
│   ├── daemon/               # sessions, panes, lifecycle, IPC
│   ├── pty/                  # cross-platform PTY abstraction
│   ├── terminal/             # VT integration and checkpoints
│   ├── runtime/              # tools, effects, budgets, workflows
│   ├── result-store/
│   ├── session-store/
│   ├── history/
│   ├── context/
│   ├── policy/
│   ├── plugin-host/
│   └── shell-integration/
├── plugins/
│   ├── docs-agent/
│   ├── explorer-agent/
│   ├── test-doctor/
│   ├── git/
│   ├── languages/
│   ├── themes/
│   └── completions/
├── sdk/
│   ├── typescript/
│   ├── rust/
│   └── plugin-template/
├── integrations/
│   ├── zsh/
│   ├── bash/
│   ├── fish/
│   ├── tmux/
│   ├── rtk/                  # native compression engine adapter/protocol
│   └── ssh-helper/
├── schemas/
├── workflows/
├── benchmarks/
├── fixtures/
├── design/
│   ├── mascot/               # canonical artwork and animation timing
│   ├── tokens/               # colors, spacing, type, motion
│   └── prototypes/           # interaction and layout prototypes
├── docs/
│   ├── architecture/
│   ├── protocol/
│   ├── plugin-sdk/
│   ├── security/
│   └── product/
└── roadmap.md
```

Do not create this entire tree on day one. Add a package only when ownership or build boundaries are real.

---

## 35. Immediate next 50 issues for the complete terminal

1. Write an architecture decision record for hybrid terminal architecture.
2. Define supported platforms and shells for alpha.
3. Define measurable terminal compatibility scenarios.
4. Specify session/window/pane/workspace identities.
5. Specify ordered event envelopes and replay semantics.
6. Specify command-block lifecycle.
7. Specify daemon/client authentication and IPC framing.
8. Prototype PTY creation, resize, signals, and teardown.
9. Prototype zsh preexec/precmd integration.
10. Prototype bash integration.
11. Record command events and PTY byte offsets in SQLite.
12. Reattach a CLI client to a running daemon session.
13. Prototype tmux control-mode adapter.
14. Prototype embedded VT candidate A.
15. Prototype web terminal candidate B.
16. Run the technology selection scorecard.
17. Define terminal checkpoint and scrollback storage.
18. Fuzz escape-sequence ingestion and clipboard controls.
19. Define plugin manifest v0.
20. Build subprocess plugin host with capability broker.
21. Define model-provider-neutral inference interface.
22. Define context pack manifest.
23. Implement workspace instruction discovery.
24. Implement file classification and exclusions.
25. Implement result/reference store.
26. Implement read-only P0 filesystem commands.
27. Implement ripgrep and Git adapters.
28. Implement deterministic compaction.
29. Design documentation claim/evidence schemas.
30. Build Markdown document parser.
31. Build manifest/config ingestion.
32. Build TypeScript symbol index.
33. Build Python symbol index.
34. Implement `docs.scan`.
35. Implement `docs.map`.
36. Implement cited `docs.question`.
37. Implement `docs.coverage` and `docs.stale`.
38. Implement hash-bound `docs.draft`.
39. Implement link/path/command verification.
40. Create docs-agent evaluation repositories and rubrics.
41. Design desktop command-block UI.
42. Design agent/result/diff/document pane contracts.
43. Define theme tokens and contrast tests.
44. Define named keybinding actions and conflict resolution.
45. Build smart-history schema and privacy controls.
46. Design completion broker and latency budgets.
47. Implement safe/private startup modes.
48. Define crash recovery and data migration tests.
49. Publish local performance/token benchmark harness.
50. Produce an alpha demo: persistent terminal + cited docs agent + compact results.

---

## 36. Technology references and prototypes to study

These are references, not mandatory dependencies:

- [tmux](https://github.com/tmux/tmux) for server-based terminal multiplexing, detach/reattach, and control-mode integration.
- [WezTerm multiplexing](https://wezterm.org/multiplexing.html) for the daemon/domain/client model.
- [Ghostty and libghostty](https://github.com/ghostty-org/ghostty) for a modern embeddable terminal engine and VT behavior.
- [Ghostty terminal API documentation](https://ghostty.org/docs) for terminal concepts and supported control sequences.
- [zsh completion system](https://zsh.sourceforge.io/Doc/Release/Completion-System.html) for contextual completion and styles.
- [RTK](https://github.com/rtk-ai/rtk) and its [architecture documentation](https://github.com/rtk-ai/rtk/blob/develop/docs/contributing/ARCHITECTURE.md) for command-specific filtering, agent hooks, token tracking, and fail-safe CLI proxy behavior.

Also evaluate, through official projects and prototypes, mature PTY libraries, tree-sitter, SQLite full-text search, WASM runtimes, shell-integration protocols, and platform accessibility APIs. Record each dependency decision in an architecture decision record with license, maintenance, security, and replacement costs.

---

## 37. Terminal architecture recommendation

Build the complete AI-oriented terminal—but build it as a layered platform, not a heroic rewrite of terminal history.

The winning architecture is:

```text
mature VT engine + OS PTYs + existing user shell
                         │
              OpenTerm session daemon
                         │
       semantic runtime + result/reference store
                         │
       agent platform + capability-safe plugins
                         │
         native terminal and AI workspace UI
```

Use tmux immediately as a prototype, remote compatibility layer, and migration bridge. Do not let tmux's byte-oriented model define the final product. Own the semantic session protocol from the beginning, because command blocks, smart history, evidence, token budgets, and plugin permissions all depend on it.

Start with the documentation agent. It will force the project to solve workspace indexing, citations, context control, history, result views, plugin isolation, and approval-gated writes in one coherent vertical slice. Once that works inside persistent terminal sessions, the project has a genuine foundation rather than a themed terminal with a chat box attached.

---

## 38. Native RTK integration

### 38.1 Decision

[RTK — Rust Token Killer](https://github.com/rtk-ai/rtk) becomes a first-party OpenTerm subsystem, not an optional afterthought and not merely a shell alias.

RTK already provides the strongest initial implementation of the command-output compression layer described throughout this roadmap:

- a single Rust binary;
- command-specific routing and filtering;
- smart filtering, grouping, truncation, and deduplication;
- support for files, Git, GitHub CLI, test runners, linters, package managers, containers, cloud tools, and infrastructure commands;
- fail-safe fallback to original output;
- preservation of process exit codes;
- verbosity levels up to raw output;
- token-savings tracking in SQLite;
- failure-output tee storage;
- agent-specific command-rewrite hooks;
- Apache-2.0 licensing.

OpenTerm should reuse those mature filters and contribute improvements upstream. It should not independently recreate RTK's command modules under different names.

### 38.2 Product boundary

```text
┌──────────────────────────────── OpenTerm ────────────────────────────────┐
│ terminal UI │ session daemon │ agents │ plugins │ policy │ result store │
│                                                                         │
│  command event                                                         │
│       │                                                                 │
│       ▼                                                                 │
│  execution router ───── native structured tool? ── yes ──▶ OpenTerm    │
│       │ no                                                              │
│       ▼                                                                 │
│  RTK compression engine ─▶ compact result + metrics + raw artifact ref  │
│       │                                                                 │
│       ▼                                                                 │
│  actual git/test/build/cloud/system command                             │
└─────────────────────────────────────────────────────────────────────────┘
```

RTK owns:

- recognition of supported conventional commands;
- command-specific flag handling;
- choosing structured output modes from upstream tools;
- parsing tool output;
- noise removal and failure-focused compaction;
- initial savings estimates;
- compatibility with standalone AI coding tools.

OpenTerm owns:

- PTYs and persistent session lifecycle;
- human-versus-agent execution identity;
- command blocks and UI presentation;
- typed result envelopes and stable refs;
- lossless raw-output storage and expansion;
- token budgets tied to the selected model;
- capabilities, approvals, secrets, and remote policy;
- plugins and workflows;
- workspace indexes and semantic code context;
- cross-command history, artifacts, and provenance;
- context-pack selection;
- agent orchestration.

### 38.3 Integration modes

Support four explicit modes.

#### Mode 1 — Embedded engine

Preferred long-term mode. RTK filtering modules are exposed through a Rust library API linked into the OpenTerm daemon.

```rust
let request = RtkRequest {
    argv,
    cwd,
    environment_profile,
    verbosity,
    output_budget,
};

let result = rtk_engine.execute(request, sinks).await?;
```

The conceptual result contains:

- original command identity;
- exact executed argv;
- exit status and signal;
- compact text;
- optional structured records;
- raw-output stream or artifact handle;
- filter strategy and module version;
- input/output byte and token estimates;
- warnings and fallback status;
- duration and child-process metadata.

RTK is currently organized as a CLI binary, so this mode likely requires an upstream refactor that separates parsing, execution, filtering, formatting, and tracking into reusable crates. OpenTerm should propose and contribute that refactor rather than maintaining a permanent private fork.

#### Mode 2 — Sidecar protocol

Recommended first production integration. OpenTerm launches a pinned RTK binary with a machine-readable protocol.

Required upstream interface:

```sh
rtk protocol execute \
  --request /path/to/request.json \
  --events-fd 3 \
  --raw-fd 4
```

Or an equivalent framed stdin/stdout protocol. It must not rely on parsing colored human output.

Advantages:

- process isolation;
- independent RTK upgrades;
- simpler initial licensing and build boundary;
- crash containment;
- usable before a stable Rust library API exists.

#### Mode 3 — CLI compatibility adapter

Immediate MVP path using the existing `rtk` executable:

- discover RTK and its version;
- invoke supported `rtk ...` commands;
- preserve exit status;
- disable color when machine parsing;
- collect RTK's compact stdout;
- ingest RTK gain/history data where safely accessible;
- enable tee storage as a transitional raw-output path;
- clearly mark results whose original output is not losslessly available.

This mode is useful but cannot satisfy OpenTerm's complete lossless-result contract for every successful command. It is a bridge, not the finished native integration.

#### Mode 4 — Standalone hook bridge

Keep RTK's existing hooks for external agents and terminals:

- Codex instruction integration;
- Claude Code and Copilot hooks;
- Cursor, Gemini CLI, OpenCode, and other supported agents;
- ordinary zsh/bash sessions outside OpenTerm.

Inside the OpenTerm terminal, the session daemon already observes command boundaries and should call RTK directly. Installing a second rewrite hook inside OpenTerm could cause double wrapping and must be detected and disabled.

### 38.4 Why RTK should be upstream-first

RTK is active, broad, and useful independently of OpenTerm. A private vendor fork would quickly create:

- duplicated filter maintenance;
- delayed support for new command versions;
- security divergence in command execution code;
- confusing user reports across `rtk` and OpenTerm;
- license and attribution bookkeeping;
- incompatible metrics/history formats.

Preferred contribution sequence:

1. document the OpenTerm integration requirements;
2. open upstream design discussions;
3. add stable JSON/event output without changing normal CLI behavior;
4. add raw-output sink callbacks or file descriptors;
5. separate reusable filter modules from CLI presentation;
6. expose command detection/rewrite as a library or protocol call;
7. add conformance fixtures for every supported command;
8. keep a narrow adapter for older RTK versions;
9. pin and verify releases in OpenTerm builds;
10. fork only if upstream integration becomes technically impossible, and document the ongoing cost.

### 38.5 Proposed RTK crate split

This is a proposed collaboration target, not a description of the current repository:

```text
rtk/
├── rtk-cli/             # clap, human formatting, init, gain commands
├── rtk-core/            # requests, results, errors, strategy metadata
├── rtk-exec/            # safe argv execution, streaming, exit handling
├── rtk-filters/         # command-specific parsers and compactors
├── rtk-detect/          # command recognition and rewrite plans
├── rtk-protocol/        # serde schemas and protocol server/client
├── rtk-tracking/        # optional analytics backend
└── rtk-fixtures/        # representative raw/compact golden outputs
```

OpenTerm should depend on the smallest appropriate public interfaces. It should not reach into RTK private modules or database files when a supported API can be added.

### 38.6 OpenTerm result mapping

Map RTK results into the common envelope:

```json
{
  "schema": "openterm.result/v1",
  "command": "exec.run",
  "status": "failed",
  "summary": "2 of 84 tests failed",
  "data": {
    "filter_engine": "rtk",
    "filter_module": "cargo-test",
    "failures": [
      {"name": "auth::expired_token", "span_ref": "span_01..."}
    ]
  },
  "refs": [
    {"id": "res_01...", "kind": "compact-result"},
    {"id": "artifact_01...", "kind": "raw-output"}
  ],
  "omitted": {
    "bytes": 58120,
    "reason": "rtk_failure_focus"
  },
  "provenance": {
    "engine": "rtk",
    "engine_version": "pinned-build-version",
    "fallback": false
  },
  "metrics": {
    "raw_bytes": 61200,
    "returned_bytes": 3080,
    "estimated_tokens_saved": 14530
  }
}
```

OpenTerm should retain RTK's compact human text while also mapping available diagnostics, tests, files, and counts into typed fields. Do not manufacture structured precision when a filter only produced text.

### 38.7 Raw output and progressive expansion

This is the most important native extension.

RTK's current failure tee is valuable, but OpenTerm requires raw output for both successes and failures when retention policy allows. The integration needs:

- streaming raw stdout and stderr into OpenTerm's content-addressed store;
- separate stream identity and ordering metadata;
- no second execution to recover missing details;
- raw artifact hashes;
- retention and sensitivity classification;
- expansion by line range, diagnostic, test, hunk, or event group;
- deterministic relationship between compact claims and raw evidence;
- zero raw output inserted into model context unless requested.

For interactive commands and TUIs, retain PTY events and checkpoints rather than pretending the final screen is ordinary stdout.

### 38.8 Command routing

The router should classify a command before execution:

```text
1. Is it a OpenTerm native typed command?
   yes -> run OpenTerm command

2. Does an enabled plugin own this exact command/schema?
   yes -> run plugin under capabilities

3. Does RTK support the executable/subcommand safely?
   yes -> execute through RTK engine

4. Is generic RTK error/test/summary mode appropriate?
   yes -> offer or apply declared generic filter policy

5. Otherwise -> transparent PTY execution with OpenTerm generic compaction
```

Routing must respect:

- shell operators and pipelines;
- aliases/functions that change command identity;
- interactive/TTY requirements;
- commands reading from stdin;
- commands that mutate state;
- unsupported or ambiguous flags;
- user/plugin exclusions;
- debugging and raw modes;
- remote execution boundaries.

Never rewrite by string concatenation. Preserve a parsed argv or an exact shell command with explicit shell semantics.

### 38.9 Double-filter prevention

Possible double-filter paths include:

- OpenTerm daemon routes through RTK while an RTK shell/agent hook rewrites again;
- an agent explicitly runs `rtk git status` inside a session configured for automatic RTK routing;
- a remote OpenTerm helper and local client both apply filters;
- generic OpenTerm compaction runs after an RTK command-specific filter.

Prevent them through:

- `OPENTERM_RTK_ACTIVE=1` execution marker;
- RTK provenance in command events;
- detection of an existing `rtk` argv prefix;
- one declared compression owner per command;
- generic compaction limited to envelope/presentation cleanup after RTK;
- visible routing diagnostics in developer mode.

### 38.10 Configuration

```toml
[compression]
engine = "rtk"
fallback = "openterm-generic"
raw_retention = "failures-and-agent-runs"
default_profile = "balanced"

[compression.rtk]
mode = "sidecar"              # embedded | sidecar | cli | disabled
version = "pinned"
verify_checksum = true
inherit_user_config = true
import_gain_history = false
exclude = ["playwright --ui", "ssh"]

[compression.profiles.balanced]
token_budget = 1800
preserve_warnings = true
preserve_source_spans = true

[compression.profiles.ultra]
token_budget = 600
allow_stats_only = true
```

Workspace configuration may reduce compression for debugging, but it cannot silently weaken organization retention, redaction, or security policy.

### 38.11 UI integration

Every RTK-backed command block can show:

- `RTK` engine badge;
- filter strategy/module;
- raw versus returned token estimate;
- percentage and cumulative tokens saved;
- compact/balanced/raw view selector;
- “expand omitted evidence” action;
- fallback or parser-warning indicator;
- link to raw artifact if retained;
- filter feedback action for false positives or missing evidence.

Add a Token Economy dashboard:

- savings by command, workspace, agent, and session;
- expansion rate after compaction;
- estimated cost and latency avoided;
- filters with the most fallbacks;
- commands missing RTK coverage;
- correctness incidents associated with over-compression;
- storage cost of retained raw output.

Do not celebrate token savings alone. Put task success, expansion rate, and omitted-context incidents beside the savings graph.

### 38.12 Smart history integration

RTK tracks command-level savings; OpenTerm tracks semantic session history. Avoid two competing sources of truth.

Plan:

- preserve RTK's standalone history when RTK runs outside OpenTerm;
- when embedded/sidecar mode supports it, disable duplicate RTK SQLite writes and emit tracking events to OpenTerm;
- import existing RTK gain history only with user consent;
- retain original command, RTK route, input/output estimates, filter module, and execution time;
- do not import raw sensitive command output from legacy tee files automatically;
- let `rtk gain` continue working in standalone mode;
- expose equivalent OpenTerm dashboards from shared protocol events, not direct database coupling.

### 38.13 Security requirements

RTK executes developer commands and intercepts agent commands, so it sits on a sensitive trust boundary.

- pin supported versions and verify release checksums/signatures when available;
- display exact version and provenance;
- preserve RTK's no-shell execution patterns where applicable;
- isolate sidecar process capabilities to the command's granted scope;
- do not let filtering hide approval-relevant targets or side effects;
- run risk analysis before compression/execution;
- retain exit codes and signals exactly;
- fail open to visible raw output for filter errors, but fail closed for capability/policy violations;
- redact secrets before analytics and model delivery, not only after filtering;
- fuzz parsers with hostile ANSI, Unicode, huge lines, and crafted tool output;
- treat hook/rewrite, execution, tracking, networking, and update code as high-risk review areas;
- make telemetry local/off by default under OpenTerm policy;
- preserve Apache-2.0 license notices and dependency attribution.

### 38.14 Versioning and compatibility

Define a compatibility matrix:

| OpenTerm integration | RTK requirement | Support level |
|---|---|---|
| CLI adapter | existing stable binary | compatibility |
| Sidecar v1 | protocol schema v1 | recommended |
| Embedded v1 | stable Rust crate API v1 | preferred |
| Hook bridge | supported RTK agent hook | external compatibility |

The protocol handshake should report:

- RTK version and build hash;
- supported protocol versions;
- command/filter registry;
- capabilities such as raw streaming, structured diagnostics, and tracking events;
- configuration schema version;
- platform limitations.

Unknown RTK versions fall back to conservative CLI or transparent execution rather than being silently treated as compatible.

### 38.15 Testing strategy

#### Golden compatibility

For every supported RTK command:

- fixture raw stdout/stderr;
- expected compact output;
- exit status;
- structured field mapping;
- raw artifact hash;
- omitted counts;
- verbose/raw behavior;
- unsupported-flag fallback.

#### Differential tests

Compare:

- direct command versus RTK standalone;
- RTK standalone versus OpenTerm sidecar;
- sidecar versus embedded engine;
- compact claim versus stored raw evidence;
- existing RTK gain metrics versus OpenTerm-imported metrics.

#### Safety tests

- shell metacharacters remain exact;
- filenames containing spaces/newlines/leading dashes;
- exit code and signal preservation;
- command-not-found behavior;
- interactive command bypass;
- double-filter detection;
- plugin cannot forge RTK provenance;
- redaction before history storage;
- corrupt or malicious sidecar messages;
- RTK crash and timeout recovery;
- old/new protocol negotiation.

### 38.16 Delivery plan

#### RTK Phase 0 — Compatibility adapter (1–2 weeks)

- detect installed RTK and version;
- add `compression.engine=rtk-cli`;
- map known command routes;
- preserve exit status;
- ingest compact output into command blocks;
- expose `rtk gain` from the Token Economy view;
- detect hooks and prevent double wrapping;
- document lossless-output limitations.

#### RTK Phase 1 — Sidecar protocol proposal (2–4 weeks)

- write protocol and raw-stream RFC;
- engage RTK upstream;
- implement experimental request/result/event schemas;
- add raw stdout/stderr sinks;
- add structured filter metadata;
- build protocol conformance fixtures;
- prototype OpenTerm daemon adapter.

#### RTK Phase 2 — First-class sidecar (3–5 weeks)

- ship pinned RTK sidecar option;
- version handshake and capability detection;
- full raw artifact retention;
- typed result mapping;
- OpenTerm-native tracking events;
- cancellation, timeout, and crash recovery;
- macOS/Linux packaging and update policy.

#### RTK Phase 3 — Library extraction (4–8 weeks, upstream-dependent)

- separate core/filter/detect/protocol APIs;
- embed engine behind a feature flag;
- eliminate process/serialization overhead where measurable;
- preserve sidecar as isolation and compatibility option;
- publish API stability and semver policy.

#### RTK Phase 4 — Ecosystem collaboration (ongoing)

- upstream new command filters from OpenTerm plugins;
- share anonymized opt-in coverage gaps, never command contents by default;
- maintain cross-project fixtures;
- coordinate security advisories;
- expose third-party filter SDK through RTK or OpenTerm without fragmenting formats.

### 38.17 RTK-specific backlog

1. Add RTK dependency/license architecture decision record.
2. Inventory RTK commands against OpenTerm command backlog.
3. Define compression-owner routing rules.
4. Implement RTK discovery and semver checks.
5. Implement explicit `rtk` prefix detection.
6. Implement RTK hook/double-wrap detection.
7. Add RTK execution provenance to result envelope.
8. Add compact/raw view state to command blocks.
9. Add lossless-availability indicator.
10. Add RTK gain dashboard adapter.
11. Define metrics import consent and deduplication.
12. Write sidecar protocol RFC.
13. Write raw-stream/result-ref RFC.
14. Write upstream crate-split proposal.
15. Build Git status/diff/log golden integration tests.
16. Build cargo/pytest/vitest golden integration tests.
17. Build lint/typecheck golden integration tests.
18. Build Docker/Kubernetes/log golden integration tests.
19. Test all exit-code and signal paths.
20. Fuzz RTK-to-OpenTerm result mapping.
21. Add model-aware tokenizer interface beyond four-characters-per-token estimates.
22. Add expansion analytics and over-compression incident tracking.
23. Add conservative fallback for unsupported flags.
24. Add remote RTK capability negotiation.
25. Document upstream contribution and release process.

### 38.18 Final integrated foundation

With RTK integrated, OpenTerm does not begin by rebuilding 100 command filters. It begins with a working compression engine and focuses its original engineering on the larger platform:

```text
libghostty/mature VT       tmux compatibility       zsh/bash/fish
          └──────────────────────┬──────────────────────┘
                                 ▼
                    OpenTerm session daemon
                                 ▼
                  RTK command compression engine
                                 ▼
          result refs + raw evidence + semantic history
                                 ▼
            agents + workflows + capability-safe plugins
```

That is a much stronger starting point. RTK becomes the native “token kernel” for conventional CLI output; OpenTerm becomes the complete terminal, session, context, safety, and agent platform around it.

---

## 39. OpenTerm identity and exceptional UI/UX

### 39.1 Product identity

The public product name is **OpenTerm**.

Naming hierarchy:

- **OpenTerm** — application and ecosystem;
- **OpenTerm Core** — session daemon and semantic runtime;
- **OpenTerm Compress** — RTK-derived token compression engine;
- **OpenTerm Flow** — low-code workflow system;
- **OpenTerm Docs** — documentation agent;
- **OpenTerm SDK** — plugin development kit;
- **OpenTerm Hub** — optional plugin/theme marketplace;
- **OpenTerm Remote** — SSH and remote-session layer.

CLI and protocol names:

- executable: `openterm`;
- short alias offered, not required: `oterm`;
- configuration: `~/.config/openterm/` or platform-native equivalent;
- project configuration: `.openterm/`;
- environment prefix: `OPENTERM_`;
- schemas: `openterm.*` and `openterm.dev/*`;
- local daemon socket: platform runtime directory plus `openterm.sock`;
- plugin IDs: reverse-domain names such as `dev.openterm.docs`.

Before public release, perform trademark, package registry, domain, executable-name, and app-store availability checks. Keep naming configurable in code until those checks are complete.

### 39.2 Experience thesis

OpenTerm should feel like the terminal finally became aware of the work happening inside it.

The UI must be:

- as immediate as a great native terminal;
- as navigable as a modern code editor;
- as inspectable as a debugger;
- as calm as a focused writing tool;
- as expressive as a personal zsh setup;
- safer and more legible when an agent acts;
- delightful without turning developer work into a toy.

The mascot supplies personality. The rest of the interface earns trust through restraint, speed, hierarchy, and evidence.

### 39.3 Non-negotiable quality bars

- Keystrokes render without perceptible delay.
- Terminal output never waits on an AI service.
- Pane resize and scroll remain smooth under heavy output.
- UI restart does not terminate sessions.
- Every action is keyboard-accessible.
- Mouse interaction enriches but never gates core features.
- The interface remains useful at 80×24.
- Core status never relies on color alone.
- Reduced-motion mode preserves meaning.
- Agent activity never steals keyboard focus.
- Destructive scope is visible before approval.
- Large output does not cause layout jumps.
- Empty, loading, offline, denied, failed, and partial states are deliberately designed.
- No essential control is hidden behind a mascot animation or novelty interaction.

### 39.4 Visual language

The visual system should combine terminal precision with warm, tactile details:

- dark and light foundations with excellent contrast;
- restrained neutral surfaces;
- one vivid OpenTerm accent, theme-overridable;
- monospace content paired with an optional UI font for chrome;
- thin separators and spacing before heavy boxes;
- command blocks as subtle temporal landmarks;
- status encoded with icon, label, and color;
- motion used for causality, not decoration;
- the mascot rendered crisply in terminal glyphs rather than blurred raster imitation.

Avoid:

- a generic purple-gradient AI dashboard;
- permanent glowing borders;
- glass effects that reduce readability;
- excessive cards inside cards;
- chat bubbles for every tool event;
- animated backgrounds behind terminal text;
- a sidebar that consumes a third of the screen by default;
- confetti for routine success;
- hiding exact commands behind friendly prose.

### 39.5 Design tokens

Token categories:

```text
color.background.canvas
color.background.surface
color.background.elevated
color.text.primary
color.text.secondary
color.text.muted
color.border.subtle
color.border.focus
color.accent.openTerm
color.status.success|warning|error|running|waiting
color.effect.read|write|network|remote|privileged
color.agent.thinking|acting|waiting|blocked
color.mascot.body|eyes|background

space.1 ... space.8
radius.small|medium|large
type.terminal|ui|code
motion.fast|normal|deliberate
easing.enter|exit|spring
opacity.disabled|muted|overlay
```

The terminal palette and application palette are related but separate. A user theme can change both, while effect/risk states must retain minimum perceptual separation.

### 39.6 Information architecture

Primary navigation:

- workspaces;
- sessions;
- command timeline;
- agents;
- jobs;
- docs/knowledge;
- history;
- plugins;
- settings.

The terminal remains the default center. Secondary destinations open as panes, overlays, or temporary inspectors rather than replacing the terminal with an administration dashboard.

### 39.7 The command block as the central object

A command block is more useful than raw scrollback but must remain visually light.

Collapsed block:

```text
● cargo test                           2 failed · 1.8s · RTK saved 91%
  auth::expired_token                 src/auth/tests.rs:84
  auth::missing_scope                 src/auth/tests.rs:121
```

Expanded block can reveal:

- exact shell input;
- output stream;
- compact/raw toggle;
- diagnostics;
- changed artifacts;
- environment and cwd;
- duration and resources;
- token savings;
- initiator and approvals;
- related agent reasoning summary;
- rerun, compare, copy, pin, document, and workflow actions.

Command chrome appears on hover/focus or when metadata is important. Normal terminal use should not resemble a stack of thick web cards.

### 39.8 Agent presence

Agent state should be ambient and precise:

- tiny state indicator in the status line;
- activity appears next to the command/result it affects;
- an expandable rail shows plan, current step, budgets, and tools;
- approval requests become focused overlays with exact effect scope;
- completed work collapses into a concise evidence-backed summary;
- background agents use nonintrusive notifications;
- multiple agents appear as a task tree, not overlapping chat streams.

The mascot may react to major agent states, but must not be the only state indicator.

Suggested mascot state mapping:

- idle: default pose with no continuous animation;
- agent started: brief look-around;
- task completed: one jump/wave sequence;
- waiting for approval: subtle look toward the approval control;
- error: still pose plus explicit error UI—no comedic animation;
- long-running job: optional low-frequency blink in GUI rendition;
- reduced motion: pose change only.

### 39.9 Interaction model

Every operation should have one obvious path and one expert shortcut.

- `Cmd/Ctrl+K`: universal action palette;
- `Cmd/Ctrl+P`: files, symbols, docs, sessions, and results;
- `Cmd/Ctrl+Shift+P`: workflows and agent actions;
- pane-local search remains familiar;
- command-block navigation has dedicated next/previous shortcuts;
- selection opens a minimal context bar;
- holding a modifier reveals effect and provenance details;
- Escape always retreats one interaction layer;
- keyboard focus is visible and never trapped.

Keybindings remain configurable through named actions, with discoverable conflict resolution.

### 39.10 Progressive disclosure

Use four layers:

1. **Glance** — state, summary, severity, and next action.
2. **Inspect** — structured diagnostics, groups, affected files, and evidence.
3. **Expand** — exact output sections, diffs, traces, and metadata.
4. **Forensic** — raw streams, event sequence, environment profile, parser version, hashes.

This same pattern should govern terminal results, agents, history, docs, plugins, and remote operations.

### 39.11 Onboarding

First run should take under two minutes:

1. OpenTerm welcomes the user with the mascot and a real terminal prompt.
2. Detect the current shell and offer transparent integration.
3. Import theme/keybinding preferences without modifying dotfiles yet.
4. Explain local-only defaults in one screen.
5. Offer a model provider or “terminal only for now.”
6. Demonstrate one compact command result.
7. Offer to run the documentation-agent scan read-only.

Never require an account to use local terminal features. Never dump a ten-step feature tour over the prompt.

### 39.12 Theme experience

- live preview using a real command block, diff, diagnostics, and mascot;
- instant switching without restart;
- light/dark/system modes;
- separate terminal and chrome font controls;
- import common terminal color schemes;
- import Ghostty-compatible themes where licensing and format permit;
- theme diagnostics for contrast, indistinguishable statuses, and unsupported colors;
- shareable declarative theme packages;
- per-workspace theme accent without arbitrary workspace code.

### 39.13 Motion system

Motion durations:

- 60–100 ms: hover/focus feedback;
- 120–180 ms: small reveal/collapse;
- 180–240 ms: pane/overlay transition;
- mascot animation: canonical 60 ms frame cadence;
- no layout animation during heavy terminal output.

Principles:

- animate opacity and transform where possible;
- preserve spatial continuity;
- never delay command input or approval;
- cancel obsolete transitions;
- freeze offscreen animation;
- respect OS and OpenTerm reduced-motion preferences;
- test at slow frame rates and under load.

### 39.14 Empty and failure states

Examples:

- no sessions: one-click new local session plus recent workspaces;
- no search results: show scope and filters, not a joke;
- agent offline: terminal remains primary and fully usable;
- model unavailable: preserve draft and offer retry/provider change;
- plugin crashed: isolate, explain capability scope, restart/disable;
- compression fallback: show raw output and parser warning;
- daemon recovering: show sessions being restored individually;
- remote disconnected: preserve local scrollback and offer explicit reconnect;
- index stale: label results and allow refresh.

### 39.15 Performance budgets

Set budgets before implementation:

| Interaction | Target |
|---|---:|
| local keystroke to visible glyph | p95 < 16 ms |
| command-block metadata update | p95 < 50 ms |
| deterministic completion first result | p95 < 50 ms |
| command palette open | p95 < 100 ms |
| workspace switch UI response | p95 < 100 ms |
| search first page on indexed history | p95 < 150 ms |
| client attach to warm daemon | target < 300 ms |
| cold window usable | target < 1 s on reference machine |

These are product targets, not promises before prototyping. Measure on reference hardware and publish regressions in CI.

### 39.16 UX validation

Test with:

- developers who live in tmux;
- developers with heavy zsh customization;
- users of GUI terminals and IDE-integrated terminals;
- screen-reader and keyboard-only users;
- operators managing remote systems;
- newcomers learning the shell;
- AI coding-tool power users;
- users working offline or with no model configured.

Research tasks:

- recover an old command and adapt it safely;
- diagnose a failing test;
- compare compact and raw output;
- approve a file change while rejecting a network action;
- detach and recover a session;
- install and inspect a plugin;
- generate and verify project documentation;
- understand what an agent is currently doing;
- turn a successful command sequence into a workflow.

Track completion, errors, time, confidence, and whether users can accurately explain what happened.

### 39.17 Design delivery artifacts

Before implementation of each surface, create:

- user journey;
- state model;
- keyboard map;
- low-fidelity flow;
- high-fidelity light/dark screens;
- narrow and wide layouts;
- reduced-motion variant;
- empty/loading/error/permission-denied states;
- interaction prototype;
- accessibility notes;
- analytics-free local usability test plan;
- implementation tokens and component contract.

OpenTerm should maintain a small “UX constitution” in the repository so plugins and first-party features feel coherent.

---

## 40. OpenTerm mascot specification

### 40.1 Canonical asset

The OpenTerm monster is the canonical mascot. Its exact terminal art and animation timing are preserved in:

```text
design/mascot/
├── README.md
├── types.ts
├── art.ts
├── animation.ts
└── preview.txt
```

The files were derived directly from the user's local OpenCode project at `/Users/remipelloux/Desktop/projet/OpenOS/OpenCode/components/LogoV2/`. Preserve the provenance file.

### 40.2 Exact animation contract

- 60 ms per frame;
- random click choice between `JUMP_WAVE` and `LOOK_AROUND`;
- jump/wave crouches for two frames, raises arms for three, settles for one, and repeats;
- look-around holds right for five frames, left for five, then settles;
- no new animation starts while one is running;
- reduced-motion mode ignores animation triggers;
- fixed-height container prevents layout shift;
- small, large, and Apple Terminal-compatible presentations use the same pose state.

### 40.3 Surface adapters

Implement from the same canonical art/state machine:

- **TUI adapter**: React/Ink or selected TUI framework;
- **native terminal welcome adapter**: rendered in the app chrome while visually matching terminal glyphs;
- **command palette cameo**: static small pose only;
- **agent status adapter**: state-triggered, never continuously distracting;
- **website/docs adapter**: faithful SVG or HTML recreation generated from canonical pose data;
- **reduced-motion adapter**: instant pose transitions or static default.

Do not independently redraw each surface. Add snapshot tests comparing every pose with `art.ts`.

### 40.4 Personality rules

The mascot is:

- curious;
- alert;
- capable;
- slightly mischievous;
- reassuring during complex work.

The mascot is not:

- a replacement for error messages;
- an excuse for childish copy;
- constantly moving;
- used to pressure users into enabling AI, telemetry, or paid services;
- shown celebrating destructive or high-risk actions;
- allowed to obscure terminal content.

### 40.5 Placement

- prominent on first run and welcome screen;
- compact on empty agent/docs views;
- optional in the status area;
- absent or frozen during full-screen TUIs, presentations, and focused modes;
- never placed inside raw command output;
- hidden automatically at constrained terminal sizes;
- user can disable mascot visibility separately from reduced motion.

### 40.6 Mascot milestones

1. Preserve canonical source and provenance — complete.
2. Add unit tests for animation sequence and random selection.
3. Add pose snapshot tests.
4. Build the first OpenTerm TUI adapter.
5. Build native-client adapter after renderer selection.
6. Add theme tokens for body, eyes, and background.
7. Add reduced-motion and visibility settings.
8. Connect only meaningful agent/session events.
9. Add accessibility label and hide decorative glyph detail from screen readers.
10. Create brand guidelines and export assets for docs/site.

### 40.7 Welcome screen concept

```text
╭─ OpenTerm ───────────────────────────────────────────────────────────────╮
│                                                                        │
│                              ▐●‿●▌                                     │
│                             ▐ ● ● ▌                                    │
│                             ▐ ╰─╯ ▌                                    │
│                              ▝▀▀▀▘                                     │
│                              ▝▘ ▝▘                                     │
│                                                                        │
│                     Your terminal, with context.                       │
│                                                                        │
│  ~/project  main  ·  zsh  ·  RTK ready  ·  Docs index ready           │
╰────────────────────────────────────────────────────────────────────────╯

$ _
```

The actual UI should adapt to terminal width and avoid drawing the full frame during ordinary prompt use. The first impression is warm; the lasting impression must be speed.

---

## 41. `ai-*` command namespace and LLM connections

### 41.1 Namespace invariant

Every OpenTerm-native user-facing tool executable starts with `ai-`.

Examples:

- `ai-grep`, never a replacement executable named `grep`;
- `ai-read`, never `cat`;
- `ai-ssh`, never `ssh`;
- `ai-ask`, never an ambiguous `ask`;
- `ai-plan`, `ai-apply`, and `ai-verify` for explicit mutation stages.

The `openterm` executable is the application launcher and control plane. It may open the UI, manage the daemon, and show global help; it must not become an unprefixed collection of tool subcommands.

Internal protocol IDs remain semantic names such as `fs.read`, `code.search`, and `llm.ask`. They are not shell commands. The command registry maps each public `ai-*` executable to exactly one typed internal command.

### 41.2 Why a prefix is valuable

- never shadows trusted system tools;
- makes agent-generated commands visibly AI-oriented;
- allows simple capability and audit rules;
- supports discovery with shell completion over `ai-*`;
- prevents confusion between raw and compressed/semantic behavior;
- permits safe side-by-side comparison with Unix commands;
- makes plugin ownership conflicts detectable;
- creates a coherent OpenTerm vocabulary across shells and remote hosts.

### 41.3 Multicall implementation

Ship one Rust binary and link it under registered command names:

```text
openterm       -> application/control binary
ai-ask         -> openterm multicall link
ai-grep        -> openterm multicall link
ai-read        -> openterm multicall link
ai-test        -> openterm multicall link
```

At startup, the binary reads its executable name and dispatches to the central registry. Benefits:

- one installation and update;
- one configuration/policy engine;
- no duplicated provider or RTK initialization;
- consistent help, errors, result envelopes, and telemetry policy;
- plugins can register command descriptors without shipping arbitrary shell wrappers;
- command-name prefix can be enforced by tests.

Windows packaging may use small launch shims or copied executables if symbolic links are unsuitable. The public names remain identical.

### 41.4 Initial command catalog

| Public command | Internal command | Purpose | Delivery |
|---|---|---|---|
| `ai-ask` | `llm.ask` | Read-only terminal/workspace advice | Implemented foundation |
| `ai-doctor` | `system.doctor` | Tool and provider diagnostics | Implemented foundation |
| `ai-version` | `system.version` | Version information | Implemented foundation |
| `ai-grep` | `code.search` | RTK-backed grouped search | Implemented RTK slice |
| `ai-find` | `fs.find` | Compact bounded file discovery | Implemented RTK slice |
| `ai-tree` | `fs.tree` | Token-efficient workspace tree | Implemented RTK slice |
| `ai-read` | `fs.read` / `code.slice` | Bounded intelligent file reads | Implemented RTK slice |
| `ai-diff` | `data.diff` / `vcs.diff` | Compact Git diffs | Implemented RTK slice |
| `ai-test` | `test.run` | Failure-focused tests | Implemented RTK slice |
| `ai-status` | `vcs.status` | Compact repository status | Implemented RTK slice |
| `ai-log` | `log.inspect` | Deduplicated log inspection | Implemented RTK slice |
| `ai-run` | `exec.run` | Budgeted argv-safe execution | P1 |
| `ai-explain` | `llm.explain` | Explain selected output/result | P1 |
| `ai-expand` | `result.expand` | Expand omitted evidence | P1 |
| `ai-plan` | `action.plan` | Read-only effect plan | P1 |
| `ai-apply` | `action.apply` | Apply an approved bound plan | P2 |
| `ai-verify` | `action.verify` | Verify effects and postconditions | P2 |
| `ai-ssh` | `remote.run` | Policy-bound remote operation | P3 |

No plugin may register an unprefixed public executable. Plugins use reverse-domain ownership internally and the global registry rejects collisions.

### 41.5 `ai-ask` product contract

`ai-ask` answers questions such as:

```sh
ai-ask "how do I see free disk space here?"
ai-ask "what command shows the processes using the most CPU?"
ai-ask "propose how to organize this folder as a clean project"
```

Default semantics:

- read-only;
- concise answer first;
- exact commands in fenced blocks;
- explain important platform differences;
- inspect the current directory only when the selected provider has a read-only local tool surface;
- never execute suggested commands automatically;
- requests to modify or organize return a plan;
- clearly distinguish observation from inference;
- store no conversation unless the user opts into a session.

Future explicit modes:

- `ai-ask --general`: no workspace inspection;
- `ai-ask --workspace`: bounded read-only context pack;
- `ai-ask --result REF`: explain an exact stored result;
- `ai-ask --session`: retain a local conversation thread;
- `ai-ask --json`: normalized machine-readable answer envelope.

Do not add an `--apply` flag to `ai-ask`. Mutation belongs to the plan/apply/verify commands.

### 41.6 Provider architecture from OpenAgents

OpenAgents demonstrates a useful division:

1. a declarative provider profile owns identity, aliases, auth type, endpoint, and model discovery;
2. a protocol transport owns message conversion, request construction, and response normalization;
3. credential resolution, streaming, retries, and conversation state remain outside the profile;
4. provider-specific data stays in an extension field instead of polluting the common result;
5. Codex app-server is treated as a distinct transport rather than forced into a generic chat-completions adapter.

OpenTerm adopts the same shape in smaller Rust modules. The initial one-shot `ai-ask` contract returns text; the richer normalized streaming envelope lands only when the interactive agent surface consumes it.

### 41.7 Provider matrix

| Provider | Initial transport | Authentication | Rich transport |
|---|---|---|---|
| OpenAI | Responses API | Platform API key | Streaming Responses API |
| Claude | Messages API | API key; later workload identity | Streaming Messages API |
| Mistral | Chat Completions | Workspace API key | Streaming Chat Completions |
| Codex | `codex exec` | Codex-managed cached login or supported API auth | `codex app-server` JSON-RPC |

Direct API providers require explicit model selection until OpenTerm has a setup flow and persisted provider configuration. Do not silently chase a fast-moving “latest” model alias in production workflows.

### 41.8 Codex integration levels

#### Level 1 — `codex exec`

Appropriate for stateless `ai-ask`:

- final response on stdout;
- progress on stderr;
- ephemeral execution;
- read-only sandbox;
- optional JSONL event output;
- existing Codex authentication reused without copying credentials.

#### Level 2 — `codex app-server`

Appropriate for the OpenTerm agent pane:

- initialize/initialized handshake;
- threads and turns;
- incremental item events;
- approvals;
- interruption and steering;
- conversation history;
- version-generated TypeScript/JSON schemas;
- stdio or authenticated local socket transport.

OpenTerm should use stable app-server capabilities first and opt into experimental API fields only for a concrete feature.

#### Level 3 — Codex SDK

Evaluate for background automation and CI workflows where a long-lived rich client is unnecessary. Keep it separate from the interactive app-server connection.

### 41.9 Credentials and configuration

Credentials:

- `OPENAI_API_KEY`;
- `ANTHROPIC_API_KEY`;
- `MISTRAL_API_KEY`;
- Codex-managed login/authentication.

Rules:

- environment variables are acceptable for credentials, not behavioral preferences;
- provider/model defaults live in OpenTerm configuration;
- move API keys to OS-keychain secret refs when configuration UI lands;
- never expose values through `ai-doctor`, history, logs, results, crash reports, or model context;
- never place credentials on command lines;
- do not parse or duplicate Codex's credential cache;
- credentials are bound to known provider origins;
- custom endpoints require an explicit trust step before receiving a credential.

### 41.10 Provider delivery roadmap

#### Provider Phase 0 — one-shot foundation

- `ai-ask` multicall executable;
- Codex `exec` read-only provider;
- OpenAI Responses request/response parser;
- Claude Messages request/response parser;
- Mistral Chat Completions request/response parser;
- explicit API models;
- missing-credential and parser tests;
- `ai-doctor` configuration status.

#### Provider Phase 1 — robust HTTP transport

- shared timeout and cancellation;
- bounded response bodies;
- structured status/rate-limit errors;
- origin-locked redirects;
- retries only for classified transient failures;
- request IDs and normalized usage;
- mock-server integration tests;
- optional SSE streaming.

#### Provider Phase 2 — setup and secrets

- `ai-provider list|add|login|test|remove` command family;
- OS keychain storage;
- live model discovery;
- persisted provider/model selection;
- per-provider base URL and trust UI;
- credential rotation and logout;
- no secret values in TOML.

#### Provider Phase 3 — workspace context

- bounded context packs;
- exact result/file references;
- provider-specific prompt caching;
- context explanation and token estimates;
- no raw scrollback injection;
- redaction before context construction.

#### Provider Phase 4 — rich Codex client

- app-server daemon adapter;
- schema generation pinned to supported Codex version;
- thread and turn mapping;
- event projection into OpenTerm command blocks;
- approval bridge into OpenTerm policy UI;
- cancellation, steering, and recovery;
- local-socket authentication for remote attachment.

#### Provider Phase 5 — safe actions

- `ai-plan` produces effect-typed hash-bound plans;
- `ai-apply` requires an unchanged approved plan;
- `ai-verify` gathers postcondition evidence;
- direct `ai-ask` remains non-mutating;
- agent tools are capability-gated and absent when not needed.

### 41.11 Provider quality gates

- no API request without an explicit provider and resolved model;
- no credential printed or persisted;
- parser fixtures for success, empty output, refusal, truncation, malformed body, and provider error;
- cancellation stops local processing and network/child work;
- exit status distinguishes usage, configuration, provider, and policy errors;
- all command names pass the `ai-` invariant test;
- API providers work without Codex installed;
- Codex works without API-provider keys;
- terminal and deterministic `ai-*` commands work with no LLM provider;
- live-provider tests remain opt-in and never run with secrets against untrusted PR code.
