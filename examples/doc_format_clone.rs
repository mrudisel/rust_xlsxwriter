// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

//! The following example demonstrates cloning a format and setting the
//! properties.

use rust_xlsxwriter::{Format, Workbook, XlsxColor, XlsxError};

fn main() -> Result<(), XlsxError> {
    // Create a new Excel file object.
    let mut workbook = Workbook::new();

    // Add a worksheet.
    let worksheet = workbook.add_worksheet();

    // Create a new format and set some properties.
    let format1 = Format::new().set_bold();

    // Clone a new format and set some properties.
    let format2 = format1.clone().set_font_color(XlsxColor::Blue);

    worksheet.write_string(0, 0, "Hello", &format1)?;
    worksheet.write_string(1, 0, "Hello", &format2)?;

    workbook.save("formats.xlsx")?;

    Ok(())
}
