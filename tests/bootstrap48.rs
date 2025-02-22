// Test case that compares a file generated by rust_xlsxwriter with a file
// created by Excel.
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

use rust_xlsxwriter::{Format, Workbook, XlsxBorder, XlsxColor, XlsxError, XlsxPattern};

mod common;

// Test case to demonstrate creating a basic file with theme colors.
fn create_new_xlsx_file(filename: &str) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let format1 = Format::new().set_background_color(XlsxColor::Theme(5, 0));
    let format2 = Format::new().set_background_color(XlsxColor::Theme(5, 1));
    let format3 = Format::new().set_background_color(XlsxColor::Theme(5, 2));
    let format4 = Format::new().set_background_color(XlsxColor::Theme(5, 3));
    let format5 = Format::new().set_background_color(XlsxColor::Theme(5, 4));
    let format6 = Format::new().set_background_color(XlsxColor::Theme(5, 5));

    let format7 = Format::new()
        .set_border(XlsxBorder::Thin)
        .set_border_color(XlsxColor::Theme(9, 0));

    let format8 = Format::new()
        .set_background_color(XlsxColor::Theme(9, 1))
        .set_foreground_color(XlsxColor::Theme(9, 5))
        .set_pattern(XlsxPattern::DarkHorizontal);

    worksheet.write_blank(1, 1, &format1)?;
    worksheet.write_blank(3, 1, &format2)?;
    worksheet.write_blank(5, 1, &format3)?;
    worksheet.write_blank(7, 1, &format4)?;
    worksheet.write_blank(9, 1, &format5)?;
    worksheet.write_blank(11, 1, &format6)?;
    worksheet.write_blank(13, 1, &format7)?;
    worksheet.write_blank(15, 1, &format8)?;

    worksheet.set_tab_color(XlsxColor::Theme(4, 0));

    workbook.save(filename)?;

    Ok(())
}

#[test]
fn bootstrap48_theme_colors() {
    let test_runner = common::TestRunner::new()
        .set_name("bootstrap48")
        .set_function(create_new_xlsx_file)
        .initialize();

    test_runner.assert_eq();
    test_runner.cleanup();
}
