mod math;
mod utils;

use crate::math::gcd;
use crate::utils::input;

fn main() {
    let a = input::read_int();
    let b = input::read_int();
    println!("{}", gcd(a, b));
}
