# Rust for Ruby / Python Developers

What actually needs studying when you come from a garbage-collected,
dynamically typed language, and what you can skip for now.

> Every Rust snippet here was compiled with `rustc 1.96.0` (edition 2021).
> Lines marked ❌ fail to compile with the error code shown.
> See also: [ownership_and_borrowing.md](ownership_and_borrowing.md)

---

## What carries over, and what doesn't

**Carries over (learn in a day):** `if`/`else`, loops, functions, string
formatting, `map`/`filter` style collection code, modules, a package manager,
unit tests.

**Doesn't carry over:** everything Ruby and Python handle *for* you at
runtime: memory management, `nil` checks, type checks, exceptions. In Rust
these become decisions you make in code, and the compiler checks them.

**Threads are not "the same everywhere."** CRuby (MRI) and CPython have a GIL
(global interpreter lock), so your threads don't run Ruby or Python code in
parallel and data races are rare. Rust threads truly run in parallel, and the
compiler is what prevents data races (see §8).

---

## Study order at a glance

| # | Topic | Why it's new for you | Priority |
|---|---|---|---|
| 1 | Ownership / borrowing / lifetimes | No GC: you decide who owns and who borrows | ★★★ must |
| 2 | Types and data in memory | `String` vs `&str`, number types, stack vs heap | ★★★ must |
| 3 | Enums + `match` | Variants carry data; `match` must cover every case | ★★★ must |
| 4 | `Option` / `Result` / `?` | No `nil`, no exceptions | ★★★ must |
| 5 | Traits and generics | No classes, no inheritance, no duck typing | ★★★ must |
| 6 | Iterators and closures | Familiar, but lazy and ownership-aware | ★★ soon |
| 7 | Smart pointers | What to use when the borrow checker says no | ★★ soon |
| 8 | Threads (`Send`/`Sync`) | Real parallelism, checked at compile time | ★ when needed |
| 9 | Async | Not like `asyncio`; needs a runtime | ★ when needed |

---

## 1. Ownership and borrowing

In Ruby and Python, every variable is a reference to a heap object, sharing is
free, and the GC cleans up. In Rust, **every value has exactly one owner**, and
everyone else borrows it, either read-only (`&`) or exclusively for writing
(`&mut`). The full explanation is in
[ownership_and_borrowing.md](ownership_and_borrowing.md).

### Ownership is a design change, not just syntax

Code like this is normal in Ruby and Python:

```ruby
class Node
  attr_accessor :parent, :children   # child -> parent AND parent -> child
end
```

Code shaped like this is what "fighting the borrow checker" usually means: a
web of objects that all point at each other. Rust code tends toward
**tree-shaped ownership**: one owner holds the data, and everything else
refers to it by **ID or index**:

```rust
struct Node { name: String, parent: Option<usize>, children: Vec<usize> }
struct Tree { nodes: Vec<Node> }   // the Tree owns every Node

impl Tree {
    fn add(&mut self, name: &str, parent: Option<usize>) -> usize {
        let id = self.nodes.len();
        self.nodes.push(Node { name: name.to_string(), parent, children: vec![] });
        if let Some(p) = parent { self.nodes[p].children.push(id); }
        id
    }
}
```

The same pattern shows up in games (entity IDs), graphs, ASTs and caches.

### Variables are immutable by default

```rust
let x = 5;
x = 6;          // ❌ E0384: cannot assign twice to immutable variable `x`

let mut y = 5;
y = 6;          // OK
```

---

## 2. Types and data in memory

### `String` vs `&str` (the week-one stumbling block)

| Type | What it is | Ruby/Python analogy |
|---|---|---|
| `String` | Owned, growable, heap-allocated text | a `str` you own |
| `&str` | Borrowed view into some text | a read-only slice of it |

Rule of thumb: **take `&str` as a parameter, return `String`.** A `&str`
parameter accepts both kinds:

```rust
fn greet(name: &str) -> String { format!("Hello, {name}!") }

let owned = String::from("Kim");
greet(&owned);  // &String converts to &str automatically
greet("Lee");   // string literals are already &str

let text = String::from("hello world");
let word: &str = &text[0..5];   // a view, no copy
```

The same split exists for lists: `Vec<T>` (owned) vs `&[T]` (borrowed slice).

### Number types are explicit and never mix implicitly

Python has `int` and `float`. Ruby has `Integer` and `Float`. Rust has
`i8`…`i128`, `u8`…`u128`, `isize`/`usize` (sizes and indexes), `f32` and `f64`.

```rust
let a: i32 = 5;
let b: i64 = 10;
let c = a + b;                       // ❌ E0308: mismatched types

let b2: i64 = a as i64;              // explicit conversion
let small: Result<u8, _> = 300i32.try_into();  // Err: 300 doesn't fit in a u8
```

### Type inference

You write far fewer types than you'd expect. `let v = vec![1, 2, 3];` needs
no annotation. Every type is still fixed at compile time, though, and a
variable can't change type later.

---

## 3. Enums and pattern matching

Rust enums are not Ruby symbols or Python `Enum`s. **Each variant can carry
its own data**:

```rust
enum Shape {
    Circle { r: f64 },
    Rect { w: f64, h: f64 },
}

fn area(s: &Shape) -> f64 {
    match s {
        Shape::Circle { r } => 3.14159 * r * r,
        Shape::Rect { w, h } => w * h,
    }
}
```

`match` must cover **every** variant. If you add `Triangle` later, every
`match` that doesn't handle it stops compiling:

```
❌ E0004: non-exhaustive patterns: `&Shape::Triangle { .. }` not covered
```

Compare a Ruby `case` or Python `if isinstance(...)` chain, where a forgotten
branch silently returns `nil` or `None`. Many Ruby/Python patterns become
an enum in Rust: state machines, message types, AST nodes, "either this or that"
return values.

---

## 4. `Option` and `Result` instead of `nil` and exceptions

### No null: `Option<T>`

A value that might be missing has type `Option<T>`, either `Some(value)` or
`None`. You can't use it without handling the `None` case, so
`NoMethodError: undefined method for nil` and
`AttributeError: 'NoneType' object has no attribute` can't happen.

```rust
let mut ages = HashMap::new();
ages.insert("kim", 30);

match ages.get("lee") {                      // returns Option<&i32>
    Some(age) => println!("{age}"),
    None => println!("no lee"),
}

if let Some(age) = ages.get("kim") { println!("kim is {age}"); }

let age = ages.get("kim").copied().unwrap_or(0);   // like Ruby's `|| 0` / Python's `or 0`
```

### No exceptions: `Result<T, E>` and `?`

Functions that can fail return `Result<T, E>`, either `Ok(value)` or
`Err(error)`. The `?` operator means "if this is an error, return it to my
caller":

```rust
#[derive(Debug)]
struct Config { port: u16 }

fn parse_port(s: &str) -> Result<Config, std::num::ParseIntError> {
    let port: u16 = s.trim().parse()?;   // on failure, return the Err right away
    Ok(Config { port })
}

parse_port("8080");  // Ok(Config { port: 8080 })
parse_port("abc");   // Err(ParseIntError { kind: InvalidDigit })
```

| Ruby / Python | Rust |
|---|---|
| `raise` / `raise` | `return Err(...)` |
| exception propagates up automatically | `?` (explicit, visible at each call) |
| `rescue` / `except` | `match result { Ok(..) => .., Err(e) => .. }` |
| unexpected crash | `panic!`, only for bugs, not for expected failures |

`.unwrap()` means "panic if this is `None`/`Err`." It's fine in prototypes and
tests; in real code, replace it with `?` or proper handling.

---

## 5. Traits instead of classes and inheritance

| Ruby / Python | Rust |
|---|---|
| `class` with fields and methods | `struct` (data) + `impl` block (methods) |
| inheritance | **none**; use traits and composition |
| mixin / ABC / protocol | `trait` |
| duck typing | explicit trait bound |
| `to_s` / `__str__` | `impl Display` |
| `inspect` / `__repr__` | `#[derive(Debug)]` |
| `==` / `__eq__` | `#[derive(PartialEq)]` |
| `dup` / `copy.copy` | `#[derive(Clone)]`, then `.clone()` |

```rust
trait Speak { fn speak(&self) -> String; }

struct Dog;
struct Cat { name: String }

impl Speak for Dog { fn speak(&self) -> String { "Woof".to_string() } }
impl Speak for Cat { fn speak(&self) -> String { format!("{} says meow", self.name) } }

#[derive(Debug, Clone, PartialEq)]   // free implementations of common traits
struct User { name: String, age: u32 }
```

### Duck typing becomes a trait bound

```rust
fn show<T: Display>(x: T) { println!("<{x}>"); }   // "anything printable"
show(42);
show("hi");
```

### Static vs dynamic dispatch, a choice Ruby and Python never ask for

```rust
// Static dispatch: the compiler generates a separate copy per concrete type. Fast.
fn announce(x: &impl Speak) { println!("{}", x.speak()); }

// Dynamic dispatch: one list holding different types, resolved at runtime (like Ruby/Python).
let animals: Vec<Box<dyn Speak>> = vec![Box::new(Dog), Box::new(Cat { name: "Nabi".into() })];
for a in &animals { println!("{}", a.speak()); }
```

Default to generics (`impl Trait`). Use `dyn Trait` when you need a mixed
collection.

---

## 6. Iterators and closures

This will feel the most familiar:

```rust
let nums = vec![1, 2, 3, 4, 5, 6];

// Ruby:   nums.select(&:even?).map { _1 ** 2 }.sum
// Python: sum(n * n for n in nums if n % 2 == 0)
let total: i32 = nums.iter().filter(|n| *n % 2 == 0).map(|n| n * n).sum();  // 56

let names: Vec<String> = vec!["a", "b"].into_iter().map(|s| s.to_uppercase()).collect();
```

### What's different

**Iterators are lazy.** Nothing runs until something consumes them
(`collect`, `sum`, `for`, …):

```rust
let lazy = nums.iter().map(|n| { println!("visiting {n}"); n * 2 });
println!("nothing printed yet");
let first_two: Vec<i32> = lazy.take(2).collect();   // only now visits 1 and 2
```

**Three ways to iterate, matching the three ways to pass a value:**

| Method | Yields | Collection afterwards |
|---|---|---|
| `.iter()` | `&T` | unchanged, still usable |
| `.iter_mut()` | `&mut T` | modified in place |
| `.into_iter()` | `T` | consumed (moved) |

**Closures capture with ownership rules.** A closure that changes captured
state must itself be `mut`:

```rust
let mut count = 0;
let mut inc = || count += 1;   // FnMut: mutably borrows `count`
inc(); inc();
println!("{count}");           // 2
```

`move || ...` makes the closure take ownership of what it captures. It's
required when the closure outlives the current scope, such as when it's
passed to a thread.

---

## 7. Smart pointers: when the borrow checker says no

Learn **when** to use each, not every detail:

| You want… | Use |
|---|---|
| a value on the heap (recursive types, trait objects) | `Box<T>` |
| several owners, one thread | `Rc<T>` |
| to mutate through a shared reference, one thread | `RefCell<T>` |
| shared + mutable, one thread (closest to Ruby/Python objects) | `Rc<RefCell<T>>` |
| several owners across threads | `Arc<T>` |
| shared + mutable across threads | `Arc<Mutex<T>>` |

```rust
let shared = Rc::new(RefCell::new(vec![1]));
let other = Rc::clone(&shared);       // second owner, no deep copy
other.borrow_mut().push(2);
println!("{:?}", shared.borrow());    // [1, 2]
```

If `Rc<RefCell<…>>` ends up everywhere in your code, it's usually a sign to
revisit the ownership design (§1).

---

## 8. Threads: real parallelism, checked at compile time

```rust
let counter = Arc::new(Mutex::new(0));
let handles: Vec<_> = (0..4).map(|_| {
    let counter = Arc::clone(&counter);
    thread::spawn(move || { *counter.lock().unwrap() += 1; })
}).collect();
for h in handles { h.join().unwrap(); }
// counter == 4
```

The compiler rejects types that aren't safe to share across threads:

```rust
let r = Rc::new(5);
thread::spawn(move || println!("{r}"));
// ❌ E0277: `Rc<i32>` cannot be sent between threads safely
```

In Ruby or Python, the GIL makes many race conditions unlikely without making
them impossible. In Rust, a data race is a **compile error**.

---

## 9. Async (only when you need networking)

- Rust has no built-in async runtime. You choose one, almost always
  [`tokio`](https://tokio.rs).
- A future does **nothing** until it is `.await`ed (unlike a JS Promise, which
  starts immediately).
- `async fn` + `.await` look like Python's `asyncio`, but the error messages
  involve `Send`, `Pin` and lifetimes. Learn it after §1–§8 feel natural.

---

## Easy to pick up, because it maps from tools you know

| Ruby / Python | Rust |
|---|---|
| Bundler / pip, Poetry | `cargo` |
| `Gemfile` / `pyproject.toml` | `Cargo.toml` |
| gem / package | crate (published on crates.io) |
| RSpec, Minitest / pytest | `#[test]` + `cargo test` (built in) |
| RuboCop / ruff, black | `cargo clippy` / `cargo fmt` |
| `require` / `import` | `mod` + `use` |
| irb / python REPL | `evcxr` (as used in this repo's notebooks) |

**Macros:** at first you only *use* them: `println!`, `vec!`, `format!`,
`#[derive(...)]`. Writing your own can wait.

---

## Safe to skip at first

- Complex lifetime annotations (`'a` beyond simple cases)
- `unsafe`
- Writing macros (`macro_rules!`, procedural macros)
- `Pin`, and how async works internally
- Advanced traits: GATs (generic associated types), HRTBs (higher-ranked
  trait bounds)

You'll recognize when you need them.

---

## Mindset tips

1. **Use `.clone()` freely at first.** Ruby and Python effectively share and
   copy behind your back all the time. Get it working, then remove clones as
   your understanding of ownership improves.
2. **Read compiler errors closely.** They usually explain the problem and
   suggest a fix. `rustc --explain E0502` gives a full explanation of any
   error code.
3. **Model data with enums before reaching for traits.** If the set of cases
   is closed and known, an enum + `match` is simpler than a trait hierarchy.
4. **Run `cargo clippy` often.** It teaches idiomatic Rust as you go.

---

## Where to practice in this repo

| Topic (§) | Resource |
|---|---|
| 1–4: ownership, types, enums, `Option`/`Result` | `01_beginner_rust_course.ipynb`, `rustlings/` |
| 5–8: traits, generics, iterators, closures, smart pointers, threads | `02_intermediate_rust_course.ipynb`, `rustlings-intermediate/` |
| 9 + skipped topics: async, unsafe, macros, advanced traits | `03_advanced_rust_course.ipynb`, `rustlings-advanced/` |
| Ownership in depth | [ownership_and_borrowing.md](ownership_and_borrowing.md) |
