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
| 2 | `lifetime_views` | Borrowed views into owned storage |
| 3 | `unsafe_fixed_buffer` | Safe API over unsafe `MaybeUninit` storage |
| 4 | `macro_json` | Declarative macros with recursive rules |
| 5 | `async_scraper_plan` | Async-shaped scraper planning |
| 6 | `typestate_connection` | Typestate transitions |
| 7 | `middleware_chain` | Decorator-style middleware |
| 8 | `command_undo_redo` | Command pattern with undo and redo |
| 9 | `blog_api_routes` | REST route and error modeling |
| 10 | `task_manager_architecture` | CLI, storage, logging, and export architecture |
| 11 | `quiz1_traits_lifetimes` | Quiz: traits and lifetimes |
| 12 | `quiz2_unsafe_macros_async` | Quiz: unsafe, macros, and async |
| 13 | `quiz3_patterns_systems` | Quiz: patterns and systems tools |

## How To Work

Each exercise is an interactive typing lecture. Read the comments, replace the
`todo!()` calls, and rerun Rustlings. Reference solutions live in `solutions/`
and are checked by `rustlings dev check`.
