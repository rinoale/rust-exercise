# Rust: Ownership, Borrow, And Move

Rust ownership is about **who is responsible for a value**.

Core rules:

1. Every value has one owner.
2. When the owner goes out of scope, the value is dropped.
3. Ownership can be moved to another variable.
4. Values can be borrowed without taking ownership.

## Ownership

```rust
let s = String::from("hello");
```

Here, `s` owns the `String`.

When `s` leaves scope, Rust automatically frees the value:

```rust
{
    let s = String::from("hello");
} // s is dropped here
```

## Move

A **move** transfers ownership from one variable to another.

```rust
let s1 = String::from("hello");
let s2 = s1;

// println!("{}", s1); // error: s1 was moved
println!("{}", s2); // ok
```

After `s1` moves into `s2`, `s1` can no longer be used.

This prevents two variables from both thinking they own the same heap memory.

## Copy

Some simple types do not move. They are copied automatically.

```rust
let x = 5;
let y = x;

println!("{}", x); // ok
println!("{}", y); // ok
```

Types like `i32`, `bool`, `char`, `f64`, and tuples containing only `Copy` types are usually `Copy`.

So this is a copy:

```rust
let a = 10;
let b = a;
```

But this is a move:

```rust
let s1 = String::from("hello");
let s2 = s1;
```

## Clone

Use `clone()` when you want a real duplicate of non-`Copy` data.

```rust
let s1 = String::from("hello");
let s2 = s1.clone();

println!("{}", s1); // ok
println!("{}", s2); // ok
```

`clone()` can be more expensive because it may copy heap data.

## Borrow

A **borrow** lets code use a value without taking ownership.

```rust
let s = String::from("hello");

let len = calculate_length(&s);

println!("{}", s); // still ok

fn calculate_length(text: &String) -> usize {
    text.len()
}
```

`&s` means "borrow `s` immutably."

The function can read the value, but it does not own it.

## Immutable Borrow

You can have many immutable borrows at the same time.

```rust
let s = String::from("hello");

let r1 = &s;
let r2 = &s;

println!("{}, {}", r1, r2);
```

This is allowed because nobody can change the value through immutable references.

## Mutable Borrow

A mutable borrow lets code change a value without taking ownership.

```rust
let mut s = String::from("hello");

change(&mut s);

fn change(text: &mut String) {
    text.push_str(", world");
}
```

Important rule: you can only have **one mutable borrow** at a time.

```rust
let mut s = String::from("hello");

let r1 = &mut s;
// let r2 = &mut s; // error

println!("{}", r1);
```

You also cannot mix mutable and immutable borrows at the same time:

```rust
let mut s = String::from("hello");

let r1 = &s;
// let r2 = &mut s; // error

println!("{}", r1);
```

## Drop

`drop` means Rust cleans up a value when its owner goes out of scope.

You can also call `drop()` manually:

```rust
let s = String::from("hello");
drop(s);

// println!("{}", s); // error: s was dropped
```

After a value is dropped, it cannot be used again.

## Shadowing

Shadowing creates a new variable with the same name.

```rust
let x = String::from("hello");
let x = x.len();

println!("{}", x); // 5
```

The first `x` was a `String`.

The second `x` is a `usize`.

Shadowing is different from mutation because it creates a new binding.

## Lifetime

A **lifetime** describes how long a reference is valid.

Rust checks lifetimes to make sure references do not outlive the values they point to.

```rust
let r;

{
    let x = 5;
    r = &x;
} // x is dropped here

// println!("{}", r); // error: r points to dropped data
```

## Summary Of Ownership-Related Behaviors

| Behavior | Meaning |
|---|---|
| Ownership | A variable is responsible for a value |
| Move | Ownership transfers to another variable |
| Copy | Simple values are duplicated automatically |
| Clone | Data is explicitly duplicated |
| Borrow | A value is used without taking ownership |
| Immutable borrow | Read-only borrow with `&T` |
| Mutable borrow | Changeable borrow with `&mut T` |
| Drop | A value is cleaned up when its owner ends |
| Shadowing | A new variable uses the same name |
| Lifetime | Rust checks how long references are valid |

## Simple Mental Model

```rust
let s1 = String::from("hello"); // s1 owns the String
let s2 = s1;                    // ownership moves to s2
let s3 = &s2;                   // s3 borrows from s2
let s4 = s2.clone();            // s4 owns a separate copy
```

## Rule Of Thumb

Use ownership when a function should take responsibility for a value.

Use `&T` when a function only needs to read.

Use `&mut T` when a function needs to modify.

Use `.clone()` only when you really need another owned copy.
