# LLM provider architecture

## Purpose

OpenTerm uses LLMs for optional guidance through commands such as `ai-ask`. Terminal operation, RTK compression, and deterministic commands must continue working without an LLM connection.

The initial provider set is:

- OpenAI through the Responses API;
- Claude through the Anthropic Messages API;
- Mistral through Chat Completions;
- Codex through the locally installed Codex CLI;
- Ollama through the local OpenAI-compatible Chat Completions API;
- LM Studio through the local OpenAI-compatible Chat Completions API.

## OpenAgents findings

The design was informed by the maintainer's earlier OpenAgents project.

Useful patterns retained:

- provider identity, authentication, and endpoint metadata are declarative;
- wire-protocol conversion stays provider-specific;
- callers consume one narrow normalized result;
- credentials are resolved separately from request formatting;
- streaming and cancellation belong to the transport;
- Codex rich-client integration uses `codex app-server` over JSON-RPC;
- the central agent/tool surface stays small while providers live at the edge.

OpenTerm does not copy OpenAgents' large conversation loop. `ai-ask` currently needs only a one-shot text result.

## Command boundary

Every public tool command starts with `ai-`:

```text
ai-ask
ai-doctor
ai-version
ai-grep
ai-read
ai-test
```

`openterm` remains the application launcher. Internal typed IDs such as `llm.ask` or `fs.read` are protocol names, not executables.

Packaging uses one multicall Rust binary with `ai-*` links. This keeps startup, configuration, policy, and provider behavior consistent without compiling a separate implementation for every command.

## Provider contract

The current contract intentionally returns only text:

```rust
Provider::ask(&AskRequest) -> Result<String, String>
```

The next transport contract should add:

- streamed text deltas;
- normalized usage;
- provider request/response IDs;
- finish reason;
- cancellation;
- structured provider errors;
- optional tool calls for capability-gated modes;
- raw event references for debugging.

Do not add those fields until the interactive client consumes them.

## Current transports

### OpenAI

- Endpoint: `POST {base}/v1/responses`.
- Base URL: `OPENAI_BASE_URL`, else `base_url` in `providers.json`, else
  `https://api.openai.com`.
- Authentication: bearer token from `OPENAI_API_KEY` or `providers.json`.
- Model: required explicitly by `ai-ask`, or the model saved by `ai-connect`.
- Stable `instructions` plus `prompt_cache_key` so gateways can reuse prefixes.
- Response parser: concatenates text content from response output messages.
- `usage` cached-token counts are reported on stderr when the gateway hits cache.

### Claude

- Endpoint: `POST https://api.anthropic.com/v1/messages`.
- Authentication: `x-api-key` from `ANTHROPIC_API_KEY`.
- Required API-version header is sent.
- Model: required explicitly by `ai-ask`.
- Response parser: concatenates text content blocks.

### Mistral

- Endpoint: `POST https://api.mistral.ai/v1/chat/completions`.
- Authentication: bearer token from `MISTRAL_API_KEY`.
- Model: required explicitly by `ai-ask`.
- Response parser: reads the first assistant message.

### Ollama and LM Studio

- Endpoint: `POST {host}/v1/chat/completions`.
- Authentication: none.
- Default hosts: Ollama `http://127.0.0.1:11434`, LM Studio `http://127.0.0.1:1234`.
- Override with `OLLAMA_HOST` or `LMSTUDIO_HOST` (scheme optional).
- Model: required; `ai-connect` lists only models the runtime currently exposes.
- Response parser: reads the first assistant message.

### Codex CLI

The first implementation uses `codex exec` because `ai-ask` is a one-shot command. It sets:

- `--ignore-user-config` to isolate command behavior from unrelated invalid settings;
- `--ephemeral` to avoid retaining a conversation rollout;
- `--sandbox read-only`;
- `--skip-git-repo-check` so general shell questions work anywhere;
- `--color never` for clean output.

The future terminal-integrated agent should use `codex app-server`, which provides authentication, threads, turns, approvals, and streamed events over JSON-RPC. Generate schemas from the installed Codex version rather than hand-maintaining them.

`ai-connect` checks the installed CLI through `codex login status` and persists
only OpenTerm behavioral settings. It does not read `~/.codex/auth.json`, call
private Codex backends, embed OAuth client constants, or store Codex tokens.

## Safety modes

`ai-ask` is advice-only:

- direct API providers receive no tools or filesystem context;
- Codex may inspect the working directory in a read-only sandbox;
- requests to organize or modify files return a proposed plan;
- generated commands are text, not automatically executed;
- secrets are never included in prompts automatically.

Future action commands must be separate:

- `ai-plan`: inspect and propose a hash-bound plan;
- `ai-apply`: apply an approved plan under capabilities;
- `ai-verify`: verify the resulting state.

Do not turn `ai-ask` into an implicit mutation path.

## Credential rules

- API keys are secrets; behavioral settings are not.
- Read keys only at request time.
- Direct-provider keys currently use environment variables or a private
  mode-0600 compatibility file. Move them to OS keychain-backed references
  before the next distributable alpha.
- Never store keys in command history, result records, prompts, or logs.
- Never pass keys on command lines.
- Bind credentials to their exact provider endpoint.
- Do not forward credentials across redirects to another origin.
- `ai-doctor` reports only configured/not configured.
- Codex owns its own authentication cache; OpenTerm should not parse or copy its tokens.

## Configuration direction

The future configuration belongs in `~/.config/openterm/config.toml`:

```toml
[llm]
default_provider = "codex"

[llm.providers.openai]
model = "<user-selected-model>"
# Optional OpenAI-compatible gateway origin (no trailing /v1)
# base_url = "https://example.com"

[llm.providers.claude]
model = "<user-selected-model>"

[llm.providers.mistral]
model = "<user-selected-model>"
```

Do not hard-code a fast-moving API model as a permanent product default. Setup should discover available models or ask the user to select one, then persist that selection.

## Verification

- unit-test each provider response parser;
- test missing credentials without network access;
- test unknown providers and missing models;
- use mock HTTP servers before adding live CI tests;
- keep opt-in live-provider smoke tests outside ordinary CI;
- test Codex absence, authentication failure, timeout, interruption, and malformed output;
- ensure every registered executable starts with `ai-`;
- run `make verify` before commit.

## Official references

- OpenAI models and Responses support: https://developers.openai.com/api/docs/models
- Codex app-server: https://developers.openai.com/codex/app-server
- Codex non-interactive mode: https://developers.openai.com/codex/noninteractive
- Claude API overview: https://platform.claude.com/docs/en/api/overview
- Claude authentication: https://platform.claude.com/docs/en/manage-claude/authentication
- Mistral API reference: https://docs.mistral.ai/api
- Mistral API keys: https://docs.mistral.ai/admin/security-access/api-keys
