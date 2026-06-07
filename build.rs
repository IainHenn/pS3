fn main() {
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=.sqlx");
    println!("cargo:rerun-if-changed=src/migrations");

    dotenvy::dotenv().ok();
}
