// Test case that compares a file generated by rust_xlsxwriter with a file
// created by Excel.
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

use rust_xlsxwriter::{Format, Workbook, XlsxError};

mod common;

// Test to demonstrate worksheet protection.
fn create_new_xlsx_file(filename: &str) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    let worksheet = workbook.add_worksheet();

    let unlocked = Format::new().set_unlocked();
    let hidden = Format::new().set_unlocked().set_hidden();

    worksheet.write_number_only(0, 0, 1)?;
    worksheet.write_number(1, 0, 2, &unlocked)?;
    worksheet.write_number(2, 0, 3, &hidden)?;

    workbook.save(filename)?;

    Ok(())
}

#[test]
fn test_protect01() {
    let test_runner = common::TestRunner::new()
        .set_name("protect01")
        .set_function(create_new_xlsx_file)
        .initialize();

    test_runner.assert_eq();
    test_runner.cleanup();
}
