# Changelog

All notable changes to this project are documented here. The format is based on
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- 🎉 **Part 2 · Real-World Engineering — planned arc complete (7 lessons)**,
  each comparing one concept across Rust, Go, and Python with tested modules:
  Repository Pattern & DI · Configuration & Secrets · Logging & Observability ·
  Caching · The Testing Pyramid · Clean & Hexagonal Architecture · CI/CD.
  18 new example modules (6 concepts × 3 languages; CI/CD uses the real
  pipeline as its case study), all fmt/lint clean with passing tests. Highlights:
  tested secret redaction, injected clocks (no sleeps), an executable
  architecture test, and per-layer pyramid test modules.

- **Part 2 · Real-World Engineering started** — a new lesson format that compares
  one engineering concept across all three languages with tested modules in each.
  - **Repository Pattern & Dependency Injection** (bilingual EN + PT): ports &
    adapters / hexagonal design, dependency injection, and testing without a
    database. The identical domain is implemented and tested in Rust
    (`part2-repository-rust`, 3 tests), Go (`part2-repository-go`), and Python
    (`part2-repository-py`, 4 tests) — shown side by side via synced tabs. A
    comparison table maps trait/interface/protocol and static/dynamic dispatch.
  - CI matrices extended to cover all three Part 2 modules.
  - Sidebar gains a "Part 2 · Real-World Engineering" group.

- **Python Foundations track is now 100% complete** — five bilingual (EN + PT)
  lessons, each with a tested module (`ruff` clean, `python3 -m unittest` green
  including doctests):
  - **The Data Model** (`py-datamodel`, 13 tests): dunder methods — `__len__`/
    `__getitem__` sequences, arithmetic dunders, `__repr__` vs `__str__`.
  - **Type Hints** (`py-typing`, 17 tests): annotations, `X | None` Optionals,
    generics, dataclasses, the mutable-default trap.
  - **Protocols & Duck Typing** (`py-protocols`, 8 tests): structural typing,
    `typing.Protocol`, `@runtime_checkable`, Protocol vs ABC.
  - **Iterators & Generators** (`py-iterators`, 14 tests): the iterator protocol,
    `yield`, lazy/infinite sequences, generator pipelines.
  - **Testing** (`py-testing`, 10 tests): `unittest`/`pytest`, fixtures,
    `assertRaises`, `subTest`, mocking with `unittest.mock`, doctests.
  - Every inline Python program was run with `python3` to confirm output. CI runs
    a `python-examples` matrix (ruff + unittest) over all five modules.
- 🎉 **Part 1 · Foundations is complete across all three languages** — 20 lessons
  (Rust 9 · Go 6 · Python 5), 40 bilingual pages, 20 tested example modules/crates.

- **Go Foundations track is now 100% complete** — six bilingual (EN + PT) lessons,
  each with a tested Go module (`gofmt` clean, `go vet` clean, `go test` green):
  - **Structs, Methods & Interfaces** (`go-interfaces`): receivers, implicit
    interface satisfaction, embedding, type switches, table-driven tests.
  - **Error Handling** (`go-errors`, 8 tests): errors as values, sentinel + custom
    errors, `%w` wrapping, `errors.Is`/`errors.As`.
  - **Slices, Maps & Strings** (`go-collections`): backing-array aliasing, the map
    comma-ok idiom, bytes vs runes, `strings.Builder`.
  - **Testing & Documentation** (`go-testing`): table-driven subtests, runnable
    `Example` functions with `// Output:`, benchmarks, coverage.
  - **Generics** (`go-generics`, 7 tests): type parameters, constraints & the `~`
    operator, `comparable`/`any`, generic types.
  - **Goroutines & Channels** (`go-concurrency`): goroutines, buffered/unbuffered
    channels, `select`, `sync.WaitGroup`/`Mutex`, tests pass under `-race`.
  - Every inline Go program was run with `go run` (concurrency ones under `-race`)
    to confirm output. CI runs a `go-examples` matrix over all six modules.
  - Sidebar Go group badge flipped to "6 · complete".
- Sidebar reorganised: Part 1 · Foundations now groups lessons by language
  (Rust, Go) as collapsible sub-sections.

- **Rust Foundations track is now 100% complete** — nine bilingual (EN + PT)
  lessons, each with a tested companion crate. This release adds five:
  - **Testing & Documentation** (`rust-testing`, 8 unit + 1 integration + 2
    doc-tests): the three test kinds, `#[should_panic]`, `Result`-returning tests.
  - **Generics & Lifetimes** (`rust-generics`, 7 tests): type parameters, bounds,
    monomorphization, lifetime annotations, structs holding references.
  - **Smart Pointers & Interior Mutability** (`rust-smart-pointers`, 7 tests):
    `Box`, `Rc`, `RefCell`, `Rc<RefCell<T>>`, and the cycle/`Weak` trap.
  - **Concurrency Basics** (`rust-concurrency`, 5 tests): threads, `move`
    closures, `mpsc` channels, `Arc<Mutex<T>>`, `thread::scope` — all
    deterministic.
  - **Unsafe & FFI** (`rust-unsafe`, 7 tests): the five `unsafe` abilities, a safe
    `split_at_mut` over an unsafe core, and calling C both ways.
  - Every inline program in all five lessons was compiled with `rustc` to confirm
    its output. CI runs fmt/clippy/test across all nine crates via a matrix.
- **Part 1 · Foundations → Rust → Collections & Iterators** — a complete,
  bilingual lesson covering `Vec`, `HashMap` (entry API) and `String`, lazy
  iterator adapters (`map`/`filter`/`fold`/`enumerate`/`zip`/`partition`), a
  zero-cost-abstraction benchmark (iterator chain ~4500 ns vs index loop
  ~4195 ns vs intermediate-collect ~6461 ns), three common mistakes with real
  errors, three exercises with solutions, a CSV-summariser mini-project,
  interview questions, and a challenge. Every inline program was compiled with
  `rustc` to confirm its output (including catching that `max_by_key` returns the
  last maximum on ties).
- `examples/rust-collections/` — dependency-free companion crate: word-count,
  top-words, iterator helpers, 7 passing tests, clean `clippy`, and an
  `iterbench` benchmark binary.
- **Part 1 · Foundations → Rust → Types & Traits** — a complete, bilingual lesson
  covering required vs default methods, static vs dynamic dispatch (with a
  flowchart and a measured benchmark: static ~461 ns vs dynamic ~1998 ns per
  iteration), `impl Trait`, derivable traits, object safety, and the orphan rule.
  Three exercises with full solutions, a plugin-style mini-project, interview
  questions, and a challenge. Every inline program is self-contained and was
  compiled with `rustc` to confirm its output.
- `examples/rust-traits/` — dependency-free companion crate: a `Shape` trait with
  three implementors, static/dynamic dispatch helpers, `impl Trait`, 9 passing
  tests, clean `clippy`, and a `dispatch` benchmark binary.

- **Part 1 · Foundations → Rust → Error Handling with `Result`, `Option` & `?`**
  — a complete, bilingual lesson: theory (the two prelude enums), a `?`-propagation
  flowchart, several **complete runnable programs** (each with its own `fn main`),
  fail-fast vs fail-soft parsing, the three most common mistakes with real
  compiler/panic output, three exercises with full solutions, a performance note
  on `Result` vs `panic!`, best-practices/anti-patterns, a mini-project, interview
  questions, a challenge, and references.
- `examples/rust-error-handling/` — dependency-free companion crate with a typed
  `ParseError` enum, `?`-based parser, fail-fast/fail-soft strategies, and
  `Option` helpers. 9 passing tests, clean `cargo fmt`/`clippy`, runnable binary.
- CI now tests every example crate via a build matrix.

### Changed

- Every displayed code sample is now **self-contained and runnable** (own `fn
  main` or a whole item from the tested crate) so readers can replicate without
  errors — verified by compiling each inline program with `rustc`.

## [0.1.0] — Foundation release

### Added

- Astro Starlight site framework with English + Português (i18n), offline
  full-text search, dark/light themes, SEO metadata, and responsive layout.
- Custom teaching components: `Quiz` (interactive), `Benchmark` (proportional
  bars), `Mermaid` (theme-aware client render), `Interview` (collapsible),
  `LessonMeta` (difficulty + reading time + language pills), `Progress`
  (localStorage lesson tracking), and an authorship `Footer`.
- Frontmatter schema extensions: `difficulty`, `estimatedMinutes`, `languages`.
- **Part 1 · Foundations → Rust → Ownership & the Borrow Checker** — a complete,
  bilingual lesson covering theory, a move diagram, tested examples, the three
  most common borrow-checker errors, three exercises with solutions, a measured
  benchmark, best-practices/anti-patterns, interview questions, a challenge, and
  references.
- `examples/rust-ownership/` — dependency-free companion crate: 7 passing tests
  (`cargo test`), clean `cargo fmt`/`cargo clippy`, and a `bench` binary that
  produces the numbers in the lesson.
- Project governance: README, MIT LICENSE, CONTRIBUTING, CODE_OF_CONDUCT,
  SECURITY, ROADMAP, and this CHANGELOG.
- CI workflow that builds the site and runs the Rust checks on every push.
