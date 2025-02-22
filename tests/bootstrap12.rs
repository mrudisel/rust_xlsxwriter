// Test case that compares a file generated by rust_xlsxwriter with a file
// created by Excel.
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

use rust_xlsxwriter::{Format, Workbook, XlsxError};

mod common;

// Test case to demonstrate creating a basic file with some string cell data.
fn create_new_xlsx_file(filename: &str) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    let format1 = Format::new().set_bold();
    let format2 = Format::new().set_italic();
    let format3 = Format::new().set_bold().set_italic();

    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "Hello", &format1)?;
    worksheet.write_string(1, 0, "Hello", &format2)?;
    worksheet.write_string(2, 0, "Hello", &format3)?;

    workbook.save(filename)?;

    // Also test multiple saves.
    workbook.save(filename)?;
    workbook.save(filename)?;
    workbook.save(filename)?;

    Ok(())
}

#[test]
fn bootstrap12_bold_and_italic_text_mixed() {
    let test_runner = common::TestRunner::new()
        .set_name("bootstrap12")
        .set_function(create_new_xlsx_file)
        .initialize();

    test_runner.assert_eq();
    test_runner.cleanup();
}
