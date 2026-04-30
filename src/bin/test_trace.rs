#[cfg(test)]
fn main() {}

#[cfg(not(test))]
fn main() {
    // This runs as a binary but we need the lib internals
    // Instead, let's trace using a test in the lib
    println!("Build the test target");
}
