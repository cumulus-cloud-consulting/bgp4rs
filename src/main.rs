use app::args::parse;

pub mod shared;
pub mod app;

#[tokio::main (flavor = "multi_thread")]
async fn main() {
    if let Ok(args) = parse() {
        println!("Hello, world!");
    } else {
        println!("Error!");

    }
}
