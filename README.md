# Rust Exercise

Rust learning course notebooks using the `evcxr_jupyter` kernel.

## Courses

| File | Level | Lectures |
|------|-------|----------|
| 01_beginner_rust_course.ipynb | Beginner | Syntax, variables, functions, control flow, ownership, structs, enums, collections, error handling, modules |
| 02_intermediate_rust_course.ipynb | Intermediate | Generics, traits, lifetimes, closures, iterators, smart pointers, concurrency, libraries (serde/clap/regex), testing |
| 03_advanced_rust_course.ipynb | Advanced | Advanced traits, advanced lifetimes, unsafe Rust, macros, async/await, design patterns (creational/structural/behavioral), web frameworks, systems tools |

## Notes

| File | Topic |
|------|-------|
| ownership_and_borrowing.md | Why Rust is memory-safe: move vs `&` vs `&mut`, borrow rules, lifetimes, compared with C++ references |
| rust_for_ruby_python_devs.md | What a Ruby/Python developer needs to study first, in order, with side-by-side comparisons |

## Rustlings Courses

| Directory | Level | Start |
|-----------|-------|-------|
| rustlings-intermediate | Intermediate | `cd rustlings-intermediate && rustlings` |
| rustlings-advanced | Advanced | `cd rustlings-advanced && rustlings` |

## Protocol Demos

Rust ports of the Ruby demos in `ruby_codes` — each is a standalone cargo
project with its own README:

| Directory | What it shows | Run |
|-----------|---------------|-----|
| grpc_demo | tonic client/server + a MITM tap that decodes HTTP/2, HPACK, gRPC framing and protobuf byte by byte | `cd grpc_demo && ./run_demo.sh` |
| stun_turn_demo | STUN (RFC 5389) / TURN (RFC 5766) wire format, hole punching, P2P chat + file transfer over UDP | `cd stun_turn_demo && cargo run --bin run_scripted_demo` |

## Setup

```bash
cargo install evcxr_jupyter && evcxr_jupyter --install
```
