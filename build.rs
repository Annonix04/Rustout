fn main() {
    println!("cargo:rerun-if-changed=app.res");
    println!("cargo:rustc-link-arg=/NODEFAULTLIB:libcmt");
    println!("cargo:rustc-link-arg=app.res");
}

