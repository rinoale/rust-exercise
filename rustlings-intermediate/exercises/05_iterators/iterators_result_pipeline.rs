// Typing lecture: iterator adaptors with `Result`.
//
// Iterator pipelines often combine parsing, filtering, flattening, and error
// propagation. This exercise practices `filter_map`, `flat_map`, `try_fold`,
// `enumerate`, and `collect`.
//
// TODO: Implement each function with iterator adaptors.

use std::num::ParseIntError;

fn parse_numbers_lossy(input: &str) -> Vec<i32> {
    todo!("parse comma-separated integers and skip invalid fields");
}

fn expand_inclusive_ranges(ranges: &[(i32, i32)]) -> Vec<i32> {
    todo!("expand every inclusive range and flatten the result");
}

fn checked_sum(items: &[&str]) -> Result<i32, ParseIntError> {
    todo!("parse and sum, returning the first parse error");
}

fn indexed_even_numbers(numbers: &[i32]) -> Vec<(usize, i32)> {
    todo!("return index/value pairs for even numbers");
}

fn main() {
    println!("{:?}", parse_numbers_lossy("1, two, 3"));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_numbers_and_skips_invalid_ones() {
        assert_eq!(parse_numbers_lossy("1, two, 3, 4x, 5"), vec![1, 3, 5]);
    }

    #[test]
    fn expands_ranges() {
        assert_eq!(
            expand_inclusive_ranges(&[(1, 3), (7, 8)]),
            vec![1, 2, 3, 7, 8]
        );
    }

    #[test]
    fn sums_or_returns_first_error() {
        assert_eq!(checked_sum(&["1", "2", "3"]), Ok(6));
        assert!(checked_sum(&["1", "bad", "3"]).is_err());
    }

    #[test]
    fn returns_even_numbers_with_original_indices() {
        assert_eq!(indexed_even_numbers(&[5, 8, 9, 12]), vec![(1, 8), (3, 12)]);
    }
}
