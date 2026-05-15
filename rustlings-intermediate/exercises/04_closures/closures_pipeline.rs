// Typing lecture: closures.
//
// Closures let callers pass behavior into a function. This pipeline accepts
// three closures: one to keep values, one to transform them, and one to combine
// them into a result.
//
// TODO: Complete the pipeline with iterator adaptors.

fn filter_map_reduce<Filter, Map, Reduce>(
    values: Vec<i32>,
    filter: Filter,
    map: Map,
    initial: i32,
    reduce: Reduce,
) -> i32
where
    Filter: Fn(&i32) -> bool,
    Map: Fn(i32) -> i32,
    Reduce: Fn(i32, i32) -> i32,
{
    todo!("filter, map, then fold the values");
}

fn main() {
    let answer = filter_map_reduce(vec![1, 2, 3, 4], |n| n % 2 == 0, |n| n * 2, 0, |a, b| a + b);
    println!("{answer}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_maps_and_reduces() {
        let result = filter_map_reduce(
            vec![1, 2, 3, 4, 5, 6],
            |n| n % 2 == 0,
            |n| n * 2,
            0,
            |acc, n| acc + n,
        );

        assert_eq!(result, 24);
    }

    #[test]
    fn closures_can_capture_environment() {
        let threshold = 3;
        let multiplier = 10;

        let result = filter_map_reduce(
            vec![1, 2, 3, 4, 5],
            |n| *n > threshold,
            |n| n * multiplier,
            100,
            |acc, n| acc + n,
        );

        assert_eq!(result, 190);
    }
}
