#![allow(dead_code)]
use std::process::Command;

#[derive(alox_48::Deserialize, Debug)]
// #[marshal(from = "alox_48::Userdata")]
struct Floats([f32; 3]);

impl From<alox_48::Userdata> for Floats {
    fn from(value: alox_48::Userdata) -> Self {
        let floats = bytemuck::cast_slice(&value.data);
        Self(std::array::from_fn(|i| floats[i]))
    }
}

fn main() {
    color_eyre::install().unwrap();

    let data = Command::new("ruby")
        .arg("-e")
        .arg(
            r#"
            class MyClass
                def _dump(_)
                    [15.0, 12.0, 13.0].pack('F*')
                end
            end
            
            puts Marshal.dump(MyClass.new)
        "#,
        )
        .output()
        .unwrap()
        .stdout;

    println!("{}", pretty_hex::pretty_hex(&data));

    let result: Floats = alox_48::from_bytes(&data).unwrap();

    println!("{result:#?}")
}
