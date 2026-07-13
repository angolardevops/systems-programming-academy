# Roadmap

The Academy is built in vertical slices: a complete, tested lesson beats ten
stubs. Each lesson meets the full quality bar before the next begins.

## ✅ Shipped — Foundation release

- Site framework: Astro Starlight, i18n (EN/PT), offline search, dark/light,
  SEO, responsive.
- Teaching components: `Quiz`, `Benchmark`, `Mermaid`, `Interview`,
  `LessonMeta`, `Progress`, authorship `Footer`.
- **Part 1 → Rust → Ownership & the Borrow Checker** (EN + PT), with a tested
  companion crate and a real benchmark.
- **Part 1 → Rust → Error Handling with `Result`, `Option` & `?`** (EN + PT),
  with a 9-test companion crate and fully runnable inline examples.
- **Part 1 → Rust → Types & Traits** (EN + PT), with a 9-test companion crate and
  a static-vs-dynamic-dispatch benchmark.
- **Part 1 → Rust → Collections & Iterators** (EN + PT), with a 7-test companion
  crate and a zero-cost-abstraction benchmark.
- **Part 1 → Rust — track 100% complete** (EN + PT), 9 lessons / 9 tested crates
  / 69 passing tests:
  Ownership · Error Handling · Types & Traits · Collections & Iterators ·
  Testing & Documentation · Generics & Lifetimes · Smart Pointers & Interior
  Mutability · Concurrency Basics · Unsafe & FFI.

## ✅ Part 1 · Foundations — COMPLETE (all three languages, 20 lessons)

- ~~Rust: complete~~ ✅ (9 lessons)
- ~~Go: complete~~ ✅ (6 lessons)
- ~~Python: complete~~ ✅ (5 lessons): The Data Model · Type Hints ·
  Protocols & Duck Typing · Iterators & Generators · Testing.

Every lesson is bilingual (EN + PT), ships a tested companion module/crate under
`examples/`, and had every inline program verified by running it. Next up is
Part 2.

## ✅ Part 2 · Real-World Engineering — planned arc COMPLETE (7 lessons)

Each lesson compares the same concept across Rust, Go, and Python with tested
modules in all three.

- ~~Repository Pattern & Dependency Injection~~ ✅ (ports & adapters; tested
  modules in all three languages)
- ~~Configuration & Secrets~~ ✅ (typed fail-fast loader, injected env, tested
  secret redaction; modules in all three languages)
- ~~Logging & Observability~~ ✅ (structured JSON lines, levels, injected sink,
  request_id context, tested log output; slog/logging/hand-rolled)
- ~~Caching~~ ✅ (TTL cache, injected clock — no sleeps in tests, hit/miss
  counters, cache-aside proven by a call-counting backend)
- ~~The Testing Pyramid~~ ✅ (one signup feature tested at unit/integration/e2e
  levels in all three languages; fakes vs mocks; the ice-cream cone)
- ~~Clean & Hexagonal Architecture~~ ✅ (domain/app/adapters/composition layers,
  the Dependency Rule, and an executable architecture test in Python)
- ~~CI/CD~~ ✅ (the Academy's own 4-job, 38-module pipeline dissected; gates,
  matrices, caching, deploy with needs: and secret stores)

Possible Part 2 extensions (not yet scheduled): Code Review · Refactoring &
Technical Debt · Security · Production Readiness.

## ✅ Part 3 · DevOps Automation — core COMPLETE (4 projects)

Real ops tools built in all three languages, tested to byte-identical output,
then benchmarked — with the why-Python/why-Go/why-Rust decision made on numbers.
The four projects cover the four ops-tool archetypes: CLI (throughput/startup),
resident agent (memory), generator (maintainability), concurrent prober
(fan-out/timeouts).

- ~~Project: Log Analyzer CLI~~ ✅ (500k-line benchmark: Rust 98ms · Go 131ms ·
  Python 736ms; startup 2/3/26ms; binaries 555K/2.4M/interpreter)
- ~~Project: Prometheus-Style Exporter~~ ✅ (registry + text exposition format;
  live-diffed byte-identical /metrics; hand-rolled TCP HTTP in Rust; resident
  memory 2.0/7.6/21.3 MB — the agent calculus)
- ~~Project: Config Generator~~ ✅ (spec -> nginx + systemd artifacts; golden-file
  tests; byte-identical CLI outputs; the maintainability-driven why-X)
- ~~Project: Health-Check Agent~~ ✅ (parallel TCP probing with timeouts —
  Part 1 concurrency doing ops work: 5 timeouts in 504ms not 2.5s; real-socket
  tests via ephemeral bind-then-close ports; exit-code contract)

Possible Part 3 extensions (not scheduled): Backup automation · YAML parser ·
Infrastructure auditing.

## 🔭 Later parts
- **Part 4 — Concurrency & Parallelism:** threads → async → channels →
  **mini-NGINX** in all three languages, benchmarked.
- **Part 5 — Building Frameworks:** rebuild an ORM, a web framework, and a
  green-thread runtime from scratch.
- **Capstone — Online Ticket Platform:** a production microservice architecture,
  no frameworks, each language where it fits best.

## Definition of done (per lesson)

Executable ✓ · Tested ✓ · Benchmarked (where relevant) ✓ · Bilingual ✓ ·
All required sections ✓ · `cargo/go/pytest` green in CI ✓ · No TODOs ✓.
