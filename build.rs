fn main() -> Result<(), Box<dyn std::error::Error>>{
    println!("cargo:rerun-if-changed=.env");
    println!("cargo:rerun-if-changed=.sqlx");
    println!("cargo:rerun-if-changed=src/migrations");
    println!("cargo:rerun-if-changed=src/grpc/proto/ps3.proto");

    dotenvy::dotenv().ok();

    let out_dir = "src/grpc/generated";
    std::fs::create_dir_all(out_dir)?;

    tonic_build::configure()
        .out_dir(out_dir)
        .compile_protos(&["src/grpc/proto/ps3.proto"], &["src/grpc/proto"])?;
    Ok(())
}
