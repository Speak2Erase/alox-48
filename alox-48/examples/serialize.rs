#[derive(alox_48::Deserialize, alox_48::Serialize, Debug)]
struct MyStruct {
    test: String,
}

#[derive(alox_48::Deserialize, alox_48::Serialize, Debug)]
struct Foo(String);

#[rustfmt::skip]
const BYTES: &[u8] = &[
    0x04, 0x08, 0x6F, 0x3A, 0x0D, 0x4D, 0x79, 0x53, 0x74, 0x72, 0x75, 0x63, 0x74, 0x06, 0x3A, 0x0A, // ..o:.MyStruct.:.
    0x40, 0x74, 0x65, 0x73, 0x74, 0x49, 0x22, 0x08, 0x68, 0x69, 0x21, 0x06, 0x3A, 0x06, 0x45, 0x54, // @testI".hi!.:.ET
];

fn main() {
    let output = alox_48::to_bytes(MyStruct {
        test: "hi!".to_string(),
    })
    .unwrap();

    println!("{}", pretty_hex::pretty_hex(&output));

    assert_eq!(output, BYTES, "Output does not match expected bytes");

    let output: MyStruct = alox_48::from_bytes(&output).unwrap();

    println!("{output:#?}")
}
