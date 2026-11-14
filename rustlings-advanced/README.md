# Rustlings Advanced

This is a community Rustlings course for the advanced material in
`03_advanced_rust_course.ipynb`.

Run it like Rustlings:

```bash
cd rustlings-advanced
rustlings
```

Useful commands:

```bash
rustlings run associated_converters
rustlings hint quiz1_traits_lifetimes
rustlings check-all
rustlings dev check
```

## Course Map

| Order | Exercise | Topic |
| --- | --- | --- |
| 1 | `associated_converters` | Associated types and newtype pattern |
| 2 | `associated_gat_views` | Generic associated types and borrowed views |
| 3 | `lifetime_views` | Borrowed views into owned storage |
| 4 | `hrtb_line_transforms` | Higher-ranked trait bounds |
| 5 | `unsafe_fixed_buffer` | Safe API over unsafe `MaybeUninit` storage |
| 6 | `unsafe_split_at_mut` | Raw pointers behind a safe API |
| 7 | `macro_json` | Declarative macros with recursive rules |
| 8 | `macro_rules_collections` | Macro repetition and optional trailing commas |
| 9 | `async_scraper_plan` | Async-shaped scraper planning |
| 10 | `async_manual_future` | Manual `Future`, `Poll`, and `Pin` |
| 11 | `typestate_connection` | Typestate transitions |
| 12 | `middleware_chain` | Decorator-style middleware |
| 13 | `command_undo_redo` | Command pattern with undo and redo |
| 14 | `blog_api_routes` | REST route and error modeling |
| 15 | `task_manager_architecture` | CLI, storage, logging, and export architecture |
| 16 | `ffi_repr_packets` | `repr(C)`, layout, and byte parsing |
| 17 | `quiz1_traits_lifetimes` | Quiz: traits and lifetimes |
| 18 | `quiz2_unsafe_macros_async` | Quiz: unsafe, macros, and async |
| 19 | `quiz3_patterns_systems` | Quiz: patterns and systems tools |

## How To Work

Each exercise is an interactive typing lecture. Read the comments, replace the
`todo!()` calls, and rerun Rustlings. Reference solutions live in `solutions/`
and are checked by `rustlings dev check`.
