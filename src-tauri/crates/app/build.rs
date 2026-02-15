fn main() {
    dotenvy::dotenv().ok();

    for var in &["LASTFM_API_KEY", "LASTFM_API_SECRET", "DISCORD_CLIENT_ID"] {
        if let Ok(val) = std::env::var(var) {
            println!("cargo:rustc-env={var}={val}");
        } else {
            panic!("Missing {var} in .env for release build");
        }
    }
}
