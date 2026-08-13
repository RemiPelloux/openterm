# OpenTerm — Decision-Complete Product and Engineering Roadmap

**Revision:** 5

**Updated:** 2026-07-03

**License:** Apache-2.0

**Platforms:** macOS first, Linux before 1.0

**Terminal engine:** pinned Ghostty fork

**Core language:** Rust

**Public command invariant:** every tool command begins with `ai-`

## 1. Product definition

OpenTerm is a local-first terminal application designed for humans and AI
agents. It combines a native terminal, deterministic output compression,
lossless evidence storage, optional LLM guidance, persistent sessions, safe
workflows, and capability-restricted plugins.

The product does not replace Unix, Git, SSH, shells, or mature parsers. It
wraps them with bounded output, typed results, stable references, side-effect
metadata, and approval gates.

### 1.1 Product promise

OpenTerm 1.0 must:

- work as a reliable daily terminal, including full-screen TUI applications;
- reduce agent-facing output by at least 60% on the benchmark suite;
- preserve exact evidence behind expandable local references;
- work without an LLM for all deterministic terminal operations;
- keep terminal data local unless the user explicitly calls a provider;
- recover running sessions after a UI crash;
- provide inspectable plans before filesystem, Git, or remote mutation;
- support local plugins and system OpenSSH;
- pass accessibility, security, compatibility, and performance gates.

### 1.2 1.0 boundaries

Included:

- native macOS and Linux terminal clients;
- local sessions, tabs, panes, command blocks, and searchable history;
- RTK-derived deterministic compression and lossless raw capture;
- filesystem, search, Git, development, system, documentation, and SSH tools;
- OpenAI, Claude, Mistral, and Codex CLI connectivity;
- low-code workflows with approvals and deterministic replay;
- local WASI plugins;
- themes, profiles, completion, keybindings, and mascot states.

Excluded from terminal core:

- Docker, Kubernetes, databases, cloud consoles, and fleet orchestration;
- a hosted OpenTerm account, sync service, or plugin marketplace;
- arbitrary shell embedded inside workflow YAML;
- automatic execution of natural-language mutation requests;
- custom SSH cryptography or transport.

The existing `ai-docker` adapter remains an experimental RTK compatibility
command. It does not expand the 1.0 core scope.

## 2. Current repository state

The source tree currently provides a real macOS terminal alpha and a Rust
multicall tool binary.

### 2.1 Shipped foundations

- Ghostty-based macOS application with Metal rendering, PTYs, tabs, and splits.
- Native launcher, mascot splash, SwiftUI appearance editor, and shell wrapper.
- zsh and Bash integration with Git-aware prompts and `ai-cd`.
- encrypted SQLite evidence store with stable `res_*` identifiers.
- result expansion, paging, export, pinning, recoverable trash, and history.
- RTK 0.43 compatibility bridge with single execution and evidence capture.
- OpenAI, Claude, Mistral, and Codex CLI advice transports.
- interactive provider setup and model selection for direct API providers.
- command/help registry invariants and Pelloux verification.

### 2.2 Shipped command inventory

The registry contains 63 public commands: 24 OpenTerm-native commands and 39
RTK-backed commands.

| Area | Shipped commands |
|---|---|
| Guidance | `ai-help`, `ai-ask`, `ai-chat`, `ai-explain`, `ai-panel`, `ai-workspace` |
| Providers | `ai-connect` |
| Product | `ai-doctor`, `ai-version`, `ai-context`, `ai-ui` |
| Evidence | `ai-run`, `ai-result`, `ai-expand`, `ai-page`, `ai-export`, `ai-pin`, `ai-forget` |
| History | `ai-history`, `ai-search-history`, `ai-clean-history`, `ai-usage` |
| Files/search | `ai-rg`, `ai-grep`, `ai-find`, `ai-tree`, `ai-read`, `ai-ls`, `ai-wc` |
| Data/output | `ai-json`, `ai-err`, `ai-summary`, `ai-smart`, `ai-env`, `ai-deps`, `ai-gain` |
| Git/logs | `ai-status`, `ai-diff`, `ai-log` |
| General development | `ai-test`, `ai-lint`, `ai-format`, `ai-prettier` |
| TypeScript/JavaScript | `ai-tsc`, `ai-jest`, `ai-vitest`, `ai-playwright`, `ai-npm`, `ai-pnpm` |
| Python | `ai-pytest`, `ai-ruff`, `ai-mypy` |
| Rust | `ai-cargo` |
| Go | `ai-go`, `ai-golangci` |
| Ruby | `ai-rspec`, `ai-rubocop` |
| JVM/.NET | `ai-gradle`, `ai-maven`, `ai-dotnet` |
| Experimental | `ai-docker` |

### 2.3 Confirmed technical debt

These are known facts, not speculative features:

1. RTK-backed processes are buffered with `Command::output`; large streams are
   not yet bounded and interactive progress is delayed.
2. Stored evidence for RTK routes contains RTK-filtered output, not the child
   process's pre-filter raw stream.
3. The evidence encryption key is a mode-0600 file, not an OS-keystore secret.
4. Direct-provider API keys are stored in a mode-0600 JSON file until the
   Keychain/Secret Service migration lands.
5. The terminal has no OpenTerm-owned session daemon or crash recovery yet.
6. Shell hooks do not yet emit trustworthy command start/end boundaries.
7. Results have metadata and evidence, but not the complete versioned typed
   envelope required by workflows and plugins.
8. The documentation agent, safe mutation engine, plugin host, and SSH layer
   are not implemented.
9. macOS builds are ad-hoc signed and not notarized.
10. The local Cargo cache contains a root-owned registry entry that produces a
    cleanup warning; repository verification still succeeds.

### 2.4 Immediate product gate

Do not add broad mutation, remote, cloud, or plugin execution before streaming
capture, secret storage, result schemas, and policy decisions are stable.

## 3. Architecture

### 3.1 Selected approach

Three approaches were evaluated:

1. extend the Rust foundation and embed pinned Ghostty components;
2. use tmux and RTK wrappers as the permanent architecture;
3. build terminal emulation, rendering, PTYs, and compression from scratch.

Approach 1 remains selected. Ghostty supplies mature terminal behavior; Rust
owns OpenTerm sessions, evidence, policies, providers, workflows, and plugins.
tmux remains an optional import/compatibility backend. Rebuilding terminal
emulation adds years of compatibility work without improving the AI layer.

### 3.2 Target process topology

```text
OpenTerm.app / OpenTerm Linux client
        │ control messages + ordered terminal frames
        ▼
opentermd
  ├── PTY/session manager
  ├── command registry and policy engine
  ├── evidence/result store
  ├── native compression engine
  ├── workflow and documentation agents
  ├── provider transports
  └── WASI plugin host
        │
        ├── local processes and system tools
        ├── system OpenSSH
        └── explicit provider network calls
```

### 3.3 Component ownership

| Component | Responsibility |
|---|---|
| `openterm` | Application launcher and multicall CLI |
| `opentermd` | PTYs, sessions, jobs, history, policies, and agents |
| `openterm-core` | Schemas, registry, references, effects, budgets |
| `openterm-compress` | RTK-derived parsers and deterministic compact views |
| `openterm-store` | SQLite metadata, encrypted blobs, migrations, retention |
| macOS client | SwiftUI/AppKit, Metal, embedded Ghostty, accessibility |
| Linux client | GTK4, Ghostty rendering path, desktop integration |
| plugin host | WASI components, WIT interfaces, capability broker |

The current single crate should split only when daemon and compression work
create real ownership boundaries. Empty crate scaffolding is not a milestone.

### 3.4 Local protocol

Use a versioned Unix-socket protocol:

- JSON or MessagePack control messages during alpha;
- ordered binary terminal frames for throughput;
- `session_id`, `pane_id`, `sequence`, `kind`, and `length` on every frame;
- request IDs and cancellation on every long-running operation;
- feature negotiation and protocol version at connection time;
- replay from the last acknowledged sequence after reconnect;
- bounded queues with explicit dropped-frame diagnostics.

Do not expose the daemon on TCP in 1.0.

### 3.5 Persistence

SQLite WAL stores metadata. Encrypted blobs store:

- stdout and stderr segments;
- ordered terminal event segments;
- raw and compact command views;
- workflow plans, approvals, and verification records;
- documentation indexes and citations;
- session, pane, command, and result relationships.

macOS Keychain and Linux Secret Service store provider credentials and the
evidence master key. SQLite, logs, argv, prompts, and workflows never contain
secret material.

## 4. Result and execution contract

### 4.1 Result envelope

Every managed command eventually returns:

```json
{
  "schema": "openterm.result/v1",
  "id": "res_...",
  "command": "ai-rg",
  "status": "ok",
  "summary": "14 matches in 5 files",
  "data": {},
  "diagnostics": [],
  "refs": [],
  "effects": {"filesystem": "read", "network": "none"},
  "omitted": {"bytes": 28410, "reason": "budget"},
  "metrics": {"duration_ms": 31, "raw_bytes": 30120, "returned_bytes": 1710},
  "provenance": {"adapter": "rtk", "version": "0.43.0"},
  "next": ["ai-expand res_..."]
}
```

Human output is a rendering of this envelope. `--json` returns the stable
schema. Compact output must never be the only surviving output.

### 4.2 Reference types

| Prefix | Meaning |
|---|---|
| `res_*` | immutable command result |
| `view_*` | derived compact or queried result view |
| `file_*` | file content at a known hash |
| `span_*` | exact byte, line, or syntax range |
| `snap_*` | workspace or directory snapshot |
| `patch_*` | proposed change set with preconditions |
| `proc_*` | managed process |
| `job_*` | workflow execution |
| `session_*` | terminal session |
| `host_*` | configured remote host |
| `artifact_*` | build, report, archive, or generated output |

### 4.3 Effect classes

| Class | Examples | Default policy |
|---|---|---|
| Pure | parsing stored JSON | automatic |
| Read | search, list, inspect Git | automatic inside workspace |
| Local write | create, patch, format | preview and explicit approval |
| Destructive | remove, overwrite, kill | preview, explicit approval, recovery metadata |
| Network read | fetch docs, provider call | explicit command and visible destination |
| Network write | push, publish, remote mutation | explicit approval per target |
| Privileged | ownership, system config | never implicit; OS authorization visible |

## 5. Command design rules

1. Every new executable begins with `ai-`.
2. Never shadow `grep`, `cd`, `ssh`, or another system executable.
3. `ai-cd` remains a shell function because child processes cannot change the
   parent shell directory.
4. Prefer a typed native command when progressive disclosure is the product.
5. Prefer a direct argv adapter when Git, OpenSSH, or a mature tool owns the
   difficult semantics.
6. Never invoke a shell unless the user explicitly requests shell parsing.
7. Every large result gets a reference and an exact expansion path.
8. Read-only commands may run automatically; mutation uses plan/apply/verify.
9. Command names describe intent, not implementation.
10. Avoid collisions: Git history uses `ai-git-log`; `ai-log` remains log
    inspection. Workflow execution uses `ai-flow-run`; `ai-run` remains process
    execution.

## 6. Canonical command roadmap

Status markers:

- **Now**: implemented in the repository.
- **Next**: required before session-daemon work expands.
- **1.0**: committed to the 1.0 scope.
- **Plugin**: intentionally outside terminal core.

### 6.1 Assistance and providers

| Command | Status | Contract |
|---|---|---|
| `ai-ask` | Now | read-only answer or executable suggestion |
| `ai-explain` | Now | explain stored evidence or last result |
| `ai-connect` | Now | configure providers; Codex auth stays in Codex CLI |
| `ai-panel` | Now | open native agent panel |
| `ai-chat` | Now | in-terminal mascot chat; read-only workspace context |
| `ai-plan` | 1.0 | produce typed, hash-bound effect plan |
| `ai-apply` | 1.0 | apply one approved plan reference |
| `ai-verify` | 1.0 | verify plan postconditions |
| `ai-undo` | 1.0 | apply available inverse operation with preview |
| `ai-provider` | 1.0 | list, select, and diagnose providers |
| `ai-model` | 1.0 | list and select discovered models |
| `ai-auth` | 1.0 | open provider-specific authentication flow |

### 6.2 Results and context economy

| Command | Status | Contract |
|---|---|---|
| `ai-result` | Now | inspect result metadata |
| `ai-expand` | Now | return exact evidence |
| `ai-page` | Now | bounded byte-range retrieval |
| `ai-export` | Now | materialize evidence to a file |
| `ai-pin` | Now | protect evidence from cleanup |
| `ai-forget` | Now | move evidence to recoverable trash |
| `ai-usage` | Now | local store and expansion statistics |
| `ai-restore` | Now | restore a trashed result |
| `ai-purge` | Next | permanently delete approved expired trash |
| `ai-query` | 1.0 | query structured result data |
| `ai-compare` | 1.0 | compare two results semantically |
| `ai-compact` | 1.0 | create a deterministic bounded view |
| `ai-redact` | 1.0 | create a shareable derived view without secrets |
| `ai-context-pack` | 1.0 | assemble cited evidence under a token budget |

### 6.3 Filesystem and reading

| Command | Status | Contract |
|---|---|---|
| `ai-workspace` | Now | cwd, repository, and agent context |
| `ai-context` | Now | branch and working-tree state |
| `ai-ls` | Now | compact directory listing |
| `ai-tree` | Now | bounded directory tree |
| `ai-find` | Now | compact file discovery |
| `ai-read` | Now | bounded intelligent file read |
| `ai-wc` | Now | compact counts |
| `ai-pwd` | 1.0 | workspace root, cwd, relative location, capabilities |
| `ai-cd` | Now | parent-shell directory change |
| `ai-stat` | 1.0 | type, size, hash, encoding, ownership, Git state |
| `ai-du` | 1.0 | ranked disk hot spots and cleanup candidates |
| `ai-head` | 1.0 | bounded start with semantic boundaries |
| `ai-tail` | 1.0 | bounded tail with follow/delta mode |
| `ai-outline` | 1.0 | symbols, imports, exports, and line spans |
| `ai-symbol` | 1.0 | definition and local references |

### 6.4 Search and structured data

| Command | Status | Contract |
|---|---|---|
| `ai-rg` | Now | preferred compact ripgrep search |
| `ai-grep` | Now | legacy grouped grep compatibility |
| `ai-json` | Now | compact JSON rendering |
| `ai-summary` | Now | deterministic heuristic summary |
| `ai-smart` | Now | compact technical summary |
| `ai-err` | Now | retain errors and warnings |
| `ai-count` | 1.0 | typed counts and token estimates |
| `ai-sort` | 1.0 | stable typed sort |
| `ai-uniq` | 1.0 | unique values with counts |
| `ai-select` | 1.0 | schema-aware field selection |
| `ai-yaml` | 1.0 | parse/query/render YAML safely |
| `ai-csv` | 1.0 | typed rows, schema inference, bounded previews |
| `ai-search-change` | 1.0 | search only changed files or lines |
| `ai-find-secret` | 1.0 | local masked secret-pattern scan |
| `ai-find-dead` | 1.0 | ranked unused-code candidates with evidence |

### 6.5 Safe file changes

These commands cannot bypass `ai-plan` and approval policy.

| Command | Status | Contract |
|---|---|---|
| `ai-mkdir` | 1.0 | idempotent directory creation plan |
| `ai-touch` | 1.0 | idempotent file creation plan |
| `ai-copy` | 1.0 | collision-aware copy with hashes |
| `ai-move` | 1.0 | reference-aware move/rename |
| `ai-remove` | 1.0 | quarantine by default; permanent is explicit |
| `ai-permissions` | 1.0 | symbolic before/after permission change |
| `ai-patch` | 1.0 | atomic patch with expected hashes |
| `ai-replace` | 1.0 | constrained literal/regex replacement preview |

### 6.6 Git

| Command | Status | Contract |
|---|---|---|
| `ai-status` | Now | compact status |
| `ai-diff` | Now | compact diff with expandable evidence |
| `ai-git-log` | 1.0 | ranked compact commit history |
| `ai-show` | 1.0 | commit summary before bounded patch |
| `ai-blame` | 1.0 | bounded line provenance |
| `ai-branch` | 1.0 | branch/worktree relationships |
| `ai-stage` | 1.0 | explicit file or hunk staging plan |
| `ai-commit` | 1.0 | staged-scope validation and commit |
| `ai-conflicts` | 1.0 | conflict inventory and resolution evidence |
| `ai-impact` | 1.0 | changed symbols, packages, tests, and risks |
| `ai-review` | 1.0 | bounded review packet with verification |
| `ai-git-sync` | 1.0 | explicit fetch/pull/push target and effects |

### 6.7 Development and diagnostics

Already shipped wrappers cover Rust, Go, Python, JavaScript/TypeScript, Ruby,
JVM, and .NET. All use RTK filtering and OpenTerm evidence capture.

| Command | Status | Contract |
|---|---|---|
| `ai-test` | Now | failure-focused tests |
| `ai-lint`, `ai-format`, `ai-prettier` | Now | compact diagnostics |
| `ai-tsc`, `ai-jest`, `ai-vitest`, `ai-playwright` | Now | JS/TS diagnostics |
| `ai-pytest`, `ai-ruff`, `ai-mypy` | Now | Python diagnostics |
| `ai-cargo`, `ai-go`, `ai-golangci` | Now | Rust/Go diagnostics |
| `ai-rspec`, `ai-rubocop` | Now | Ruby diagnostics |
| `ai-gradle`, `ai-maven`, `ai-dotnet` | Now | JVM/.NET diagnostics |
| `ai-npm`, `ai-pnpm` | Now | compact package-script output |
| `ai-deps` | Now | project dependency summary |
| `ai-build` | 1.0 | detect and run project build with budgets |
| `ai-check` | 1.0 | detect repository verification commands |
| `ai-coverage` | 1.0 | changed-code coverage and uncovered spans |
| `ai-affected` | 1.0 | tests/packages affected by refs or patches |
| `ai-bench` | 1.0 | compare performance result references |
| `ai-diagnose` | 1.0 | correlate compiler, test, and runtime failures |

### 6.8 Processes and system

| Command | Status | Contract |
|---|---|---|
| `ai-run` | Now | argv-safe managed execution and evidence |
| `ai-env` | Now | filtered environment view |
| `ai-which` | 1.0 | path, version, origin, and trust source |
| `ai-ps` | 1.0 | grouped process tree and resource ranking |
| `ai-top` | 1.0 | sampled deltas rather than repaint streams |
| `ai-kill` | 1.0 | identity check and graceful escalation plan |
| `ai-disk` | 1.0 | mount capacity and warnings |
| `ai-ports` | 1.0 | listeners, processes, and ownership |
| `ai-net` | 1.0 | bounded connectivity diagnostics |
| `ai-jobs` | 1.0 | managed background job inventory |
| `ai-watch` | 1.0 | delta-only repeated execution with stop conditions |

### 6.9 Sessions and terminal control

| Command | Status | Contract |
|---|---|---|
| `ai-session` | 1.0 | list/create/name/close sessions |
| `ai-pane` | 1.0 | split, focus, move, and close panes |
| `ai-window` | 1.0 | list/create/focus windows |
| `ai-attach` | 1.0 | attach to a persistent session |
| `ai-detach` | 1.0 | detach without terminating processes |
| `ai-replay` | 1.0 | replay command or session evidence safely |
| `ai-profile` | 1.0 | manage shell/environment profiles |

### 6.10 History and memory

| Command | Status | Contract |
|---|---|---|
| `ai-history` | Now | recent scored commands |
| `ai-search-history` | Now | local history search |
| `ai-clean-history` | Now | explain cleanup candidates only |
| `ai-history-pin` | 1.0 | protect a command/session relationship |
| `ai-history-export` | 1.0 | redact and export selected history |
| `ai-history-quota` | 1.0 | configure opt-in storage quota |
| `ai-memory-note` | 1.0 | explicit workspace fact with provenance |
| `ai-memory-forget` | 1.0 | invalidate one explicit fact |

### 6.11 Documentation agent

| Command | Status | Contract |
|---|---|---|
| `ai-doc-index` | 1.0 alpha | incrementally index Git-tracked code/docs |
| `ai-doc-map` | 1.0 alpha | modules, symbols, manifests, ownership |
| `ai-doc-ask` | 1.0 alpha | answer with file/line citations |
| `ai-doc-draft` | 1.0 alpha | draft a reviewable documentation patch |
| `ai-doc-check` | 1.0 alpha | detect stale or contradictory docs |
| `ai-doc-links` | 1.0 | validate internal links and cited symbols |
| `ai-doc-onboard` | 1.0 | generate cited onboarding path |

The index uses Tree-sitter, manifests, Git history, ripgrep, and SQLite FTS5.
It must work deterministically without embeddings. Providers may improve
synthesis but cannot replace citations.

### 6.12 Workflows

| Command | Status | Contract |
|---|---|---|
| `ai-flow` | 1.0 | inspect and validate workflow files |
| `ai-flow-run` | 1.0 | execute a typed workflow |
| `ai-flow-cancel` | 1.0 | cancel a running job |
| `ai-flow-resume` | 1.0 | resume from persisted safe checkpoint |
| `ai-flow-replay` | 1.0 | deterministic replay against bound inputs |

Workflow schema:

```yaml
schema: openterm.workflow/v1
name: fix-and-test
capabilities:
  filesystem: workspace-write
  network: none
inputs:
  issue: string
steps:
  - id: search
    command: ai-rg
    args: {query: "${input.issue}", path: src}
  - id: inspect
    command: ai-read
    args: {refs: "${steps.search.data.top_files}"}
  - id: test
    command: ai-test
    args: {impacted_by: "${steps.inspect.refs}"}
```

V1 supports typed inputs, references, conditions, bounded iteration, approval
checkpoints, cancellation, resumption, and deterministic replay. It does not
support arbitrary embedded shell.

### 6.13 Plugins

| Command | Status | Contract |
|---|---|---|
| `ai-plugin` | 1.0 | list, inspect, enable, disable plugins |
| `ai-plugin-install` | 1.0 | local package install with checksum review |
| `ai-plugin-remove` | 1.0 | remove plugin while retaining audit metadata |
| `ai-plugin-permissions` | 1.0 | inspect and change granted capabilities |
| `ai-plugin-doctor` | 1.0 | compatibility and resource diagnostics |
| `ai-plugin-new` | 1.0 | scaffold Rust or TypeScript plugin |

Extension points: command, parser, compressor, docs source, provider, theme,
completion source, and declarative panel. No native dynamic libraries in 1.0.

### 6.14 SSH and remote operation

| Command | Status | Contract |
|---|---|---|
| `ai-host` | 1.0 | inspect host profiles and effective SSH config |
| `ai-ssh` | 1.0 | system OpenSSH PTY or structured remote command |
| `ai-scp` | 1.0 | previewed transfer with hashes |
| `ai-sync` | 1.0 | rsync dry-run, deletion warning, explicit apply |

OpenTerm uses the system OpenSSH client, strict host-key verification, existing
agents/keychains, and `~/.ssh/config`. It never silently accepts a changed host
key or performs a remote write.

### 6.15 Product and appearance

| Command | Status | Contract |
|---|---|---|
| `ai-ui` | Now | native appearance settings |
| `ai-doctor` | Now | environment and provider diagnostics |
| `ai-version` | Now | build/version information |
| `ai-theme` | 1.0 | list, preview, import, and select themes |
| `ai-keybind` | 1.0 | inspect conflicts and edit keybindings |
| `ai-complete` | 1.0 | install/diagnose semantic completion |
| `ai-config` | 1.0 | typed configuration inspection and edits |
| `ai-update` | 1.0 | signed update check and explicit install |

## 7. Native compression roadmap

### 7.1 Required pipeline

```text
child process
  → ordered lossless capture
  → encrypted segmented storage
  → typed parser
  → deterministic compressor
  → terminal/agent compact view
  → stable expansion references
```

The process runs once. Raw capture and compression must never invoke the child
twice.

### 7.2 Delivery sequence

1. Add streaming stdout/stderr readers with bounded channels.
2. Store ordered segments before filtering.
3. Pin compressor identity, version, and options in provenance.
4. Import Apache-2.0 RTK filtering modules with attribution.
5. Add upstream golden fixtures and intentional-deviation manifests.
6. Render compact views from stored raw evidence.
7. Add byte/token savings and decisive-diagnostic preservation tests.
8. Keep RTK CLI fallback until native compatibility passes.

### 7.3 Compaction algorithms

Use simple measurable methods:

- exact duplicate folding with counts;
- structured grouping by file, test, diagnostic, or process;
- severity and recency ranking;
- head/tail and boundary preservation;
- maximal marginal relevance only where duplicate groups dominate;
- explicit byte, item, time, and estimated-token budgets;
- extractive summaries before any optional LLM synthesis.

Every omitted region records its count, reason, and expansion command.

## 8. Smart history and retention

History remains persistent by default. Automatic cleanup is disabled until the
user explicitly configures a quota.

The existing score combines exponential freshness decay with access frequency,
pinning, references, diagnostic value, and uniqueness. Future cleanup flow:

1. protect pinned, referenced, recent, audit, and unresolved-error results;
2. rank low-value duplicates first;
3. show the exact reclaim plan;
4. move approved data to encrypted trash;
5. retain trash for 30 days;
6. audit permanent deletion locally.

Do not market the ranking model as “AI.” It is deterministic retention policy.

## 9. Exceptional UI/UX roadmap

### 9.1 Core layout

- terminal canvas remains primary;
- left navigator optionally shows workspaces and sessions;
- right sidecar hosts Ask, Docs, Plan, and Evidence;
- bottom drawer shows result diagnostics and job progress;
- command palette reaches every action without memorizing shortcuts.

### 9.2 Command blocks

Create blocks only from trusted shell integration events. Each block shows:

- command, cwd, shell, start/end time, and exit state;
- compact output and savings;
- evidence reference and expansion control;
- copy, explain, rerun, pin, export, and compare actions;
- local/remote and side-effect indicators.

Alternate-screen programs keep unmodified terminal behavior.

### 9.3 Motion and mascot

Byte, the original OpenTerm mascot in `design/mascot/`, is the product mascot.
States: idle, thinking, running, success, warning, failure, interrupted, and
sleeping. Motion never delays input, steals focus, or obscures output. Reduced
motion replaces animation with static state changes.

### 9.4 Personalization

- profiles, fonts, ligatures, opacity, blur, backgrounds, and padding;
- light/dark synchronization and accessible contrast validation;
- theme import/export with no executable theme code;
- semantic autocomplete from schemas, files, Git, and local history;
- keybinding conflict detection and searchable settings;
- screen-reader labels and complete keyboard operation.

### 9.5 UX performance gates

- input-to-render latency within one 60 Hz frame at p95;
- sidecar work never blocks terminal rendering;
- settings changes preview within 100 ms;
- daemon reconnect under one second in normal conditions;
- opening history with 100,000 commands under 300 ms p95;
- no mascot or blur animation during measured typing latency.

## 10. Security roadmap

### 10.1 Before the next distributable alpha

- migrate provider keys and evidence master key to OS secret stores;
- remove legacy plaintext secrets during an explicit migration;
- redact secret-bearing argv flags before persistence;
- bind provider credentials to exact origins and keep redirects disabled;
- cap provider, command, and parser memory;
- record all mutation approvals and remote targets;
- sign result metadata with blob hashes and verify on read.

### 10.2 Before plugins

- capability declarations for filesystem, processes, network, secrets, UI,
  result access, and provider calls;
- WASI component isolation and resource ceilings;
- package checksum and manifest review;
- no ambient filesystem, environment, or network access;
- hostile-plugin tests for escapes, exhaustion, and undeclared capabilities.

### 10.3 Before SSH

- strict host-key change behavior;
- visible local-versus-remote state;
- quoted argv tests across supported shells and hosts;
- transfer hash verification;
- remote mutation approvals and disconnect recovery.

## 11. Delivery plan

Estimates assume a small focused team. A solo build is likely 12–18 months.

### Phase 0 — Alpha hardening (1–2 sprints)

Deliver:

- OS-keystore credential and evidence-key storage;
- migration away from plaintext provider secrets;
- streaming process capture with bounded memory;
- `ai-purge` and the 30-day trash policy (`ai-restore` is complete);
- generated command registry/help/link consistency;
- benchmark corpus and current savings baseline;
- reproducible local install/update target.

Exit:

- no copied Codex tokens or embedded OAuth client credentials;
- no unbounded managed-command buffering;
- all 61 current commands have help, routing, prefix, and evidence tests;
- a clean `make verify`, release build, app build, install, and launch smoke test.

### Phase 1 — Native evidence and compression (2–3 sprints)

Deliver:

- versioned result envelope and stable JSON output;
- ordered raw stdout/stderr segments;
- native RTK-derived compressors with golden compatibility;
- query, compare, compact, and redacted derived views;
- exact token/byte savings metrics.

Exit:

- every managed command returns compact output plus recoverable raw evidence;
- median benchmark reduction reaches 60%;
- decisive failure diagnostics are never available only in omitted output.

### Phase 2 — Session daemon (4–5 sprints)

Deliver:

- `opentermd`, Unix-socket protocol, PTY ownership, process groups, resize,
  signals, cancellation, and ordered frames;
- sessions, panes, windows, attach/detach, jobs, and crash recovery;
- zsh/Bash command-boundary hooks and fish support;
- optional tmux session import.

Exit:

- UI restart does not terminate running processes;
- attach/reconnect preserves frame order and input state;
- terminal conformance tests pass for common shells and TUIs.

### Phase 3 — Native command experience (3–4 sprints)

Deliver:

- semantic command blocks and evidence inspector;
- AI/docs sidecar and command palette;
- semantic completion and searchable settings;
- profiles, theme import, keybindings, accessibility, and all mascot states;
- Linux GTK4 client foundation.

Exit:

- OpenTerm can replace an ordinary local terminal for daily use;
- keyboard-only, reduced-motion, contrast, and screen-reader checks pass.

### Phase 4 — Documentation agent alpha (3–4 sprints)

Deliver:

- incremental Git-tracked index;
- symbol, file, dependency, ownership, and evidence graph;
- cited answers, stale-doc checks, onboarding, and patch drafts;
- deterministic FTS5 retrieval with optional provider synthesis.

Exit:

- every factual answer cites an exact file/line span;
- index updates incrementally after Git/file changes;
- drafts apply only through reviewable patches.

### Phase 5 — Safe actions and workflows (4–5 sprints)

Deliver:

- plan/apply/verify/undo protocol;
- filesystem mutation and Git write commands;
- workflow parser, typed references, conditions, bounded loops, checkpoints,
  cancellation, and resume;
- impacted-test selection and rollback metadata.

Exit:

- natural-language mutation requests become plans, never implicit writes;
- stale hashes stop application;
- happy, negative, cancellation, and partial-failure paths are tested.

### Phase 6 — Plugin SDK (3–4 sprints)

Deliver:

- WASI component host and stable WIT contracts;
- local package manager and permission review;
- Rust and TypeScript SDKs;
- command, parser, compressor, docs, provider, theme, completion, and panel
  examples.

Exit:

- third parties add commands without modifying core;
- undeclared capabilities and resource exhaustion are contained.

### Phase 7 — SSH and remote workflows (3–4 sprints)

Deliver:

- host profiles, effective config inspection, remote PTYs, structured results;
- transfer/sync previews, hashes, remote evidence, and reconnect behavior;
- latency, disconnect, host-key-change, and quoting tests.

Exit:

- remote operation is explicit, inspectable, and reliable enough for 1.0.

### Phase 8 — 1.0 hardening (4–6 sprints)

Deliver:

- macOS/Linux soak and conformance testing;
- storage migration and corruption recovery;
- independent security and accessibility audits;
- signed/notarized macOS and signed Linux packages;
- updater, complete docs, plugin SDK docs, and troubleshooting;
- public-name clearance for “OpenTerm.”

Exit:

- all product promises and performance gates pass;
- no critical known regression or unresolved security blocker remains.

## 12. Next 30 implementation issues

Ordered by dependency, not excitement:

1. Add a benchmark fixture runner and record current raw/returned bytes.
2. Define `openterm.result/v1` Rust types and JSON fixtures.
3. Replace buffered process output with ordered bounded stream segments.
4. Preserve partial evidence on signal, timeout, and cancellation.
5. Add raw/compact provenance and executable-version metadata.
6. Move the evidence master key to macOS Keychain.
7. Add the Linux Secret Service backend and explicit file fallback for tests.
8. Move direct-provider API keys to OS secret storage.
9. Add a migration that removes legacy secrets from `providers.json`.
10. Add argv redaction schemas for common secret-bearing flags.
11. **Completed:** implement `ai-restore` with paired-blob rollback.
12. Implement approved `ai-purge` and 30-day trash expiry.
13. Generate command links and help from one registry representation.
14. Add a compatibility test for all 61 public command links.
15. Import the first RTK parser behind `openterm-compress`.
16. Add upstream golden fixtures and an intentional-deviation manifest.
17. Render compact views from stored raw evidence.
18. Add `ai-query` and immutable `view_*` references.
19. Add `ai-compare` for command/test result deltas.
20. Define daemon protocol versioning and feature negotiation.
21. Implement PTY create, resize, input, signal, and exit events.
22. Persist session/pane metadata and ordered frame checkpoints.
23. Add reconnect and sequence-gap recovery tests.
24. Emit zsh/Bash command start/end integration events.
25. Build the first semantic command block in the macOS client.
26. Connect the evidence inspector to a selected command block.
27. Add `ai-doc-index` with Git ignore and incremental hash behavior.
28. Add Tree-sitter symbol extraction for Rust, Swift, TypeScript, and Python.
29. Add `ai-doc-ask` with mandatory local citations.
30. Produce a signed alpha release checklist and installation smoke test.

## 13. Verification strategy

### 13.1 Required automated tests

- unit/property tests for schemas, references, budgets, and policies;
- golden tests for every compressor;
- raw-versus-compact evidence equivalence;
- success, failure, signal, timeout, cancellation, and partial output;
- Unicode, colors, mouse, clipboard, images, resize, and alternate screen;
- daemon/UI crash, reconnect, sequence gap, and migrations;
- provider mock servers for streaming, malformed data, rate limits, auth expiry,
  redirects, and cancellation;
- documentation citation accuracy and stale-index behavior;
- plugin escape, capability, version, and resource tests;
- SSH host keys, disconnects, quoting, transfer integrity, and approval;
- UI snapshots, keyboard-only, reduced motion, contrast, and screen readers;
- happy and negative paths for every behavior change.

### 13.2 Performance gates

- terminal input/rendering within one 60 Hz frame at p95;
- daemon reconnect under one second;
- compression under 10 ms p95 for common small commands;
- large output streamed with bounded memory;
- median agent-output reduction of at least 60%;
- no decisive diagnostic preserved only in discarded output;
- cold `ai-version` and `ai-context` under 30 ms on release builds;
- no network call during ordinary terminal startup.

### 13.3 Pelloux enforcement

- production files target 300 lines and fail above 400;
- functions target 25 lines and accept at most four parameters;
- no speculative abstraction, debug output, commented-out code, or magic
  constants;
- each non-trivial slice records three approaches and selects the simplest
  complete fix;
- CI runs formatting, compilation, Clippy, tests, size, license, and platform
  checks;
- all commits use Conventional Commits.

## 14. Release and migration policy

- SQLite schema migrations are forward-only and tested from every public alpha.
- Raw evidence format changes require migration or dual readers.
- Plugin/WIT and daemon protocols are versioned before third-party release.
- Ghostty and RTK revisions remain pinned with licenses and checksums.
- macOS releases require Developer ID signing, notarization, and update
  signature verification.
- Linux releases provide signed packages and checksums.
- telemetry is off by default; local savings metrics require no account.
- public beta waits for name/trademark, package, domain, and repository checks.

## 15. Product metrics

Measure:

- raw versus returned bytes and estimated tokens by command family;
- expansion rate after compaction;
- decisive-diagnostic preservation;
- command latency and peak memory;
- plan rejection, stale-plan, rollback, and verification rates;
- daemon reconnect success and lost-frame rate;
- documentation citation precision and stale-index rate;
- accessibility regressions and keyboard completion rate;
- plugin denials and capability prompts.

Avoid vanity metrics such as command count alone. A new command is valuable only
when it reduces context, improves safety, or removes repeated terminal work.

## 16. Final sequencing rule

The next major work is not more wrappers. The current 61-command surface is
enough to benchmark real repositories. The correct sequence is:

1. secure credentials and evidence keys;
2. stream and preserve raw output;
3. stabilize typed results and native compression;
4. add the persistent session daemon;
5. build semantic command blocks and the documentation agent;
6. add mutation, workflows, plugins, and SSH only on that foundation.

This keeps OpenTerm useful now while preventing a broad command catalog from
outgrowing its evidence, safety, and UX guarantees.
