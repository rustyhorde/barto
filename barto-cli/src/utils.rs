// Copyright (c) 2025 barto developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or https://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

use strip_ansi_escapes::strip;
use unicode_width::UnicodeWidthStr as _;

pub(crate) fn clean_output_string(data: &str) -> (String, usize) {
    let data = data.replace('\t', "   ");
    let data = data.replace('\n', " ");
    let data = data.replace('\r', " ");
    let final_data = String::from_utf8_lossy(&strip(data)).to_string();
    let data_uw = final_data.width();
    (final_data, data_uw)
}
