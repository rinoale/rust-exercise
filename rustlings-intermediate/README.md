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
| 2 | `generics_bounds_inventory` | Trait bounds, `where`, `Clone`, `Display` |
| 3 | `traits_shapes` | Traits, trait bounds, and formatting |
| 4 | `traits_dispatch_notifications` | Default methods, static dispatch, `dyn Trait` |
| 5 | `lifetimes_text_processor` | Struct lifetimes and borrowed returns |
| 6 | `lifetimes_excerpt_picker` | Multiple lifetime relationships and borrowed slices |
| 7 | `closures_pipeline` | `Fn` closures and iterator pipelines |
| 8 | `closures_fn_traits` | `Fn`, `FnMut`, `FnOnce`, returned closures |
| 9 | `iterators_fibonacci` | Custom iterators and adaptors |
| 10 | `iterators_result_pipeline` | `filter_map`, `flat_map`, `try_fold`, `collect` |
| 11 | `smart_pointers_tree` | `Rc<RefCell<T>>` tree ownership |
| 12 | `smart_pointers_cow` | `Cow<'a, str>` clone-on-write |
| 13 | `concurrency_word_count` | Threads, channels, and merging state |
| 14 | `concurrency_shared_counter` | `Arc<Mutex<T>>` shared state |
| 15 | `cli_parser` | CLI parser design inspired by `clap` |
| 16 | `cli_fromstr_config` | `FromStr`, typed config, parse errors |
| 17 | `async_status_plan` | Async-shaped status collection |
| 18 | `testing_calculator` | Unit tests, docs, and `Result` |
| 19 | `quiz1_generics_traits_lifetimes` | Quiz: generics, traits, and lifetimes |
| 20 | `quiz2_iterators_pointers_threads` | Quiz: iterators, pointers, and threads |
| 21 | `quiz3_libraries_testing` | Quiz: libraries, async shape, and testing |

## How To Work

Each exercise is a short typing lecture. Read the comments, replace the
`todo!()` calls with real Rust, and rerun Rustlings for immediate feedback.
Reference solutions live in `solutions/` and are used by `rustlings dev check`.
