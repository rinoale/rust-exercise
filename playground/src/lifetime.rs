pub struct User<'a> {
    name: &'a str,
}

fn main() {
    let name = "gbsong";

    let user = User { name: &name };
    println!("{}",user.name)
}
