# Rustlings Intermediate

This is a community Rustlings course for the intermediate material in
`02_intermediate_rust_course.ipynb`.

Run it the same way as Rustlings:

```bash
cd rustlings-intermediate
rustlings
```

Useful commands:

```bash
rustlings run generics_stack
rustlings hint quiz1_generics_traits_lifetimes
rustlings check-all
rustlings dev check
```

## Course Map

| Order | Exercise | Topic |
| --- | --- | --- |
| 1 | `generics_stack` | Generic structs and methods |
| 2 | `traits_shapes` | Traits, trait bounds, and formatting |
| 3 | `lifetimes_text_processor` | Struct lifetimes and borrowed returns |
| 4 | `closures_pipeline` | `Fn` closures and iterator pipelines |
| 5 | `iterators_fibonacci` | Custom iterators and adaptors |
| 6 | `smart_pointers_tree` | `Rc<RefCell<T>>` tree ownership |
| 7 | `concurrency_word_count` | Threads, channels, and merging state |
| 8 | `cli_parser` | CLI parser design inspired by `clap` |
| 9 | `async_status_plan` | Async-shaped status collection |
| 10 | `testing_calculator` | Unit tests, docs, and `Result` |
| 11 | `quiz1_generics_traits_lifetimes` | Quiz: generics, traits, and lifetimes |
| 12 | `quiz2_iterators_pointers_threads` | Quiz: iterators, pointers, and threads |
| 13 | `quiz3_libraries_testing` | Quiz: libraries, async shape, and testing |

## How To Work

Each exercise is a short typing lecture. Read the comments, replace the
`todo!()` calls with real Rust, and rerun Rustlings for immediate feedback.
Reference solutions live in `solutions/` and are used by `rustlings dev check`.
