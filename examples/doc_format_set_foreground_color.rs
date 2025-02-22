// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

//! The following example demonstrates setting the foreground/pattern color.

use rust_xlsxwriter::{Format, Workbook, XlsxColor, XlsxError, XlsxPattern};

fn main() -> Result<(), XlsxError> {
    // Create a new Excel file object.
    let mut workbook = Workbook::new();

    // Add a worksheet.
    let worksheet = workbook.add_worksheet();

    let format1 = Format::new()
        .set_background_color(XlsxColor::Yellow)
        .set_foreground_color(XlsxColor::Red)
        .set_pattern(XlsxPattern::DarkVertical);

    worksheet.write_blank(0, 0, &format1)?;

    workbook.save("formats.xlsx")?;

    Ok(())
}
