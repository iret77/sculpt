# SCULPT

SCULPT is a research compiler for a converging, AI-first programming language.
You write structured, non-deterministic code; the compiler turns it into a compact IR
and asks an LLM to produce target IR, which is then built deterministically by a target provider.

This repo contains the MVP compiler in Rust and a small set of example programs.

## Why SCULPT
- **Converging programming**: fewer lines mean a larger solution space; more lines constrain it.
- **AI-first**: the compiler, not the developer, handles prompting and target translation.
- **Provider-based**: LLM providers + target providers are pluggable.

## Status
- MVP research compiler (CLI) with LLM-backed build pipeline.
- Built-in targets: `cli`, `web`, `cpp`.

## Install
```bash
cargo install --path .
```

## Quick Start
Write or use an example file, then build:
```bash
sculpt examples
sculpt build examples/hello_world.sculpt --target cli --provider openai
```

Run the output:
```bash
sculpt run examples/hello_world.sculpt --target cli
```

## Commands
- `sculpt examples` — write example programs into `examples/`
- `sculpt build <file.sculpt> --target <cli|web|cpp> --provider <openai|stub>`
- `sculpt freeze <file.sculpt> --target <...> --provider <...>`
- `sculpt replay <file.sculpt> --target <...>`
- `sculpt run <file.sculpt> --target <...>`
- `sculpt target list`
- `sculpt target describe --target <name>`
- `sculpt auth check --provider openai [--verify]`

## OpenAI Authentication
Set your API key as an environment variable:
```bash
export OPENAI_API_KEY=... 
```

Optional config file (overridden by env var):
```json
{
  "provider": "openai",
  "openai": {
    "api_key": "...",
    "model": "gpt-4.1-mini"
  }
}
```

## Examples
- `examples/hello_world.sculpt` — deterministic hello world
- `examples/snake_high_nd.sculpt` — high ND, large solution space
- `examples/snake_low_nd.sculpt` — low ND, tight solution space
- `examples/invoice_review.sculpt` — business/web flavored example

## Syntax
See `SCULPT_Syntax_Manifest.md` for the current MVP syntax.

## License
MIT
