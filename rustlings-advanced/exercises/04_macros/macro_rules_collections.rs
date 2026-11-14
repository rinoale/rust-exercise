// Typing lecture: macro repetition and optional trailing commas.
//
// Declarative macros are often used to build small collection literals or DSLs.
// This exercise practices recursive counting, repetition, and token patterns.
//
// TODO: Replace the catch-all arms with real macro rules.

use std::collections::HashMap;

macro_rules! count_exprs {
    ($($tokens:tt)*) => {{
        let count: usize = todo!("count comma-separated expressions");
        count
    }};
}

macro_rules! maplit {
    ($($tokens:tt)*) => {
        todo!("build a HashMap from key => value pairs")
    };
}

fn main() {
    let values: HashMap<&str, i32> = maplit!("one" => 1, "two" => 2);
    println!("{} entries", values.len());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_expressions() {
        assert_eq!(count_exprs!(), 0);
        assert_eq!(count_exprs!(1), 1);
        assert_eq!(count_exprs!("a", 2 + 3, true,), 3);
    }

    #[test]
    fn builds_hash_maps() {
        let values: HashMap<&str, i32> = maplit!(
            "rust" => 2015,
            "edition" => 2024,
        );

        assert_eq!(values.get("rust"), Some(&2015));
        assert_eq!(values.get("edition"), Some(&2024));
    }
}
