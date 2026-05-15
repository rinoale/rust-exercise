// Typing lecture: testing and documentation.
//
// Rust unit tests usually live beside the code they test. Documentation
// comments use Markdown and can include examples in real library crates.
//
// TODO: Implement the calculator so the unit tests pass.

#[derive(Debug, PartialEq, Eq)]
enum CalculatorError {
    DivideByZero,
}

struct Calculator;

impl Calculator {
    /// Adds two numbers.
    ///
    /// Example:
    /// `Calculator::add(2, 3)` returns `5`.
    fn add(left: i32, right: i32) -> i32 {
        todo!("add the two values");
    }

    /// Subtracts `right` from `left`.
    fn subtract(left: i32, right: i32) -> i32 {
        todo!("subtract right from left");
    }

    /// Multiplies two numbers.
    fn multiply(left: i32, right: i32) -> i32 {
        todo!("multiply the two values");
    }

    /// Divides `left` by `right`.
    ///
    /// Returns `CalculatorError::DivideByZero` when `right` is zero.
    fn divide(left: i32, right: i32) -> Result<i32, CalculatorError> {
        todo!("divide, but reject zero divisors");
    }
}

fn main() {
    println!("6 * 7 = {}", Calculator::multiply(6, 7));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_arithmetic_works() {
        assert_eq!(Calculator::add(2, 3), 5);
        assert_eq!(Calculator::subtract(10, 4), 6);
        assert_eq!(Calculator::multiply(6, 7), 42);
        assert_eq!(Calculator::divide(12, 3), Ok(4));
    }

    #[test]
    fn division_by_zero_is_an_error() {
        assert_eq!(
            Calculator::divide(12, 0),
            Err(CalculatorError::DivideByZero)
        );
    }

    #[test]
    fn addition_is_commutative_for_sample_values() {
        for left in -10..=10 {
            for right in -10..=10 {
                assert_eq!(Calculator::add(left, right), Calculator::add(right, left));
            }
        }
    }
}
