#![allow(dead_code)]
use std::process::Command;

fn main() {
    color_eyre::install().unwrap();

    let data = Command::new("ruby")
        .arg("-e")
        .arg(
            r#"
class MyClass
    def initialize()
        @test = Test.new
        @bool = true
    end
end

class Test
    def initialize()
        @map = {
            :abc => true
        }
    end
end

klass = MyClass.new
print Marshal.dump(klass)
        "#,
        )
        .output()
        .unwrap()
        .stdout;

    assert_eq!(
        data[0..=1],
        [4, 8],
        "The data should have version number 4.8"
    );

    println!("{}", pretty_hex::pretty_hex(&data));

    let result: Result<alox_48::Value, _> = alox_48::from_bytes(&data);

    println!("{result:#?}")
}
