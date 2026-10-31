---
name: Code Check
slug: code-check
description: Static code checking for Ragasukma - Rust (cargo clippy) and SolidJS web (tsc, eslint, knip). Use after modifying code, before finishing a task.
tags: [lint, clippy, tsc, eslint, knip, quality]
---

# Code Check

Run the checks below after modifying code. Run only the section(s) relevant to the changed files:
Rust changes -> **Rust**, `web/` changes -> **SolidJS**. If both changed, run both.

Use ONLY the commands in this file. Do not invent other lint flags or tools.

## Rust

Run from the project root (`ragasukma/`):

```bash
cargo clippy --bin ragasukma -- \
  -W dead_code \
  -W unused \
  -W clippy::redundant_clone \
  -W clippy::await_holding_lock \
  -W clippy::mutex_atomic \
  -W clippy::rc_buffer \
  -W clippy::panic \
  -W clippy::unwrap_used
```

PowerShell (single line, no line continuation):

```powershell
cargo clippy --bin ragasukma -- -W dead_code -W unused -W clippy::redundant_clone -W clippy::await_holding_lock -W clippy::mutex_atomic -W clippy::rc_buffer -W clippy::panic -W clippy::unwrap_used
```

What the flags catch:

| Flag | Catches |
| --- | --- |
| `dead_code` | Unused functions, structs, fields, constants |
| `unused` | Unused imports, variables, mut, results |
| `clippy::redundant_clone` | `.clone()` on values that are never used again |
| `clippy::await_holding_lock` | A `Mutex`/`RwLock` guard held across an `.await` (deadlock risk) |
| `clippy::mutex_atomic` | `Mutex<bool>` / `Mutex<usize>` that should be an atomic |
| `clippy::rc_buffer` | `Rc<String>` / `Rc<Vec<T>>` (use `Rc<str>` / `Rc<[T]>`) |
| `clippy::panic` | Explicit `panic!` in non-test code |
| `clippy::unwrap_used` | `.unwrap()` on `Option`/`Result` (use `?`, `match`, `unwrap_or*`) |

## SolidJS (web)

Run from the `web/` directory:

```bash
cd web
npx tsc --noEmit
npx eslint .
npx knip
```

| Command | Catches |
| --- | --- |
| `npx tsc --noEmit` | TypeScript type errors (no output files written) |
| `npx eslint .` | Lint problems and code style issues |
| `npx knip` | Unused files, exports, types, and dependencies |

## Workflow

1. Run the relevant command(s) above.
2. Read the warnings/errors and fix every one that points at code you touched.
3. Fix the root cause. Do not silence a warning with `#[allow(...)]`, `// eslint-disable`, or `@ts-ignore` unless there is a documented reason.
   - `unwrap_used`: replace with `?`, `if let`, `match`, `unwrap_or`, or `unwrap_or_else`.
   - `panic`: return an `Err` instead of panicking.
   - `await_holding_lock`: copy what you need out of the guard and drop it before `.await`.
   - `dead_code` / `unused` / `knip`: delete the unused item instead of keeping it.
4. Re-run the same command until it is clean for the code you changed.
5. In the final report, state which commands were run and their result. Warnings that were already there and are outside your change may be listed, but do not fix them unless asked.
