// Test case that compares a file generated by rust_xlsxwriter with a file
// created by Excel.
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

use rust_xlsxwriter::{Format, Workbook, Worksheet, XlsxError};

mod common;

// Test to demonstrate activated and selected worksheets.
fn create_new_xlsx_file(filename: &str) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    let mut worksheet1 = Worksheet::new();
    let mut worksheet2 = Worksheet::new();
    let mut worksheet3 = Worksheet::new();

    let bold = Format::new().set_bold();

    worksheet1.write_string_only(0, 0, "Foo")?;
    worksheet1.write_number_only(1, 0, 123)?;

    worksheet3.write_string_only(1, 1, "Foo")?;
    worksheet3.write_string(2, 1, "Bar", &bold)?;
    worksheet3.write_number_only(3, 2, 234)?;

    worksheet2.set_name("Data Sheet")?;
    worksheet2.set_active(true); // This should be overridden by workbook3.

    worksheet2.set_selected(true);
    worksheet3.set_selected(true);
    worksheet3.set_active(true);

    workbook.push_worksheet(worksheet1);
    workbook.push_worksheet(worksheet2);
    workbook.push_worksheet(worksheet3);

    workbook.save(filename)?;

    Ok(())
}

#[test]
fn test_simple03() {
    let test_runner = common::TestRunner::new()
        .set_name("simple03")
        .set_function(create_new_xlsx_file)
        .initialize();

    test_runner.assert_eq();
    test_runner.cleanup();
}
