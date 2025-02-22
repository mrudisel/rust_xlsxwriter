// Test case that compares a file generated by rust_xlsxwriter with a file
// created by Excel.
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

use rust_xlsxwriter::{Format, Workbook, XlsxError};

mod common;

// Test case to demonstrate creating a basic file with number formats.
fn create_new_xlsx_file(filename: &str) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    let format1 = Format::new().set_num_format_index(2);
    let format2 = Format::new().set_num_format_index(10);
    let format3 = Format::new().set_num_format_index(49);

    let worksheet = workbook.add_worksheet();
    worksheet.write_number(0, 0, 1, &format1)?;
    worksheet.write_number(1, 1, 2, &format2)?;
    worksheet.write_number(2, 2, 3, &format3)?;

    workbook.save(filename)?;

    Ok(())
}

#[test]
fn bootstrap14_number_format_via_legacy_index() {
    let test_runner = common::TestRunner::new()
        .set_name("bootstrap15")
        .set_function(create_new_xlsx_file)
        .initialize();

    test_runner.assert_eq();
    test_runner.cleanup();
}
