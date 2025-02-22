// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

//! The following example demonstrates writing unformatted numbers to an Excel
//! worksheet. Any numeric type that will convert [`Into`] f64 can be
//! transferred to Excel.

use rust_xlsxwriter::{Workbook, XlsxError};

fn main() -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    // Add a worksheet to the workbook.
    let worksheet = workbook.add_worksheet();

    // Write some different rust number types to a worksheet.
    // Note, u64 isn't supported by Excel.
    worksheet.write_number_only(0, 0, 1_u8)?;
    worksheet.write_number_only(1, 0, 2_i16)?;
    worksheet.write_number_only(2, 0, 3_u32)?;
    worksheet.write_number_only(3, 0, 4_f32)?;
    worksheet.write_number_only(4, 0, 5_f64)?;

    // Write some numbers with implicit types.
    worksheet.write_number_only(5, 0, 1234)?;
    worksheet.write_number_only(6, 0, 1234.5)?;

    // Note Excel normally ignores trailing decimal zeros
    // when the number is unformatted.
    worksheet.write_number_only(7, 0, 1234.50000)?;

    workbook.save("numbers.xlsx")?;

    Ok(())
}
