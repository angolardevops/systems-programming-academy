# Contributing

Thank you for helping build the Academy. The bar is deliberately high: this is
meant to be publishable without further editorial work.

## Non-negotiables for any lesson

1. **Everything executes.** No pseudo-code, no placeholders, no `TODO`.
2. **Everything is tested.** Companion code lives in `examples/<lang>-<topic>/`
   and ships with passing tests. Reference the exact functions from the prose.
3. **Benchmarks are real.** If you state a performance number, include the binary
   that produced it and run it in `--release`.
4. **Bilingual.** Add both the English (`src/content/docs/…`) and Portuguese
   (`src/content/docs/pt/…`) versions of a lesson in the same PR.
5. **Every required section is present:** Theory · Visual explanation · Examples ·
   Common mistakes & debugging · Exercises + solutions · Performance · Best
   practices vs anti-patterns · Interview questions · Challenge · References.

## Local checks before you open a PR

```bash
# Site builds cleanly
npm install
npm run build

# Rust companion code (repeat per crate you touched)
cd examples/rust-ownership
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```

## Style

- Prefer the idiom of the language being taught; write in the spirit of its
  creator (van Rossum / Pike / Hoare).
- Keep code snippets small and explained. Long code belongs in `examples/`.
- Use the provided components (`Quiz`, `Benchmark`, `Mermaid`, `Interview`,
  `LessonMeta`, `Progress`) rather than hand-rolling markup.

## Commit & PR

- One topic per pull request.
- Describe what you added and paste the output of the test/bench commands.
