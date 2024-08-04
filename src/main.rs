use dotenvy::dotenv;
use std::env;
struct Env {
    test: String,
}
impl Env {
    fn new() -> Self {
        dotenv().ok();

        Env {
            test: env::var("TEST").expect("TEST is not set"),
        }
    }
}

fn main() {
    let env = Env::new();
    println!("TEST: {}", env.test);
}
