# Rust: Array vs Vector

In Rust, an **array** is fixed-size, while a **vector** (`Vec<T>`) is growable.

```rust
let arr: [i32; 3] = [1, 2, 3];

let mut vec: Vec<i32> = Vec::new();
vec.push(1);
vec.push(2);
vec.push(3);
```

## Main Differences

| Feature | Array | Vector |
|---|---|---|
| Type | `[T; N]` | `Vec<T>` |
| Size | Fixed at compile time | Can grow or shrink at runtime |
| Storage | Usually stack | Heap allocation |
| Length part of type? | Yes: `[i32; 3]` and `[i32; 4]` are different types | No |
| Mutability | Can change elements, but not length | Can change elements and length |
| Use when | Size is known and constant | Size is dynamic or unknown |

## Example

```rust
let mut arr = [1, 2, 3];
arr[0] = 10;
// arr.push(4); // error: arrays cannot grow

let mut vec = vec![1, 2, 3];
vec[0] = 10;
vec.push(4); // ok
```

## Both Can Be Used As Slices

Both arrays and vectors can be borrowed as slices:

```rust
fn print_items(items: &[i32]) {
    println!("{:?}", items);
}

let arr = [1, 2, 3];
let vec = vec![4, 5, 6];

print_items(&arr);
print_items(&vec);
```

## Rule Of Thumb

Use an **array** when the length is small and fixed.

Use a **vector** when you need a collection whose size can change.
