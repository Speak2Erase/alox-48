#![allow(dead_code)]
use std::{collections::HashMap, process::Command};

#[derive(Debug, alox_48::Deserialize, Default)]
struct MyClass {
    #[marshal(skip)]
    #[marshal(default = "default_test")]
    test: Test,
    #[marshal(deserialize_with = "with_test")]
    bool: i32,
}

#[derive(Debug, alox_48::Deserialize, Default)]
struct Test {
    map: HashMap<String, bool>,
}

fn default_test() -> Test {
    Test {
        map: HashMap::new(),
    }
}

fn with_test<'de, D>(deserializer: D) -> Result<i32, alox_48::DeError>
where
    D: alox_48::DeserializerTrait<'de>,
{
    let bool: bool = alox_48::Deserialize::deserialize(deserializer)?;
    Ok(bool as i32 >> 4)
}

fn main() {
    color_eyre::install().unwrap();

    let data = Command::new("ruby")
        .arg("-e")
        .arg(
            r#"
        class MyClass
            def initialize()
                @bool = true
            end
        end
        
        klass = MyClass.new
        puts Marshal.dump(klass)
      "#,
        )
        .output()
        .unwrap()
        .stdout;

    println!("{}", pretty_hex::pretty_hex(&data));

    let other: MyClass = alox_48::from_bytes(&data).unwrap();

    println!("{other:?}",);
}
