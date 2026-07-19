# The Ultimate Systems Programming Academy

> From first principles to production — with **Python**, **Go**, and **Rust**.

An open educational resource that teaches **software engineering**, not just
syntax. Every lesson is executable, tested, benchmarked with real numbers, and
written bilingually (English / Português).

**📖 Read it live: <https://angolardevops.github.io/systems-programming-academy/>**
(deployed automatically from `main` via GitHub Actions)

Built with [Astro Starlight](https://starlight.astro.build/): fast static output,
full-text offline search, dark/light themes, SEO, and WCAG-friendly navigation.

## Project status

This is the foundation release. The site framework and the first complete,
fully-tested lesson are done; the remaining curriculum is built out to the same
bar over time.

| Area | Status |
| --- | --- |
| Site framework (nav, search, i18n, dark mode, teaching components) | ✅ Complete |
| Part 1 → **Rust foundations track** (9 lessons, EN + PT) | ✅ Complete & tested (69 tests) |
| Part 1 → **Go foundations track** (6 lessons, EN + PT) | ✅ Complete & tested (`go test -race`) |
| Part 1 → **Python foundations track** (5 lessons, EN + PT) | ✅ Complete & tested (unittest + doctests) |
| **Part 2 → Real-World Engineering** (7 lessons, EN + PT) | ✅ Planned arc complete & tested |
| **Part 3 → DevOps Automation** (4 projects × 3 languages, EN + PT) | ✅ Complete, tested & benchmarked |
| **Part 4 → Concurrency & Parallelism** (3 lessons + mini-NGINX, 3 languages) | ✅ Complete, tested & benchmarked |
| **Part 5 → Building Frameworks** (routing · query builder · DI container · validation · template engine · JSON serialization · test framework, 3 languages) | ✅ Complete, tested (7 lessons) |
| **Part 6 → Command-Line Tools & Dashboards** (performance dashboard · port scanner · ping · traceroute · eBPF profiler, 3 languages) | ✅ Complete, tested (5 projects) |
| **Capstone → Secure Guestbook** (composes the frameworks; SQLi + XSS defeated, 3 languages) | ✅ Complete & tested |
| **Capstone → Running App** (HTTP server + real SQLite; SQLi defeated against actual SQLite, 3 languages) | ✅ Complete & tested |

**Part 1 · Foundations is complete across all three languages — 20 lessons.**

The nine complete Rust lessons: Ownership · Error Handling · Types & Traits ·
Collections & Iterators · Testing & Documentation · Generics & Lifetimes · Smart
Pointers & Interior Mutability · Concurrency Basics · Unsafe & FFI.

The six complete Go lessons: Structs/Methods/Interfaces · Error Handling ·
Slices/Maps/Strings · Testing & Documentation · Generics · Goroutines & Channels.

The five complete Python lessons: The Data Model · Type Hints · Protocols & Duck
Typing · Iterators & Generators · Testing.

Each lesson ships a tested companion module/crate under `examples/`.

See [ROADMAP.md](./ROADMAP.md) for the full plan and [CHANGELOG.md](./CHANGELOG.md)
for what shipped.

## Quick start

```bash
# 1. Install site dependencies
npm install

# 2. Run the dev server (http://localhost:4321)
npm run dev

# 3. Build the static site (output in ./dist)
npm run build && npm run preview
```

### Run every example with one command

Every lesson ships real, compilable code under `examples/`. Verify the whole
curriculum — **107 suites, 0 failing** — in one go:

```bash
scripts/run-all.sh            # every Rust/Go/Python suite (tests only)
scripts/run-all.sh --full     # also the fmt + lint gates CI enforces
scripts/run-all.sh part6      # or filter by name / language
```

Or run a single lesson's companion code:

```bash
cd examples/rust-ownership
cargo test                     # 7 passing tests
cargo run --release --bin bench  # reproduce the benchmark table
```

### Zero-setup environment

Open the repo in a ready-made environment with Rust, Go, Python, `uv`, and Node
already installed — no local setup:

[![Open in GitHub Codespaces](https://github.com/codespaces/badge.svg)](https://codespaces.new/angolardevops/systems-programming-academy)

Or locally: **VS Code → Reopen in Container** (uses `.devcontainer/`), or
`nix develop` (uses `flake.nix`). Full guide:
[Run It Yourself](https://angolardevops.github.io/systems-programming-academy/toolchains/run-it-yourself/).

## Repository layout

```
academy/
├── astro.config.mjs        # Site config: i18n (EN/PT), sidebar, theming
├── src/
│   ├── components/         # Quiz, Benchmark, Mermaid, LessonMeta, Progress …
│   ├── content/docs/       # Lessons — root = English, pt/ = Português
│   └── styles/academy.css  # Custom theme layer on top of Starlight
└── examples/
    └── rust-ownership/     # Tested companion crate for the first lesson
```

## Contributing

Read [CONTRIBUTING.md](./CONTRIBUTING.md) and our
[Code of Conduct](./CODE_OF_CONDUCT.md). Every code sample must compile, be
formatted, be linted, and be tested — CI enforces this.

## License

Content and code are released under the [MIT License](./LICENSE).

---

**Author:** Walter Angolar

Educational philosophy inspired by Guido van Rossum (Python), Rob Pike (Go), and
Graydon Hoare (Rust).
