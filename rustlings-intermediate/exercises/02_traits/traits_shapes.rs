// Typing lecture: traits.
//
// Traits describe shared behavior. `Circle` and `Rectangle` are different
// types, but both can promise to provide `area` and `perimeter`.
//
// TODO: Fill in the trait implementations and the generic summary function.

use std::f64::consts::PI;
use std::fmt::Debug;

trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
}

#[derive(Debug)]
struct Circle {
    radius: f64,
}

#[derive(Debug)]
struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        todo!("calculate circle area");
    }

    fn perimeter(&self) -> f64 {
        todo!("calculate circle perimeter");
    }
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        todo!("calculate rectangle area");
    }

    fn perimeter(&self) -> f64 {
        todo!("calculate rectangle perimeter");
    }
}

fn describe_shape(shape: &(impl Shape + Debug)) -> String {
    todo!("format the debug value, area, and perimeter");
}

fn main() {
    let circle = Circle { radius: 2.0 };
    println!("{}", describe_shape(&circle));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close_enough(left: f64, right: f64) {
        assert!(
            (left - right).abs() < 0.001,
            "{left} was not close to {right}"
        );
    }

    #[test]
    fn circle_implements_shape() {
        let circle = Circle { radius: 3.0 };

        close_enough(circle.area(), 28.274);
        close_enough(circle.perimeter(), 18.849);
    }

    #[test]
    fn rectangle_implements_shape() {
        let rectangle = Rectangle {
            width: 4.0,
            height: 5.0,
        };

        close_enough(rectangle.area(), 20.0);
        close_enough(rectangle.perimeter(), 18.0);
    }

    #[test]
    fn summary_uses_trait_bound() {
        let rectangle = Rectangle {
            width: 2.0,
            height: 8.0,
        };

        let summary = describe_shape(&rectangle);
        assert!(summary.contains("Rectangle"));
        assert!(summary.contains("area=16.00"));
        assert!(summary.contains("perimeter=20.00"));
    }
}
