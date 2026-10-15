# Ownership and Borrowing: Why Rust Is Memory-Safe

A comparison with C++ call-by-value / call-by-reference, and what the Rust
compiler actually checks.

> Every code example here was compiled with `rustc 1.96.0` (edition 2021).
> Lines marked ❌ fail to compile with the error code shown.

---

## 1. The starting question

In C++, a call site does not tell you how an argument is passed:

```cpp
void a(std::string s);        // copy
void b(const std::string& s); // read-only reference
void c(std::string& s);       // mutable reference

std::string x = "hi";
a(x); b(x); c(x);             // all three calls look identical
```

`c(x)` can change `x`, but nothing at the call site says so. The compiler
accepts it, and the caller may be surprised later.

**Is Rust "call by reference with strict compile-time checks"?** That is
partly right. Rust does make passing explicit, but it adds two more ideas:

1. The default is **move**, not reference and not copy.
2. The compiler tracks **who owns a value, who is borrowing it, and for how
   long**. Most of the safety comes from this tracking, not just from the
   explicit syntax.

---

## 2. Three ways to pass a value, all visible at the call site

```rust
fn take(s: String)      { println!("took {s}"); }   // ownership moves in
fn look(s: &String)     { println!("len {}", s.len()); } // shared borrow (read-only)
fn edit(s: &mut String) { s.push_str(" world"); }   // exclusive borrow (may modify)

fn main() {
    let mut s = String::from("hello");
    look(&s);        // caller writes &
    edit(&mut s);    // caller writes &mut, and s must be declared `mut`
    take(s);         // move: s is no longer usable here
}
```

Both sides have to state the intent:

| Intent | Function signature | Call site | Variable declaration |
|---|---|---|---|
| Give it away | `T` | `f(x)` | `let x` |
| Let it read | `&T` | `f(&x)` | `let x` |
| Let it modify | `&mut T` | `f(&mut x)` | `let mut x` |

If they don't match, the code doesn't compile:

```rust
fn edit(s: &mut String) { s.push('!'); }

let s = String::from("hi");
edit(&mut s);   // ❌ E0596: cannot borrow `s` as mutable, as it is not declared as mutable

let mut s = String::from("hi");
edit(s);        // ❌ E0308: mismatched types (expected `&mut String`, found `String`)
```

You can't modify a value by accident. Both the caller and the callee have to
agree to it.

---

## 3. Move is the default

```rust
fn take(s: String) {}

let s = String::from("hi");
take(s);
println!("{s}");   // ❌ E0382: borrow of moved value: `s`
```

When a value moves, ownership goes with it. The old variable is no longer
valid, and the compiler rejects any later use of it.

C++ `std::move` also exists, but there the moved-from object is still usable
("valid but unspecified state"). Using it compiles and can produce surprising
results. Rust makes that a compile error.

### Exception: `Copy` types

Small, plain-data types such as integers, floats, `bool`, `char`, and tuples
or arrays made of them implement `Copy`. They are copied instead of moved:

```rust
let n = 5;
let m = n;          // copy, not move
println!("{n} {m}"); // fine
```

### Explicit copy: `.clone()`

If you want a deep copy of a heap type, you have to write it out:

```rust
let a = String::from("abc");
let c = a.clone();   // costly copies are always visible
println!("{a} {c}");
```

---

## 4. The borrow rules (the core of it)

### Rule 1: Aliasing XOR mutability

At any moment, a value can have **either**

- any number of shared references (`&T`), **or**
- exactly one mutable reference (`&mut T`),

but never both.

```rust
let mut s = String::from("hi");
let a = &mut s;
let b = &mut s;     // ❌ E0499: cannot borrow `s` as mutable more than once at a time
a.push('1');
b.push('2');
```

Here is the classic bug this rule prevents:

```rust
let mut v = vec![1, 2, 3];
let r = &v[0];
v.push(4);          // ❌ E0502: cannot borrow `v` as mutable because it is also borrowed as immutable
println!("{r}");
```

The same code in C++ compiles:

```cpp
std::vector<int> v = {1, 2, 3};
int& r = v[0];
v.push_back(4);     // may reallocate → r now points to freed memory
std::cout << r;     // undefined behavior
```

This is **iterator invalidation**, a common source of use-after-free bugs in
C++. In Rust, the shared borrow `r` "locks" `v` against mutation until `r` is
no longer used.

A borrow lasts only until its **last use**, not until the end of the scope
(this is called "non-lexical lifetimes"). So this is fine:

```rust
let mut v = vec![1, 2, 3];
{ let r = &v[0]; println!("{r}"); }  // last use of r
v.push(5);                            // OK: no borrow is active anymore
```

If you only need the value, copy it out and the borrow ends right away:

```rust
let first = v[0];   // i32 is Copy, so no borrow is held
v.push(4);          // OK
```

### Rule 2: A reference can't outlive its owner (lifetimes)

```rust
let r;
{
    let s = String::from("x");
    r = &s;
}                   // s is dropped here
println!("{r}");    // ❌ E0597: `s` does not live long enough
```

In C++, returning a reference to a local or keeping a pointer to a destroyed
object compiles and becomes a **dangling reference**. In Rust it is a
compile error.

When a function returns a reference, the signature says which input it
borrows from:

```rust
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}
```

`'a` means: "the result is valid only as long as both inputs are." Callers
are then checked against that.

### Rule 3: One owner, deterministic cleanup

Each value has exactly one owner. When the owner goes out of scope, the value
is dropped (its memory is freed, files are closed, locks are released). This
is like C++ RAII, but the compiler also guarantees:

- **No double free.** Only one owner exists, so only one drop happens.
- **No use after free.** Rules 1 and 2 make sure no reference outlives the
  drop.
- **No forgotten free.** Drop is automatic. (Leaks are still possible on
  purpose, for example with `Rc` cycles or `mem::forget`.)

---

## 5. Thread safety comes from the same rules

A data race needs two threads accessing the same memory, with at least one of
them writing. That is "shared + mutable", which Rule 1 already forbids.

Rust extends this across threads with two marker traits:

- `Send`: the value can be moved to another thread.
- `Sync`: the value can be shared (`&T`) between threads.

For example, `Rc<T>` (non-atomic reference count) is neither, so sending it to
another thread is a compile error. You have to use `Arc<T>`, and add a
`Mutex<T>` if you need mutation. **Data races are compile errors, not runtime
bugs.**

---

## 6. C++ and Rust side by side

| Situation | C++ | Rust |
|---|---|---|
| Mutation visible at the call site | No, `f(x)` may modify `x` | Yes, `f(&mut x)` is required |
| Read-only reference | `const T&` (can be cast away with `const_cast`) | `&T` (enforced) |
| Default argument passing | Copy | Move (copy only for `Copy` types) |
| Using a moved-from value | Compiles (unspecified state) | ❌ E0382 |
| Mutating while a reference exists | Compiles, may dangle | ❌ E0502 |
| Two mutable references at once | Compiles | ❌ E0499 |
| Reference outlives its owner | Compiles, undefined behavior | ❌ E0597 |
| Double free / use after free | Possible | Impossible in safe Rust |
| Data race between threads | Compiles | ❌ (`Send` / `Sync`) |

---

## 7. When the rules are too strict

The borrow checker is conservative. It rejects some programs that would be
safe. These tools let you work around that **without giving up safety**, by
moving the check to runtime or making sharing explicit:

| Tool | What it does |
|---|---|
| `.clone()` | Make an independent copy and own it |
| `Rc<T>` / `Arc<T>` | Shared ownership with a reference count (single-threaded / thread-safe) |
| `RefCell<T>` | Borrow rules checked at **runtime** (panics on violation) |
| `Mutex<T>` / `RwLock<T>` | Borrow rules enforced by locking across threads |
| `Cell<T>` | Mutate `Copy` values through a shared reference |

And as a last resort, there is `unsafe`:

```rust
unsafe { /* raw pointers, FFI, ... */ }
```

Inside `unsafe`, **you** are responsible for upholding the rules. The idea
is to keep these blocks small, so a memory bug can only be in the places
marked `unsafe`.

---

## 8. Summary

- Rust does **not** default to "call by reference". It defaults to **move**,
  and references are explicit: `&` for reading, `&mut` for modifying.
- Intent has to be stated **on both sides**: the function signature and the
  call site (plus `let mut` on the variable).
- What actually makes Rust safe is that the compiler tracks **ownership**
  (who frees it), **borrows** (who is using it, read or write), and
  **lifetimes** (for how long).
- **Aliasing XOR mutability** rules out iterator invalidation,
  use-after-free and data races at compile time.
- Bugs that C++ prevents only by convention or discipline become **compile
  errors** in Rust, except inside `unsafe`, which is explicitly marked.

## Error codes in this document

| Code | Meaning | Explain locally |
|---|---|---|
| E0382 | Use of a moved value | `rustc --explain E0382` |
| E0499 | Two mutable borrows at once | `rustc --explain E0499` |
| E0502 | Mutable borrow while a shared borrow exists | `rustc --explain E0502` |
| E0596 | Mutable borrow of a non-`mut` variable | `rustc --explain E0596` |
| E0597 | Borrowed value does not live long enough | `rustc --explain E0597` |
| E0308 | Mismatched types (e.g. `String` where `&mut String` was expected) | `rustc --explain E0308` |
