// Test case that compares a file generated by rust_xlsxwriter with a file
// created by Excel.
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

use rust_xlsxwriter::{Workbook, XlsxError};

mod common;

// Test case to test array formulas, single cell range and set formula result.
fn create_new_xlsx_file(filename: &str) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    let worksheet = workbook.add_worksheet();

    worksheet.write_array_formula_only(0, 0, 2, 0, "{=SUM(B1:C1*B2:C2)}")?;

    worksheet.set_formula_result(0, 0, "0");

    workbook.save(filename)?;

    Ok(())
}

#[test]
fn test_array_formula04() {
    let test_runner = common::TestRunner::new()
        .set_name("array_formula04")
        .set_function(create_new_xlsx_file)
        .ignore_calc_chain()
        .initialize();

    test_runner.assert_eq();
    test_runner.cleanup();
}
