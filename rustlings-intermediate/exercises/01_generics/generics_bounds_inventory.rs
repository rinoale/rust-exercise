// Typing lecture: generics with trait bounds.
//
// Generic code becomes useful when you can say what a type must support.
// This exercise practices `where` clauses, trait bounds, cloning, formatting,
// and domain traits.
//
// TODO: Fill the generic helpers without changing the public signatures.

use std::fmt::Display;

trait Priced {
    fn price_cents(&self) -> u32;
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Product {
    name: &'static str,
    cents: u32,
}

impl Product {
    fn new(name: &'static str, cents: u32) -> Self {
        Self { name, cents }
    }
}

impl Display for Product {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}c", self.name, self.cents)
    }
}

impl Priced for Product {
    fn price_cents(&self) -> u32 {
        self.cents
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Inventory<T> {
    items: Vec<T>,
}

impl<T> Inventory<T> {
    fn new(items: Vec<T>) -> Self {
        Self { items }
    }

    fn len(&self) -> usize {
        self.items.len()
    }
}

impl<T> Inventory<T>
where
    T: PartialEq,
{
    fn contains(&self, needle: &T) -> bool {
        todo!("return whether the inventory contains needle");
    }
}

impl<T> Inventory<T>
where
    T: Display,
{
    fn labels(&self) -> Vec<String> {
        todo!("format every item with Display");
    }
}

fn duplicate_all<T>(items: &[T]) -> Vec<T>
where
    T: Clone,
{
    todo!("clone every item from the slice into a new Vec");
}

fn total_price<T>(items: &[T]) -> u32
where
    T: Priced,
{
    todo!("sum the price of every item");
}

fn main() {
    let inventory = Inventory::new(vec![Product::new("book", 1500)]);
    println!("{} item(s)", inventory.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_items_using_partial_eq() {
        let inventory = Inventory::new(vec![Product::new("book", 1500), Product::new("pen", 200)]);

        assert!(inventory.contains(&Product::new("book", 1500)));
        assert!(!inventory.contains(&Product::new("notebook", 3000)));
    }

    #[test]
    fn formats_labels_using_display() {
        let inventory = Inventory::new(vec![Product::new("book", 1500), Product::new("pen", 200)]);

        assert_eq!(
            inventory.labels(),
            vec![String::from("book:1500c"), String::from("pen:200c")]
        );
    }

    #[test]
    fn clones_and_prices_generic_items() {
        let products = vec![Product::new("book", 1500), Product::new("pen", 200)];

        assert_eq!(duplicate_all(&products), products);
        assert_eq!(total_price(&products), 1700);
    }
}
