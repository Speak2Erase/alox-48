// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[derive(alox_48::Serialize, Debug)]
struct Test {
    a: i32,
    b: i32,

    throws: Throws,
}

#[derive(alox_48::Serialize, Debug)]
struct Throws {
    string: &'static str,
    #[marshal(serialize_with = "throws_error")]
    what_errors: (),
}

fn throws_error<T, S>(_: &T, _serializer: S) -> alox_48::SerResult<S::Ok>
where
    S: alox_48::SerializerTrait,
{
    Err(alox_48::SerError::custom("custom error"))
}

fn main() {
    let data = [
        0x04, 0x08, 0x5b, 0x06, 0x6f, 0x3a, 0x0c, 0x42, 0x61, 0x64, 0x44, 0x61, 0x74, 0x61, 0x08,
        0x3a, 0x0a, 0x40, 0x67, 0x6f, 0x6f, 0x64, 0x5b, 0x06, 0x49, 0x22, 0x13, 0x68, 0x66, 0x6a,
        0x76, 0x68, 0x6a, 0x76, 0x6a, 0x68, 0x76, 0x68, 0x6a, 0x76, 0x6c, 0x06, 0x3a, 0x06, 0x45,
        0x54, 0x3a, 0x09, 0x40, 0x62, 0x61, 0x64, 0x7b, 0x06, 0x3a, 0x09, 0x6f, 0x6f, 0x70, 0x73,
        0x49, 0x22, 0x19, 0x73, 0x6f, 0x6d, 0x65, 0x74, 0x68, 0x69, 0x6e, 0x67, 0x20, 0x77, 0x65,
        0x6e, 0x74, 0x20, 0x77, 0x72, 0x6f, 0x6e, 0x67, 0x06, 0x3b, 0x07, 0x54, 0x3a, 0x0f, 0x40,
        0x61, 0x66, 0x74, 0x65, 0x72, 0x5f, 0x62, 0x61, 0x64, 0x49, 0x22, 0x2e, 0x69, 0x6d, 0x20,
        0x61, 0x20, 0x73, 0x74, 0x72, 0x69, 0x6e, 0x67, 0x21, 0x20, 0x6e, 0x6f, 0x74, 0x68, 0x69,
        0x6e, 0x67, 0x20, 0x63, 0x61, 0x6e, 0x20, 0x67, 0x6f, 0x20, 0x77, 0x72, 0x6f, 0x6e, 0x67,
        0x20, 0x68, 0x65, 0x72, 0x65, 0x20, 0x3a, 0x29, 0x06, 0x3b, 0x07, 0x54,
    ];

    for i in 2..data.len() {
        let mut data = data;
        data[i] = b',';

        let mut deserializer = alox_48::Deserializer::new(&data).unwrap();
        let Err((error, trace)) =
            alox_48::path_to_error::deserialize::<alox_48::Value>(&mut deserializer)
        else {
            continue;
        };

        println!("Error: {error}");
        println!("Backtrace:");
        for ctx in trace.context.iter().rev() {
            println!("    {ctx}");
        }
    }

    let test = Test {
        a: 1,
        b: 2,
        throws: Throws {
            string: "something went wrong",
            what_errors: (),
        },
    };

    let mut serializer = alox_48::Serializer::new();
    let (error, trace) =
        alox_48::path_to_error::serialize(&test, &mut serializer).expect_err("what??");

    println!("Error: {error}");
    println!("Backtrace:");
    for ctx in trace.context.iter().rev() {
        println!("    {ctx}");
    }

    let (error, trace) =
        alox_48::path_to_error::serialize(&test, alox_48::ValueSerializer).expect_err("what??");

    println!("Error: {error}");
    println!("Backtrace:");
    for ctx in trace.context.iter().rev() {
        println!("    {ctx}");
    }
}
