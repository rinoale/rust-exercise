// Typing lecture: advanced traits.
//
// Associated types let a trait name the output type once in the implementation
// instead of repeating it at every call site. The newtype pattern lets us
// implement local traits for wrapped external types.
//
// TODO: Complete the converters.

trait Converter {
    type Output;

    fn convert(self) -> Self::Output;
}

struct IntVec(Vec<i32>);

impl Converter for String {
    type Output = Vec<u8>;

    fn convert(self) -> Self::Output {
        todo!("convert the string into bytes");
    }
}

impl Converter for i32 {
    type Output = String;

    fn convert(self) -> Self::Output {
        todo!("convert the integer into a string");
    }
}

impl Converter for IntVec {
    type Output = i32;

    fn convert(self) -> Self::Output {
        todo!("sum the wrapped vector");
    }
}

fn convert_twice<C>(left: C, right: C) -> (C::Output, C::Output)
where
    C: Converter,
{
    (left.convert(), right.convert())
}

fn main() {
    let bytes = String::from("advanced").convert();
    println!("{bytes:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_converts_to_bytes() {
        assert_eq!(String::from("Rust").convert(), vec![82, 117, 115, 116]);
    }

    #[test]
    fn integer_converts_to_string() {
        assert_eq!(42_i32.convert(), "42");
    }

    #[test]
    fn newtype_converts_to_sum() {
        assert_eq!(IntVec(vec![1, 2, 3, 4]).convert(), 10);
    }

    #[test]
    fn associated_output_can_be_named() {
        let pair = convert_twice(7_i32, 9_i32);

        assert_eq!(pair, (String::from("7"), String::from("9")));
    }
}
