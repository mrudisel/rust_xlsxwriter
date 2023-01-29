// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

//! The following example demonstrates getting worksheet reference by name.

use rust_xlsxwriter::{Workbook, XlsxError};

fn main() -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    // Start with a reference to worksheet1.
    let mut worksheet1 = workbook.add_worksheet();
    let name1 = worksheet1.name().to_owned(); // "Sheet1"
    worksheet1.write_string_only(0, 0, "Hello")?;

    // If we don't try to use the workbook1 reference again we can switch to
    // using a reference to worksheet2.
    let mut worksheet2 = workbook.add_worksheet().set_name("Data")?;
    let name2 = worksheet2.name().to_owned();
    worksheet2.write_string_only(0, 0, "Hello")?;

    // Stop using worksheet2 and move back to worksheet1.
    worksheet1 = workbook.worksheet_from_name(&name1)?;
    worksheet1.write_string_only(1, 0, "Sheet1")?;

    // Stop using worksheet1 and move back to worksheet2.
    worksheet2 = workbook.worksheet_from_name(&name2)?;
    worksheet2.write_string_only(1, 0, "Sheet2")?;

    workbook.save("workbook.xlsx")?;

    Ok(())
}
