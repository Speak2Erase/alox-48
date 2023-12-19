use std::process::Command;

use alox_48::Symbol;

fn main() {
    color_eyre::install().unwrap();

    let data = Command::new("ruby")
        .arg("-e")
        .arg("puts Marshal.dump(:hello)")
        .output()
        .unwrap()
        .stdout;

    let data: Symbol = alox_48::from_bytes(&data).unwrap();

    println!("{data}")
}
