// Typing lecture: iterator adaptors with `Result`.

use std::num::ParseIntError;

fn parse_numbers_lossy(input: &str) -> Vec<i32> {
    input
        .split(',')
        .filter_map(|field| field.trim().parse::<i32>().ok())
        .collect()
}

fn expand_inclusive_ranges(ranges: &[(i32, i32)]) -> Vec<i32> {
    ranges
        .iter()
        .flat_map(|&(start, end)| start..=end)
        .collect()
}

fn checked_sum(items: &[&str]) -> Result<i32, ParseIntError> {
    items.iter().try_fold(0, |total, item| {
        item.parse::<i32>().map(|value| total + value)
    })
}

fn indexed_even_numbers(numbers: &[i32]) -> Vec<(usize, i32)> {
    numbers
        .iter()
        .copied()
        .enumerate()
        .filter(|pair| pair.1 % 2 == 0)
        .collect()
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
