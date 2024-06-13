// Copyright (c) 2024 Lily Lyons
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

fn main() {
    let bytes = [
        4, 8, 123, 7, 58, 11, 111, 98, 106, 101, 99, 116, 111, 58, 11, 79, 98, 106, 101, 99, 116,
        0, 58, 19, 115, 101, 108, 102, 95, 114, 101, 102, 101, 114, 101, 110, 99, 101, 91, 8, 64,
        6, 64, 6, 64, 6,
    ];

    let value: alox_48::Value = alox_48::from_bytes(&bytes).unwrap();
    println!("{:?}", value);
}
