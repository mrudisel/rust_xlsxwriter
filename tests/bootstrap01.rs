// Test case that compares a file generated by rust_xlsxwriter with a file
// created by Excel.
//
// SPDX-License-Identifier: MIT OR Apache-2.0 Copyright 2022, John McNamara,
// jmcnamara@cpan.org

use rust_xlsxwriter::{Workbook, XlsxError};

mod common;

// Test case to demonstrate creating a basic file with 1 worksheet and no data.
fn create_new_xlsx_file1(filename: &str) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new(filename);
    _ = workbook.add_worksheet();

    workbook.close()?;

    Ok(())
}

// Test case to demonstrate creating a basic file with 1 worksheet and no data.
// Has an implicit add_worksheet.
fn create_new_xlsx_file2(filename: &str) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new(filename);

    workbook.close()?;

    Ok(())
}

#[test]
fn bootstrap01_single_worksheet() {
    let testcase = "bootstrap01";

    let (excel_file, xlsxwriter_file) = common::get_xlsx_filenames_unique(testcase, 'a');
    _ = create_new_xlsx_file1(&xlsxwriter_file);
    common::assert_eq(&excel_file, &xlsxwriter_file);
}

#[test]
fn bootstrap01_add_default_worksheet() {
    let testcase = "bootstrap01";

    let (excel_file, xlsxwriter_file) = common::get_xlsx_filenames_unique(testcase, 'b');
    _ = create_new_xlsx_file2(&xlsxwriter_file);
    common::assert_eq(&excel_file, &xlsxwriter_file);
    common::remove_test_xlsx_file(&xlsxwriter_file);
}
