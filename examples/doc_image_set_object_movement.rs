// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

//! This example shows how to create an image object and set the option to
//! control how it behaves when the cells underneath it are changed.

use rust_xlsxwriter::{Image, Workbook, XlsxError, XlsxObjectMovement};

fn main() -> Result<(), XlsxError> {
    // Create a new Excel file object.
    let mut workbook = Workbook::new();

    // Add a worksheet to the workbook.
    let worksheet = workbook.add_worksheet();

    // Create a new image object.
    let mut image = Image::new("examples/rust_logo.png")?;

    // Set the object movement/positioning options.
    image.set_object_movement(XlsxObjectMovement::MoveButDontSizeWithCells);

    // Insert the image.
    worksheet.insert_image(1, 2, &image)?;

    // Save the file to disk.
    workbook.save("image.xlsx")?;

    Ok(())
}
