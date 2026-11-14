# Rust: Stack, Heap, And Memory

The **stack** and **heap** are two places where a program stores data in memory.

In Rust, the important question is often not only "is this on the stack or heap?"

The more important questions are:

```text
Who owns this value?
Who can borrow it?
When is it dropped?
Does this clone allocate?
```

## Stack

The **stack** is used for values whose size is known and fixed.

It is fast because Rust can push and pop values in order.

```rust
fn main() {
    let x = 10;
    let y = true;
    let arr = [1, 2, 3];
}
```

These can live on the stack:

```text
x: i32
y: bool
arr: [i32; 3]
```

The stack works roughly like this:

```text
main starts
  x is pushed
  y is pushed
  arr is pushed
main ends
  arr is popped
  y is popped
  x is popped
```

The stack is good for small, fixed-size values.

## Heap

The **heap** is used for values whose size may be unknown, large, or growable at runtime.

Examples:

```rust
let s = String::from("hello");
let v = vec![1, 2, 3];
```

`String` and `Vec<T>` store their actual data on the heap.

But the variables themselves are still stack values.

## String On Stack And Heap

```rust
let s = String::from("hello");
```

Memory looks roughly like this:

```text
Stack:
s:
  pointer -> heap data
  length: 5
  capacity: 5

Heap:
"h" "e" "l" "l" "o"
```

More accurate explanation:

```text
The String value is on the stack,
but it owns data on the heap.
```

This is why a `String` can grow:

```rust
let mut s = String::from("hello");
s.push_str(", world");
```

Rust cannot store all possible future string contents as one fixed-size stack value, so `String` uses heap memory for its text.

## Vec On Stack And Heap

```rust
let v = vec![1, 2, 3];
```

Memory looks roughly like this:

```text
Stack:
v:
  pointer -> heap data
  length: 3
  capacity: 3

Heap:
1 2 3
```

The `Vec<T>` metadata is on the stack.

The elements are on the heap.

## Array Usually Lives Directly On The Stack

```rust
let arr = [1, 2, 3];
```

Rust knows the exact size:

```rust
[i32; 3]
```

So the whole array can usually be stored directly on the stack.

By contrast:

```rust
let v = vec![1, 2, 3];
```

uses a `Vec`, so the actual elements are stored on the heap.

## Move Usually Copies Stack Metadata

```rust
let s1 = String::from("hello");
let s2 = s1;
```

Rust does **not** copy `"hello"` on the heap.

It moves the stack metadata:

```text
s1: pointer, length, capacity
s2: pointer, length, capacity
```

After the move, only `s2` is valid.

```rust
let s1 = String::from("hello");
let s2 = s1;

// println!("{}", s1); // error: s1 was moved
println!("{}", s2); // ok
```

This is why moving a `String` is cheap compared to cloning it.

## Clone Copies Heap Data

```rust
let s1 = String::from("hello");
let s2 = s1.clone();

println!("{}", s1);
println!("{}", s2);
```

`clone()` creates another owned copy.

For `String`, that usually means copying the heap data too.

## Borrowing Avoids Moving And Cloning

Borrowing lets code use a value without taking ownership.

```rust
fn print_text(text: &String) {
    println!("{}", text);
}

fn main() {
    let s = String::from("hello");

    print_text(&s);

    println!("{}", s); // still usable
}
```

The function borrows `s`, so ownership stays with `main`.

Usually this version is more flexible:

```rust
fn print_text(text: &str) {
    println!("{}", text);
}
```

`&str` can accept string slices from both `String` and string literals.

## String Literal Vs String

These are different:

```rust
let a = "hello";
let b = String::from("hello");
```

`a` has type:

```rust
&'static str
```

The text is stored in the program binary and lives for the whole program.

`b` has type:

```rust
String
```

It owns heap memory and can grow.

```rust
let mut b = String::from("hello");
b.push_str(", world");
```

## Box

`Box<T>` puts one value on the heap.

```rust
let x = Box::new(10);
```

Memory looks roughly like this:

```text
Stack:
x = pointer

Heap:
10
```

`Box<T>` is useful when:

- you want heap allocation for a single value
- you need recursive types
- you want a stable-sized pointer to a value

Example recursive type:

```rust
enum List {
    Cons(i32, Box<List>),
    Nil,
}
```

Without `Box`, Rust cannot know the size of `List`.

## Heap Allocation Has A Cost

Stack allocation is very cheap.

Heap allocation usually needs a memory allocator, so it is more expensive.

This does not mean "avoid heap always."

Better rule:

- use stack naturally for small fixed-size values
- use heap when you need dynamic size, ownership, or sharing
- avoid unnecessary `.clone()`

## Length And Capacity

`String` and `Vec<T>` have:

```text
pointer
length
capacity
```

Example:

```rust
let mut v = Vec::new();

v.push(1);
v.push(2);
v.push(3);
```

The vector may allocate extra capacity so it does not need to allocate every time you push.

You can create capacity upfront:

```rust
let v: Vec<i32> = Vec::with_capacity(10);

println!("{}", v.len());      // 0
println!("{}", v.capacity()); // 10
```

Length means how many items are currently in use.

Capacity means how many items can fit before reallocating.

## References Are Stack Values Too

```rust
let s = String::from("hello");
let r = &s;
```

`r` is a reference. It is a pointer-like value.

Memory looks roughly like this:

```text
Stack:
s = pointer, length, capacity
r = reference to s
```

But `r` does not own the string.

It only borrows it.

## Drop

Rust usually drops values automatically.

```rust
{
    let v = vec![1, 2, 3];
} // v is dropped here
```

When `v` is dropped, Rust frees its heap allocation.

This cleanup is deterministic: it happens when the value goes out of scope.

You do not manually call `free()` like in C.

## Ownership And Heap

Rust ownership is especially important for heap data.

```rust
let s1 = String::from("hello");
let s2 = s1;

// println!("{}", s1); // error
println!("{}", s2);
```

When `s1` moves to `s2`, Rust transfers ownership of the heap data.

Rust prevents both `s1` and `s2` from owning the same heap memory because that could cause double-free bugs.

## Stack Vs Heap Is A Mental Model

As a beginner, this mental model is useful:

- fixed-size local values usually use the stack
- dynamic owned data usually uses the heap

But compilers can optimize. Sometimes exact memory placement is different from the simple model.

So in Rust, focus more on ownership and borrowing than memorizing exact memory locations.

## Video Summary: Visualizing Rust Memory Layout

Video: [Visualizing memory layout of Rust's data types](https://www.youtube.com/watch?v=7_o-YRxf_cc)

The video explains how Rust values are represented in memory and why that matters for ownership, borrowing, moves, and performance.

When a Rust program runs, the operating system gives the process its own virtual address space. That memory is divided into areas such as:

| Memory area | Purpose |
|---|---|
| Text segment | Compiled machine instructions |
| Data segment | Initialized static/global variables |
| BSS segment | Uninitialized static/global variables |
| Stack | Function calls, local variables, parameters, return addresses |
| Heap | Dynamic allocations, such as data owned by `String`, `Vec<T>`, `Box<T>`, `Rc<T>`, and `Arc<T>` |

Each function call creates a **stack frame**.

A stack frame stores things like:

- function parameters
- local variables
- return address

When the function returns, its stack frame is removed.

This is why stack allocation is fast, but also why references to local variables cannot escape the function safely.

Example:

```rust
fn make_number() -> i32 {
    let x = 10;
    x
}
```

Returning `x` by value is okay because the value is copied or moved out.

But returning a reference to `x` would be invalid because `x` disappears when the function ends.

```rust
// This is invalid:
// fn bad() -> &i32 {
//     let x = 10;
//     &x
// }
```

## Video Summary: Common Type Layouts

Integer values are fixed-size values.

```rust
let n: i32 = 10;
```

An array stores a fixed number of elements.

```rust
let arr = [1, 2, 3];
```

A slice is not the array itself. It is a view into existing data.

```rust
let slice = &arr[0..2];
```

A slice is roughly:

```text
pointer + length
```

A `String` owns heap-allocated UTF-8 bytes.

```rust
let s = String::from("hello");
```

Roughly:

```text
Stack:
s = pointer, length, capacity

Heap:
"h" "e" "l" "l" "o"
```

A `Vec<T>` is similar.

```rust
let v = vec![1, 2, 3];
```

Roughly:

```text
Stack:
v = pointer, length, capacity

Heap:
1 2 3
```

A `struct` stores its fields together, with possible padding for alignment.

```rust
struct Point {
    x: i32,
    y: i32,
}
```

An `enum` stores which variant is active, plus enough space for the data of the active variant.

```rust
enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
}
```

So an enum is like:

```text
variant tag + variant data
```

## Video Summary: Smart Pointers

`Box<T>` owns one value on the heap.

```rust
let b = Box::new(10);
```

Roughly:

```text
Stack:
b = pointer

Heap:
10
```

`Rc<T>` allows multiple owners in a single thread.

```rust
use std::rc::Rc;

let a = Rc::new(String::from("hello"));
let b = Rc::clone(&a);
```

`Arc<T>` is similar to `Rc<T>`, but it is safe to share between threads because its reference count is updated atomically.

Trait objects are usually represented as **fat pointers**.

```rust
trait Draw {
    fn draw(&self);
}

let item: Box<dyn Draw>;
```

A trait object pointer is roughly:

```text
data pointer + vtable pointer
```

The data pointer points to the actual value.

The vtable pointer points to information about how to call the correct trait methods for that concrete type.

## Video Summary: Move, Clone, And Layout

This code moves ownership:

```rust
let s1 = String::from("hello");
let s2 = s1;
```

Rust usually moves the stack metadata:

```text
pointer, length, capacity
```

It does not copy the heap text `"hello"`.

After the move, only `s2` is valid.

This code clones:

```rust
let s1 = String::from("hello");
let s2 = s1.clone();
```

For `String`, cloning usually creates a new heap allocation and copies the text bytes.

So:

```text
move  = transfer ownership, usually cheap
clone = create a separate owned copy, may allocate
```

## Important Layout Caveat

The video is useful as a mental model, but exact Rust layout is not always guaranteed.

For normal Rust structs and enums, the compiler only guarantees what it needs for safe Rust behavior, such as:

- fields are properly aligned
- fields do not overlap
- the type alignment is at least the maximum alignment of its fields

Rust may choose different layouts for optimization.

If code must depend on a specific memory layout, such as for C interoperability, use attributes like:

```rust
#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}
```

Source: [Rust Reference: Type Layout](https://doc.rust-lang.org/stable/reference/type-layout.html)

## Best Beginner Mental Model

```rust
let x = 10;
// x owns a simple stack value

let arr = [1, 2, 3];
// arr is usually directly on the stack

let s = String::from("hello");
// s owns heap text through stack metadata

let s2 = s;
// ownership moves; heap text is not copied

let v = vec![1, 2, 3];
// v owns heap elements through stack metadata

let r = &v;
// r borrows v; it does not own the elements

let c = v.clone();
// c owns a new heap allocation with copied elements
```

## Most Important Ideas

1. Ownership decides who frees memory.
2. Move transfers ownership.
3. Borrow lets code use data without owning it.
4. Clone creates a separate owned copy.
5. Stack is fast and fixed-size.
6. Heap is flexible but involves allocation.
