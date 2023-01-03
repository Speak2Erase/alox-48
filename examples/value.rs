#![allow(dead_code)]
use std::process::Command;

fn main() {
    color_eyre::install().unwrap();

    Command::new("ruby")
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
            Marshal.dump(klass, File.open("value.marshal", "wb"))
        "#,
        )
        .status()
        .unwrap();

    let data = std::fs::read("value.marshal").unwrap();

    assert_eq!(
        data[0..=1],
        [4, 8],
        "The data should have version number 4.8"
    );

    println!("{}", pretty_hex::pretty_hex(&data));

    let result: Result<alox_48::Value, _> = alox_48::from_bytes(&data);

    println!("{result:#?}")
}
