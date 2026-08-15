fn main() {
    let standalone = std::env::var("CARGO_FEATURE_STANDALONE").is_ok();
    let cloudflare = std::env::var("CARGO_FEATURE_CLOUDFLARE").is_ok();
    if standalone && cloudflare {
        panic!("features `standalone` and `cloudflare` are mutually exclusive — enable only one");
    }
}
