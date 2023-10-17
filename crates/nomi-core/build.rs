use std::io::Write;

fn main() {
    let mut file = std::fs::File::create("./build.log").unwrap();
    let binding = std::env::current_dir().unwrap();
    let path = binding.to_string_lossy();
    writeln!(file, "{}", path).unwrap();
    writeln!(file, "{}", path).unwrap();
}
