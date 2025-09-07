fn main() {
    println!("cargo:rustc-link-lib=framework=ApplicationServices");
    println!("cargo:rustc-link-lib=framework=Carbon");
    
    cc::Build::new()
        .file("src/ax_constants.c")
        .compile("ax_constants");
}