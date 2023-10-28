use rand::{Rng, distributions::Alphanumeric};

fn main() {
    let key: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(21)
        .map(char::from)
        .collect();

    std::fs::write("../.apikey", key).unwrap();
}