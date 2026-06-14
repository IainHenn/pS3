fn main() -> Result<(), Box<dyn std::error::Error>>{
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=.sqlx");
    println!("cargo:rerun-if-changed=src/migrations");

    dotenvy::dotenv().ok();

    tonic_build::compile_protos("proto/helloworld.proto")?;
    Ok(())
}
