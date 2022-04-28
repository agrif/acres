fn main() {
    let aec = cmake::Config::new("libaec")
        .define("BUILD_SHARED_LIBS", "OFF")
        .build();
    println!("cargo:rustc-link-search=native={}/lib", aec.display());
    println!("cargo:rustc-link-lib=static=aec");
    println!("cargo:rustc-link-lib=static=sz");
}
