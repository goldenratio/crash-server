echo "generate schema!"
if ! command -v flatc &> /dev/null
then
    echo "flatc could not be found!"
    exit 0
fi

flatc --version
touch src/generated/hello.rs
echo "pub fn get_num() -> String { "42".to_string() }" > src/generated/hello.rs