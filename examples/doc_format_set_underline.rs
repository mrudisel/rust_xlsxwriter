// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

//! The following example demonstrates setting underline properties for a
//! format.

use rust_xlsxwriter::{Format, Workbook, XlsxError, XlsxUnderline};

fn main() -> Result<(), XlsxError> {
    // Create a new Excel file object.
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let format1 = Format::new().set_underline(XlsxUnderline::None);
    let format2 = Format::new().set_underline(XlsxUnderline::Single);
    let format3 = Format::new().set_underline(XlsxUnderline::Double);
    let format4 = Format::new().set_underline(XlsxUnderline::SingleAccounting);
    let format5 = Format::new().set_underline(XlsxUnderline::DoubleAccounting);

    worksheet.write_string(0, 0, "None", &format1)?;
    worksheet.write_string(1, 0, "Single", &format2)?;
    worksheet.write_string(2, 0, "Double", &format3)?;
    worksheet.write_string(3, 0, "Single Accounting", &format4)?;
    worksheet.write_string(4, 0, "Double Accounting", &format5)?;

    workbook.save("formats.xlsx")?;

    Ok(())
}
