# Contributing

Thanks for your interest in SCULPT.

## Ground Rules
- Be respectful and constructive.
- Keep changes focused and easy to review.
- Prefer clear, minimal patches over big rewrites.

## Development Setup
```bash
cargo install --path .
```

## Testing
```bash
cargo test
```

## Coding Style
- Keep Rust code clear and explicit.
- Prefer small modules and single-purpose functions.
- Include tests for parser changes and anything affecting determinism.

## Commit Messages
Follow Conventional Commits where possible, e.g.
- `feat: add target provider`
- `fix: correct parser span`
- `docs: clarify syntax`

## Pull Requests
- Include a short summary and rationale.
- Mention any schema or syntax changes explicitly.
- Add or update tests when behavior changes.
