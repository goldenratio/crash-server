echo "generate schema!"
touch src/generated/hello.rs
echo "pub fn get_num() -> String { "42".to_string() }" > src/generated/hello.rs
flatc --version