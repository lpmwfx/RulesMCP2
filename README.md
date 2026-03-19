# RulesMCP — Rust MCP Server for AI Coding Rules

A **high-performance, zero-dependency Rust port** of the Python `rules-mcp` server. Implements the Model Context Protocol (MCP) for AI coding standards lookup across 290+ rules in 20+ languages.

## Features

- **7 MCP Tools**: help, search_rules, get_rule, get_context, get_learning_path, list_rules, get_related
- **Weighted Search**: Token-based scoring across rule file, title, tags, concepts, keywords, axioms
- **Learning Paths**: Phased rule reading order (layer 1-6) for structured learning
- **Edge Graph**: Follow rule dependencies (requires, feeds, related, etc.)
- **Zero External Deps**: Only git2, tokio, serde, directories (for caching)
- **Automatic Cloning**: Rules repo clones to `~/.cache/rules-mcp/Rules/` on first run
- **JSON-RPC stdio**: Direct MCP protocol implementation — no extra binary needed

## Installation

```bash
cargo install --git https://github.com/lpmwfx/RulesMCP rules-mcp
```

## Usage

Run as an MCP stdio server:

```bash
rules-mcp
```

The server reads JSON-RPC requests from stdin and writes responses to stdout.

### Example: Initialize

```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}
```

Response:
```json
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{},"serverInfo":{"name":"rules-mcp","version":"0.1.0"}},"error":null}
```

### Example: List Tools

```json
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":null}
```

### Example: Search Rules

```json
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"search_rules","arguments":{"query":"error handling","limit":5}}}
```

## Architecture

```
src/
├── main.rs           App layer — bootstrap, logging
├── adapter.rs        Adapter layer — 7 MCP tool methods
├── core.rs           Core layer — Registry, search, learning_path
├── repo.rs           Pal layer — Git cloning, cache management
├── server.rs         Adapter layer — JSON-RPC stdio transport
└── shared.rs         Shared layer — Entry_x, Error_x types
```

**Topology**: `shared` ← `core` ← `adapter` ← `server`, `repo` → `core`

## Rules Index

**290 rules** across **20+ categories**:
- Global (architecture, files, tech-debt)
- Rust (modules, docs, errors, ownership, threading)
- Python, JavaScript, C++, Kotlin, C#, Slint, CSS
- Project files, UIUX, DevOps, IPC

## Performance

- **Startup**: ~2 seconds (clone Rules repo once, then cached)
- **Search**: <1ms for 290 rules with token-based weighted scoring
- **Memory**: ~5MB in-memory index (register.jsonl)

## Configuration

Rules repo is cloned to (platform-dependent):
- **Linux/macOS**: `~/.cache/rules-mcp/Rules/`
- **Windows**: `%LOCALAPPDATA%\rules-mcp\cache\Rules\`

## Development

```bash
# Build
cargo build --release

# Test (manual)
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | cargo run --release

# Run in debug mode (verbose logging)
RUST_LOG=rules_mcp=debug cargo run
```

## License

EUPL-1.2 © 2024 TwistedBrain

## Related

- **Rules**: https://github.com/lpmwfx/Rules (290 markdown rule files)
- **RulesTools**: https://github.com/lpmwfx/RulesTools (Scanner + documenter in Rust)
- **Original (Python)**: https://github.com/lpmwfx/RulesMCP (deprecated, use Rust version)


---

<!-- LARS:START -->
<a href="https://lpmathiasen.com">
  <img src="https://carousel.lpmathiasen.com/carousel.svg?slot=4" alt="Lars P. Mathiasen"/>
</a>
<!-- LARS:END -->
