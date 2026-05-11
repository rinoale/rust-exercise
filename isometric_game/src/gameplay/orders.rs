#[derive(Clone, Copy)]
pub enum Order {
    Meal,
    Drink,
    Both,
}

impl Order {
    pub fn label(self) -> &'static str {
        match self {
            Order::Meal => "meal",
            Order::Drink => "drink",
            Order::Both => "meal and drink",
        }
    }

    pub fn food_cost(self) -> i32 {
        match self {
            Order::Meal | Order::Both => 1,
            Order::Drink => 0,
        }
    }

    pub fn water_cost(self) -> i32 {
        match self {
            Order::Drink | Order::Both => 1,
            Order::Meal => 0,
        }
    }

    pub fn scrap_value(self) -> i32 {
        match self {
            Order::Meal => 2,
            Order::Drink => 1,
            Order::Both => 3,
        }
    }
}
