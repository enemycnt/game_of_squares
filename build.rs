extern crate embed_resource;
macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning={}", format!($($tokens)*))
    }
}

fn main() {
    p!("Build script");
    let target = std::env::var("TARGET").unwrap();
    p!("{:?}", target);
    if target.contains("windows") {
        // std::env::set_var("PATH", "/usr/local/opt/llvm/bin:$PATH");
        p!("yes, windows");
        embed_resource::compile("build/windows/icon.rc", embed_resource::NONE);
    }
}
