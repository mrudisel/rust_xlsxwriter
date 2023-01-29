// workbook - A module for creating the Excel workbook.xml file.
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

#![warn(missing_docs)]

use std::collections::HashMap;
use std::io::{Cursor, Seek, Write};
use std::mem;
use std::path::Path;

use crate::error::XlsxError;
use crate::format::Format;
use crate::packager::Packager;
use crate::packager::PackagerOptions;
use crate::worksheet::Worksheet;
use crate::xmlwriter::XMLWriter;
use crate::{utility, DefinedName, DefinedNameType, Properties, NUM_IMAGE_FORMATS};
use crate::{XlsxColor, XlsxPattern};

/// The Workbook struct represents an Excel file in it's entirety. It is the
/// starting point for creating a new Excel xlsx file.
///
/// <img src="https://rustxlsxwriter.github.io/images/demo.png">
///
/// # Examples
///
/// Sample code to generate the Excel file shown above.
///
/// ```rust
/// # // This code is available in examples/app_demo.rs
/// #
/// use chrono::NaiveDate;
/// use rust_xlsxwriter::{Format, Image, Workbook, XlsxAlign, XlsxBorder, XlsxError};
///
/// fn main() -> Result<(), XlsxError> {
///     // Create a new Excel file object.
///     let mut workbook = Workbook::new();
///
///     // Create some formats to use in the worksheet.
///     let bold_format = Format::new().set_bold();
///     let decimal_format = Format::new().set_num_format("0.000");
///     let date_format = Format::new().set_num_format("yyyy-mm-dd");
///     let merge_format = Format::new()
///         .set_border(XlsxBorder::Thin)
///         .set_align(XlsxAlign::Center);
///
///     // Add a worksheet to the workbook.
///     let worksheet = workbook.add_worksheet();
///
///     // Set the column width for clarity.
///     worksheet.set_column_width(0, 22)?;
///
///     // Write a string without formatting.
///     worksheet.write_string_only(0, 0, "Hello")?;
///
///     // Write a string with the bold format defined above.
///     worksheet.write_string(1, 0, "World", &bold_format)?;
///
///     // Write some numbers.
///     worksheet.write_number_only(2, 0, 1)?;
///     worksheet.write_number_only(3, 0, 2.34)?;
///
///     // Write a number with formatting.
///     worksheet.write_number(4, 0, 3.00, &decimal_format)?;
///
///     // Write a formula.
///     worksheet.write_formula_only(5, 0, "=SIN(PI()/4)")?;
///
///     // Write a date.
///     let date = NaiveDate::from_ymd_opt(2023, 1, 25).unwrap();
///     worksheet.write_date(6, 0, &date, &date_format)?;
///
///     // Write some links.
///     worksheet.write_url(7, 0, "https://www.rust-lang.org")?;
///     worksheet.write_url_with_text(8, 0, "https://www.rust-lang.org", "Learn Rust")?;
///
///     // Write some merged cells.
///     worksheet.merge_range(9, 0, 9, 1, "Merged cells", &merge_format)?;
///
///     // Insert an image.
///     let image = Image::new("examples/rust_logo.png")?;
///     worksheet.insert_image(1, 2, &image)?;
///
///     // Save the file to disk.
///     workbook.save("demo.xlsx")?;
///
///     Ok(())
/// }
/// ```
pub struct Workbook {
    pub(crate) writer: XMLWriter,
    pub(crate) properties: Properties,
    pub(crate) worksheets: Vec<Worksheet>,
    pub(crate) xf_formats: Vec<Format>,
    pub(crate) font_count: u16,
    pub(crate) fill_count: u16,
    pub(crate) border_count: u16,
    pub(crate) num_format_count: u16,
    pub(crate) has_hyperlink_style: bool,
    xf_indices: HashMap<String, u32>,
    active_tab: u16,
    first_sheet: u16,
    defined_names: Vec<DefinedName>,
    user_defined_names: Vec<DefinedName>,
    read_only_mode: u8,
}

impl Default for Workbook {
    fn default() -> Self {
        Self::new()
    }
}

impl Workbook {
    // -----------------------------------------------------------------------
    // Public (and crate public) methods.
    // -----------------------------------------------------------------------

    /// Create a new Workbook object to represent an Excel spreadsheet file.
    ///
    /// The `Workbook::new()` constructor is used to create a new Excel workbook
    /// object. This is used to create worksheets and add data prior to saving
    /// everything to an xlsx file with [`save()`](Workbook::save), or
    /// [`save_to_buffer()`](Workbook::save_to_buffer).
    ///
    /// **Note**: `rust_xlsxwriter` can only create new files. It cannot read or
    /// modify existing files.
    ///
    /// # Examples
    ///
    /// The following example demonstrates creating a simple workbook, with one
    /// unused worksheet.
    ///
    /// ```
    /// # // This code is available in examples/doc_workbook_new.rs
    /// #
    /// # use rust_xlsxwriter::{Workbook, XlsxError};
    ///
    /// fn main() -> Result<(), XlsxError> {
    ///     let mut workbook = Workbook::new();
    ///
    ///     let _worksheet = workbook.add_worksheet();
    ///
    ///     workbook.save("workbook.xlsx")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Output file:
    ///
    /// <img src="https://rustxlsxwriter.github.io/images/workbook_new.png">
    ///
    pub fn new() -> Workbook {
        let writer = XMLWriter::new();

        let mut workbook = Workbook {
            writer,
            properties: Properties::new(),
            font_count: 0,
            active_tab: 0,
            fill_count: 0,
            first_sheet: 0,
            border_count: 0,
            num_format_count: 0,
            read_only_mode: 0,
            has_hyperlink_style: false,
            worksheets: vec![],
            xf_formats: vec![],
            defined_names: vec![],
            user_defined_names: vec![],
            xf_indices: HashMap::new(),
        };

        // Initialize the workbook with the same function used to reset it.
        Self::reset(&mut workbook);

        workbook
    }

    /// Add a new worksheet to a workbook.
    ///
    /// The `add_worksheet()` method adds a new [`worksheet`](Worksheet) to a
    /// workbook.
    ///
    /// The worksheets will be given standard Excel name like `Sheet1`,
    /// `Sheet2`, etc. Alternatively, the name can be set using
    /// `worksheet.set_name()`, see the example below and the docs for
    /// [`worksheet.set_name()`](Worksheet::set_name).
    ///
    /// The `add_worksheet()` method returns a borrowed mutable reference to a
    /// Worksheet instance owned by the Workbook so only one worksheet can be in
    /// existence at a time, see the example below. This limitation can be
    /// avoided, if necessary, by creating standalone Worksheet objects via
    /// [`Worksheet::new()`] and then later adding them to the workbook with
    /// [`workbook.push_worksheet`](Workbook::push_worksheet).
    ///
    /// See also the `rust_xlsxwriter` documentation on [Creating worksheets]
    /// and working with the borrow checker.
    ///
    /// [Creating worksheets]: https://rustxlsxwriter.github.io/worksheet/create.html
    ///
    /// # Examples
    ///
    /// The following example demonstrates creating adding worksheets to a
    /// workbook.
    ///
    /// ```
    /// # // This code is available in examples/doc_workbook_add_worksheet.rs
    /// #
    /// # use rust_xlsxwriter::{Workbook, XlsxError};
    ///
    /// fn main() -> Result<(), XlsxError> {
    ///     let mut workbook = Workbook::new();
    ///
    ///     let worksheet = workbook.add_worksheet(); // Sheet1
    ///     worksheet.write_string_only(0, 0, "Hello")?;
    ///
    ///     let worksheet = workbook.add_worksheet().set_name("Foglio2")?;
    ///     worksheet.write_string_only(0, 0, "Hello")?;
    ///
    ///     let worksheet = workbook.add_worksheet(); // Sheet3
    ///     worksheet.write_string_only(0, 0, "Hello")?;
    ///
    ///     workbook.save("workbook.xlsx")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/workbook_add_worksheet.png">
    ///
    pub fn add_worksheet(&mut self) -> &mut Worksheet {
        let name = format!("Sheet{}", self.worksheets.len() + 1);

        let mut worksheet = Worksheet::new();
        worksheet.set_name(name).unwrap();

        self.worksheets.push(worksheet);
        let worksheet = self.worksheets.last_mut().unwrap();

        worksheet
    }

    /// Get a worksheet reference by index.
    ///
    /// Get a reference to a worksheet created via
    /// [`workbook.add_worksheet()`](Workbook::add_worksheet) using an index
    /// based on the creation order.
    ///
    /// Due to borrow checking rules you can only have one active reference to a
    /// worksheet object created by `add_worksheet()` since that method always
    /// returns a mutable reference. For a workbook with multiple worksheets
    /// this restriction is generally workable if you can create and use the
    /// worksheets sequentially since you will only need to have one reference
    /// at any one time. However, if you can't structure your code to work
    /// sequentially then you get a reference to a previously created worksheet
    /// using `worksheet_from_index()`. The standard borrow checking rules still
    /// apply so you will have to give up ownership of any other worksheet
    /// reference prior to calling this method. See the example below.
    ///
    /// See also [`worksheet_from_name()`](Workbook::worksheet_from_name) and
    /// the `rust_xlsxwriter` documentation on [Creating worksheets]
    /// and working with the borrow checker.
    ///
    /// [Creating worksheets]: https://rustxlsxwriter.github.io/worksheet/create.html
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the worksheet to get a reference to.
    ///
    /// # Errors
    ///
    /// * [`XlsxError::UnknownWorksheetNameOrIndex`] - Error when trying to
    ///   retrieve a worksheet reference by index. This is usually an index out
    ///   of bounds error.
    ///
    /// # Examples
    ///
    /// The following example demonstrates getting worksheet reference by index.
    ///
    /// ```
    /// # // This code is available in examples/doc_workbook_worksheet_from_index.rs
    /// #
    /// # use rust_xlsxwriter::{Workbook, XlsxError};
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    /// #
    ///     // Start with a reference to worksheet1.
    ///     let mut worksheet1 = workbook.add_worksheet();
    ///     worksheet1.write_string_only(0, 0, "Hello")?;
    ///
    ///     // If we don't try to use the workbook1 reference again we can switch to
    ///     // using a reference to worksheet2.
    ///     let mut worksheet2 = workbook.add_worksheet();
    ///     worksheet2.write_string_only(0, 0, "Hello")?;
    ///
    ///     // Stop using worksheet2 and move back to worksheet1.
    ///     worksheet1 = workbook.worksheet_from_index(0)?;
    ///     worksheet1.write_string_only(1, 0, "Sheet1")?;
    ///
    ///     // Stop using worksheet1 and move back to worksheet2.
    ///     worksheet2 = workbook.worksheet_from_index(1)?;
    ///     worksheet2.write_string_only(1, 0, "Sheet2")?;
    ///
    /// #     workbook.save("workbook.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/workbook_worksheet_from_index.png">
    ///
    pub fn worksheet_from_index(&mut self, index: usize) -> Result<&mut Worksheet, XlsxError> {
        if let Some(worksheet) = self.worksheets.get_mut(index) {
            Ok(worksheet)
        } else {
            Err(XlsxError::UnknownWorksheetNameOrIndex(index.to_string()))
        }
    }

    /// Get a worksheet reference by name.
    ///
    /// Get a reference to a worksheet created via
    /// [`workbook.add_worksheet()`](Workbook::add_worksheet) using the sheet
    /// name.
    ///
    /// Due to borrow checking rules you can only have one active reference to a
    /// worksheet object created by `add_worksheet()` since that method always
    /// returns a mutable reference. For a workbook with multiple worksheets
    /// this restriction is generally workable if you can create and use the
    /// worksheets sequentially since you will only need to have one reference
    /// at any one time. However, if you can't structure your code to work
    /// sequentially then you get a reference to a previously created worksheet
    /// using `worksheet_from_name()`. The standard borrow checking rules still
    /// apply so you will have to give up ownership of any other worksheet
    /// reference prior to calling this method. See the example below.
    ///
    /// Worksheet names are usually "Sheet1", "Sheet2", etc., or else a user
    /// define name that was set using
    /// [`worksheet.set_name()`](Worksheet::set_name). You can also use the
    /// [`worksheet.name()`](Worksheet::name) method to get the name.
    ///
    /// See also [`worksheet_from_index()`](Workbook::worksheet_from_index) and
    /// the `rust_xlsxwriter` documentation on [Creating worksheets] and working
    /// with the borrow checker.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the worksheet to get a reference to.
    ///
    /// # Errors
    ///
    /// * [`XlsxError::UnknownWorksheetNameOrIndex`] - Error when trying to
    ///   retrieve a worksheet reference by index. This is usually an index out
    ///   of bounds error.
    ///
    /// # Examples
    ///
    /// The following example demonstrates getting worksheet reference by index.
    ///
    /// ```
    /// # // This code is available in examples/doc_workbook_worksheet_from_index.rs
    /// #
    /// # use rust_xlsxwriter::{Workbook, XlsxError};
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    ///
    ///     // Start with a reference to worksheet1.
    ///     let mut worksheet1 = workbook.add_worksheet();
    ///     worksheet1.write_string_only(0, 0, "Hello")?;
    ///
    ///     // If we don't try to use the workbook1 reference again we can switch to
    ///     // using a reference to worksheet2.
    ///     let mut worksheet2 = workbook.add_worksheet();
    ///     worksheet2.write_string_only(0, 0, "Hello")?;
    ///
    ///     // Stop using worksheet2 and move back to worksheet1.
    ///     worksheet1 = workbook.worksheet_from_index(0)?;
    ///     worksheet1.write_string_only(1, 0, "Sheet1")?;
    ///
    ///     // Stop using worksheet1 and move back to worksheet2.
    ///     worksheet2 = workbook.worksheet_from_index(1)?;
    ///     worksheet2.write_string_only(1, 0, "Sheet2")?;
    ///
    /// #     workbook.save("workbook.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/workbook_worksheet_from_name.png">
    ///
    pub fn worksheet_from_name(&mut self, sheetname: &str) -> Result<&mut Worksheet, XlsxError> {
        for (index, worksheet) in self.worksheets.iter_mut().enumerate() {
            if sheetname == worksheet.name {
                return self.worksheet_from_index(index);
            }
        }

        // If we didn't find a matching sheet name then raise
        Err(XlsxError::UnknownWorksheetNameOrIndex(
            sheetname.to_string(),
        ))
    }

    /// Get a mutable reference to the vector of worksheets.
    ///
    /// Get a mutable reference to the vector of Worksheets used by the Workbook
    /// instance. This can be useful for iterating over, and performing the same
    /// operation, on all the worksheets in the workbook. See the example below.
    ///
    /// If you are careful you can also use some of the standard [slice]
    /// operations on the vector reference, see below.
    ///
    /// See also the `rust_xlsxwriter` documentation on [Creating worksheets]
    /// and working with the borrow checker.
    ///
    /// [Creating worksheets]:
    ///     https://rustxlsxwriter.github.io/worksheet/create.html
    ///
    /// # Examples
    ///
    /// The following example demonstrates operating on the vector of all the
    /// worksheets in a workbook.
    ///
    /// ```
    /// # // This code is available in examples/doc_workbook_worksheets_mut.rs
    /// #
    /// # use rust_xlsxwriter::{Workbook, XlsxError};
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    /// #
    ///     // Add three worksheets to the workbook.
    ///     let _ = workbook.add_worksheet();
    ///     let _ = workbook.add_worksheet();
    ///     let _ = workbook.add_worksheet();
    ///
    ///     // Write the same data to all three worksheets.
    ///     for worksheet in workbook.worksheets_mut() {
    ///         worksheet.write_string_only(0, 0, "Hello")?;
    ///         worksheet.write_number_only(1, 0, 12345)?;
    ///     }
    ///
    ///     // If you are careful you can use standard slice operations.
    ///     workbook.worksheets_mut().swap(0, 1);
    /// #
    /// #     workbook.save("workbook.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file, note the same data is in all three worksheets and Sheet2
    /// and Sheet1 have swapped position:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/workbook_worksheets_mut.png">
    ///
    pub fn worksheets_mut(&mut self) -> &mut Vec<Worksheet> {
        &mut self.worksheets
    }

    /// Get a reference to the vector of worksheets.
    ///
    /// Get a reference to the vector of Worksheets used by the Workbook
    /// instance. This is less useful than
    /// [`worksheets_mut`](Workbook::worksheets_mut) version since a mutable
    /// reference is required for most worksheet operations.
    ///
    /// # Examples
    ///
    /// The following example demonstrates operating on the vector of all the
    /// worksheets in a workbook. The non mutable version of this method is less
    /// useful than `workbook.worksheets_mut()`.
    ///
    /// ```
    /// # // This code is available in examples/doc_workbook_worksheets.rs
    /// #
    /// # use rust_xlsxwriter::{Workbook, XlsxError};
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    ///
    ///     // Add three worksheets to the workbook.
    ///     let _worksheet1 = workbook.add_worksheet();
    ///     let _worksheet2 = workbook.add_worksheet();
    ///     let _worksheet3 = workbook.add_worksheet();
    ///
    ///     // Get some information from all three worksheets.
    ///     for worksheet in workbook.worksheets() {
    ///         println!("{}", worksheet.name());
    ///     }
    ///
    /// #     workbook.save("workbook.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    pub fn worksheets(&self) -> &Vec<Worksheet> {
        &self.worksheets
    }

    /// Add a worksheet object to a workbook.
    ///
    /// Add a worksheet created directly using `Workbook::new()` to a workbook.
    ///
    /// There are two way of creating a worksheet object with rust_xlsxwriter:
    /// via the [`workbook.add_worksheet()`](Workbook::add_worksheet) method and
    /// via the [`Worksheet::new()`] constructor. The first method ties the
    /// worksheet to the workbook object that will write it automatically when
    /// the file is saved, whereas the second method creates a worksheet that is
    /// independent of a workbook. This has certain advantages in keeping the
    /// worksheet free of the workbook borrow checking until you wish to add it.
    ///
    /// When working with the independent worksheet object you can add it to a
    /// workbook using `push_worksheet()`, see the example below.
    ///
    /// See also the `rust_xlsxwriter` documentation on [Creating worksheets]
    /// and working with the borrow checker.
    ///
    /// [Creating worksheets]: https://rustxlsxwriter.github.io/worksheet/create.html
    ///
    /// # Arguments
    ///
    /// * `worksheet` - The worksheet to add to the workbook.
    ///
    /// # Examples
    ///
    /// The following example demonstrates creating a standalone worksheet
    /// object and then adding to a workbook.
    ///
    /// ```
    /// # // This code is available in examples/doc_workbook_push_worksheet.rs
    /// #
    /// # use rust_xlsxwriter::{Workbook, Worksheet, XlsxError};
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    ///     let mut worksheet = Worksheet::new();
    ///
    ///     // Use the worksheet object.
    ///     worksheet.write_string_only(0, 0, "Hello")?;
    ///
    ///     // Add it to the workbook.
    ///     workbook.push_worksheet(worksheet);
    ///
    ///     // Save the workbook.
    /// #     workbook.save("workbook.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/workbook_push_worksheet.png">
    ///
    pub fn push_worksheet(&mut self, mut worksheet: Worksheet) {
        if worksheet.name().is_empty() {
            let name = format!("Sheet{}", self.worksheets.len() + 1);
            worksheet.set_name(name).unwrap();
        }

        self.worksheets.push(worksheet);
    }

    /// Save the Workbook as an xlsx file.
    ///
    /// The workbook `save()` method writes all the Workbook data to a new xlsx
    /// file. It will overwrite any existing file.
    ///
    /// The `save()` method can be called multiple times so it is possible to
    /// get incremental files at different stages of a process, or to save the
    /// same Workbook object to different paths. However, `save()` is an
    /// expensive operation which assembles multiple files into an xlsx/zip
    /// container so for performance reasons you shouldn't call it
    /// unnecessarily.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the new Excel file to create as a `&str` or as a
    ///   [`std::path`] Path or PathBuf instance.
    ///
    /// # Errors
    ///
    /// * [`XlsxError::SheetnameReused`] - Worksheet name is already in use in
    ///   the workbook.
    /// * [`XlsxError::IoError`] - A wrapper for various IO errors when creating
    ///   the xlsx file, or its sub-files.
    /// * [`XlsxError::ZipError`] - A wrapper for various zip errors when
    ///   creating the xlsx file, or its sub-files.
    ///
    /// # Examples
    ///
    /// The following example demonstrates creating a simple workbook, with one
    /// unused worksheet.
    ///
    /// ```
    /// # // This code is available in examples/doc_workbook_save.rs
    /// #
    /// # use rust_xlsxwriter::{Workbook, XlsxError};
    ///
    /// fn main() -> Result<(), XlsxError> {
    ///     let mut workbook = Workbook::new();
    ///
    ///     let _worksheet = workbook.add_worksheet();
    ///
    ///     workbook.save("workbook.xlsx")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> Result<(), XlsxError> {
        if cfg!(feature = "test_resave") {
            // Some test code to test double/multiple saves.
            let file = std::fs::File::create(<&std::path::Path>::clone(&path.as_ref()))?;
            self.save_internal(file)?;
        }

        let file = std::fs::File::create(path)?;
        self.save_internal(file)?;
        Ok(())
    }

    /// Save the Workbook as an xlsx file and return it as a byte vector.
    ///
    /// The workbook `save_to_buffer()` method is similar to the
    /// [`save()`](Workbook::save) method except that it returns the xlsx file
    /// as a `Vec<u8>` buffer suitable for streaming in a web application.
    ///
    /// # Errors
    ///
    /// * [`XlsxError::SheetnameReused`] - Worksheet name is already in use in
    ///   the workbook.
    /// * [`XlsxError::IoError`] - A wrapper for various IO errors when creating
    ///   the xlsx file, or its sub-files.
    /// * [`XlsxError::ZipError`] - A wrapper for various zip errors when
    ///   creating the xlsx file, or its sub-files.
    ///
    /// # Examples
    ///
    /// The following example demonstrates creating a simple workbook to a
    /// `Vec<u8>` buffer.
    ///
    /// ```
    /// # // This code is available in examples/doc_workbook_save_to_buffer.rs
    /// #
    /// # use rust_xlsxwriter::{Workbook, XlsxError};
    ///
    /// fn main() -> Result<(), XlsxError> {
    ///     let mut workbook = Workbook::new();
    ///
    ///     let worksheet = workbook.add_worksheet();
    ///     worksheet.write_string_only(0, 0, "Hello")?;
    ///
    ///     let buf = workbook.save_to_buffer()?;
    ///
    ///     println!("File size: {}", buf.len());
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    pub fn save_to_buffer(&mut self) -> Result<Vec<u8>, XlsxError> {
        let mut buf = vec![];
        let cursor = Cursor::new(&mut buf);
        self.save_internal(cursor)?;
        Ok(buf)
    }

    // Set the index for the format. This is currently only used in testing but
    // may be used publicly at a later stage.
    #[doc(hidden)]
    pub fn register_format(&mut self, format: &mut Format) {
        let format_key = format.format_key();

        match self.xf_indices.get_mut(&format_key) {
            Some(xf_index) => {
                format.set_xf_index(*xf_index);
            }
            None => {
                let xf_index = self.xf_formats.len() as u32;
                self.xf_formats.push(format.clone());
                format.set_xf_index(xf_index);

                self.xf_indices.insert(format_key, xf_index);
            }
        }
    }

    /// Create a defined name in the workbook to use as a variable.
    ///
    /// The `define_name()` method is used to defined a variable name that can
    /// be used to represent a value, a single cell or a range of cells in a
    /// workbook. These are sometimes referred to as a "Named Ranges".
    ///
    /// Defined names are generally used to simplify or clarify formulas by
    /// using descriptive variable names. For example:
    ///
    /// ```text
    ///     // Global workbook name.
    ///     workbook.define_name("Exchange_rate", "=0.96")?;
    ///     worksheet.write_formula_only(0, 0, "=Exchange_rate")?;
    /// ```
    ///
    /// A name defined like this is "global" to the workbook and can be used in
    /// any worksheet in the workbook.  It is also possible to define a
    /// local/worksheet name by prefixing it with the sheet name using the
    /// syntax `"sheetname!defined_name"`:
    ///
    /// ```text
    ///     // Local worksheet name.
    ///     workbook.define_name('Sheet2!Sales', '=Sheet2!$G$1:$G$10')?;
    /// ```
    ///
    /// See the full example below.
    ///
    /// Note, Excel has limitations on names used in defined names. For example
    /// it must start with a letter or underscore and cannot contain a space or
    /// any of the characters: `,/*[]:\"'`. It also cannot look like an Excel
    /// range such as `A1`, `XFD12345` or `R1C1`. If in doubt it best to test
    /// the name in Excel first.
    ///
    /// For local defined names sheet name must exist (at the time of saving)
    /// and if the sheet name contains spaces or special characters you must
    /// follow the Excel convention and enclose it in single quotes:
    ///
    /// ```text
    ///     workbook.define_name("'New Data'!Sales", ""=Sheet2!$G$1:$G$10")?;
    /// ```
    ///
    /// The rules for names in Excel are explained in the Microsoft Office
    /// documentation on how to [Define and use names in
    /// formulas](https://support.microsoft.com/en-us/office/define-and-use-names-in-formulas-4d0f13ac-53b7-422e-afd2-abd7ff379c64)
    /// and subsections.
    ///
    /// # Arguments
    ///
    /// * `name` - The variable name to define.
    /// * `formula` - The formula, value or range that the name defines..
    ///
    /// # Errors
    ///
    /// * [`XlsxError::ParameterError`] - The following Excel error cases will
    ///   raise a `ParameterError` error:
    ///   * If the name doesn't start with a letter or underscore.
    ///   * If the name contains `,/*[]:\"'` or `space`.
    ///
    /// # Examples
    ///
    /// Example of how to create defined names using the rust_xlsxwriter
    /// library.
    ///
    /// This functionality is used to define user friendly variable names to
    /// represent a value, a single cell,  or a range of cells in a workbook.
    ///
    /// ```
    /// # // This code is available in examples/app_defined_name.rs
    /// #
    /// # use rust_xlsxwriter::{Workbook, XlsxError};
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     // Create a new Excel file object.
    /// #     let mut workbook = Workbook::new();
    /// #
    /// #     // Add two worksheets to the workbook.
    /// #     let _worksheet1 = workbook.add_worksheet();
    /// #     let _worksheet2 = workbook.add_worksheet();
    /// #
    ///     // Define some global/workbook names.
    ///     workbook.define_name("Exchange_rate", "=0.96")?;
    ///     workbook.define_name("Sales", "=Sheet1!$G$1:$H$10")?;
    ///
    ///     // Define a local/worksheet name. Over-rides the "Sales" name above.
    ///     workbook.define_name("Sheet2!Sales", "=Sheet2!$G$1:$G$10")?;
    ///
    /// #     // Write some text in the file and one of the defined names in a formula.
    /// #     for worksheet in workbook.worksheets_mut() {
    /// #         worksheet.set_column_width(0, 45)?;
    /// #         worksheet.write_string_only(0, 0, "This worksheet contains some defined names.")?;
    /// #         worksheet.write_string_only(1, 0, "See Formulas -> Name Manager above.")?;
    /// #         worksheet.write_string_only(2, 0, "Example formula in cell B3 ->")?;
    /// #
    /// #         worksheet.write_formula_only(2, 1, "=Exchange_rate")?;
    /// #     }
    /// #
    /// #     // Save the file to disk.
    /// #     workbook.save("defined_name.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/app_defined_name1.png">
    ///
    /// Here is the output in the Excel Name Manager. Note that there is a
    /// Global/Workbook "Sales" variable name and a Local/Worksheet version.
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/app_defined_name2.png">
    ///
    pub fn define_name(&mut self, name: &str, formula: &str) -> Result<&mut Workbook, XlsxError> {
        let mut defined_name = DefinedName::new();

        // Match Global/Workbook or Local/Worksheet defined names.
        match name.find('!') {
            Some(position) => {
                defined_name.quoted_sheet_name = name[0..position].to_string();
                defined_name.name = name[position + 1..].to_string();
                defined_name.name_type = DefinedNameType::Local;
            }
            None => {
                defined_name.name = name.to_string();
                defined_name.name_type = DefinedNameType::Global;
            }
        }

        // Excel requires that the name starts with a letter or underscore.
        // Also, backspace is allowed but undocumented by Excel.
        if !defined_name.name.chars().next().unwrap().is_alphabetic()
            && !defined_name.name.starts_with('_')
            && !defined_name.name.starts_with('\\')
        {
            let error = format!(
                "Name '{}' must start with a letter or underscore in Excel",
                defined_name.name
            );
            return Err(XlsxError::ParameterError(error));
        }

        // Excel also prohibits certain characters in the name.
        if defined_name
            .name
            .contains([' ', ',', '/', '*', '[', ']', ':', '"', '\''])
        {
            let error = format!(
                "Name '{}' cannot contain any of the characters `,/*[]:\"'` or `space` in Excel",
                defined_name.name
            );
            return Err(XlsxError::ParameterError(error));
        }

        defined_name.range = utility::formula_to_string(formula);
        defined_name.set_sort_name();

        self.user_defined_names.push(defined_name);

        Ok(self)
    }

    /// Set the Excel document metadata properties.
    ///
    /// Set various Excel document metadata properties such as Author or
    /// Creation Date. It is used in conjunction with the [`Properties`] struct.
    ///
    /// # Arguments
    ///
    /// * `properties` - A reference to a [`Properties`] object.
    ///
    /// # Examples
    ///
    /// An example of setting workbook document properties for a file created
    /// using the rust_xlsxwriter library.
    ///
    /// ```
    /// # // This code is available in examples/app_doc_properties.rs
    /// #
    /// # use rust_xlsxwriter::{Properties, Workbook, XlsxError};
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    ///
    ///     let properties = Properties::new()
    ///         .set_title("This is an example spreadsheet")
    ///         .set_subject("That demonstrates document properties")
    ///         .set_author("A. Rust User")
    ///         .set_manager("J. Alfred Prufrock")
    ///         .set_company("Rust Solutions Inc")
    ///         .set_category("Sample spreadsheets")
    ///         .set_keywords("Sample, Example, Properties")
    ///         .set_comment("Created with Rust and rust_xlsxwriter");
    ///
    ///     workbook.set_properties(&properties);
    ///
    /// #     let worksheet = workbook.add_worksheet();
    ///
    /// #     worksheet.set_column_width(0, 30)?;
    /// #     worksheet.write_string_only(0, 0, "See File -> Info -> Properties")?;
    /// #
    /// #     workbook.save("doc_properties.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/app_doc_properties.png">
    ///
    ///
    /// The document properties can also be used to set a constant creation date
    /// so that a file generated by a rust_xlsxwriter program will have the same
    /// checksum no matter when it is created.
    ///
    ///
    /// ```
    /// # // This code is available in examples/doc_properties_checksum2.rs
    /// #
    /// use chrono::{TimeZone, Utc};
    /// use rust_xlsxwriter::{Properties, Workbook, XlsxError};
    ///
    /// fn main() -> Result<(), XlsxError> {
    ///     let mut workbook = Workbook::new();
    ///
    ///     // Create a file creation date for the file.
    ///     let date = Utc.with_ymd_and_hms(2023, 1, 1, 0, 0, 0).unwrap();
    ///
    ///     // Add it to the document metadata.
    ///     let properties = Properties::new().set_creation_datetime(&date);
    ///     workbook.set_properties(&properties);
    ///
    ///     let worksheet = workbook.add_worksheet();
    ///     worksheet.write_string_only(0, 0, "Hello")?;
    ///
    ///     workbook.save("properties.xlsx")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    ///  See also [Checksum of a saved file].
    ///
    /// [Checksum of a saved file]:
    ///     https://rustxlsxwriter.github.io/workbook/checksum.html
    ///
    pub fn set_properties(&mut self, properties: &Properties) -> &mut Workbook {
        self.properties = properties.clone();
        self
    }

    /// Add a recommendation to open the file in “read-only” mode.
    ///
    /// This method can be used to set the Excel “Read-only Recommended” option
    /// that is available when saving a file. This presents the user of the file
    /// with an option to open it in "read-only" mode. This means that any
    /// changes to the file can’t be saved back to the same file and must be
    /// saved to a new file.
    ///
    /// # Examples
    ///
    /// The following example demonstrates creating a simple workbook which opens
    /// with a recommendation that the file should be opened in read only mode.
    ///
    /// ```
    /// # // This code is available in examples/doc_workbook_read_only_recommended.rs
    /// #
    /// # use rust_xlsxwriter::{Workbook, XlsxError};
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    ///     let mut workbook = Workbook::new();
    ///
    ///     let _worksheet = workbook.add_worksheet();
    ///
    ///     workbook.read_only_recommended();
    ///
    ///     workbook.save("workbook.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Alert when you open the output file:
    ///
    /// <img src="https://rustxlsxwriter.github.io/images/workbook_read_only_recommended.png">
    ///
    pub fn read_only_recommended(&mut self) -> &mut Workbook {
        self.read_only_mode = 2;
        self
    }

    // -----------------------------------------------------------------------
    // Internal function/methods.
    // -----------------------------------------------------------------------

    // Reset workbook between saves.
    fn reset(&mut self) {
        self.writer.reset();

        let default_format = Format::new();
        self.xf_indices = HashMap::from([(default_format.format_key(), 0)]);
        self.xf_formats = vec![default_format];
        self.font_count = 0;
        self.fill_count = 0;
        self.border_count = 0;
        self.num_format_count = 0;

        for worksheet in self.worksheets.iter_mut() {
            worksheet.reset();
        }
    }

    // Internal function to prepare the workbook and other component files for
    // writing to the xlsx file.
    fn save_internal<W: Write + Seek>(&mut self, writer: W) -> Result<(), XlsxError> {
        // Reset workbook and worksheet state data between saves.
        self.reset();

        // Ensure that there is at least one worksheet in the workbook.
        if self.worksheets.is_empty() {
            self.add_worksheet();
        }
        // Ensure one sheet is active/selected.
        self.set_active_worksheets();

        // Check for the use of hyperlink style in the worksheets and if so add
        // a hyperlink style to the global formats.
        for worksheet in self.worksheets.iter() {
            if worksheet.has_hyperlink_style {
                let format = Format::new().set_hyperlink();
                self.xf_indices.insert(format.format_key(), 1);
                self.xf_formats.push(format);
                self.has_hyperlink_style = true;
                break;
            }
        }

        // Convert any worksheet local formats to workbook/global formats.
        let mut worksheet_formats: Vec<Vec<Format>> = vec![];
        for worksheet in self.worksheets.iter() {
            let formats = worksheet.xf_formats.clone();
            worksheet_formats.push(formats);
        }

        let mut worksheet_indices: Vec<Vec<u32>> = vec![];
        for formats in &mut worksheet_formats {
            let mut indices = vec![];
            for format in formats {
                let index = self.format_index(format);
                indices.push(index);
            }
            worksheet_indices.push(indices);
        }

        for (i, worksheet) in self.worksheets.iter_mut().enumerate() {
            // Map worksheet/local format indices to the workbook/global values.
            worksheet.set_global_xf_indices(&worksheet_indices[i]);

            // Perform the autofilter row hiding.
            worksheet.hide_autofilter_rows();
        }

        // Convert the images in the workbooks into drawing files and rel links.
        self.prepare_drawings();

        // Prepare the formats for writing with styles.rs.
        self.prepare_format_properties();

        // Collect workbook level metadata to help generate the xlsx file.
        let mut package_options = PackagerOptions::new();
        package_options = self.set_package_options(package_options)?;

        // Create the Packager object that will assemble the zip/xlsx file.
        let mut packager = Packager::new(writer)?;
        packager.assemble_file(self, &package_options)?;

        Ok(())
    }

    // Iterates through the worksheets and find which is the user defined Active
    // sheet. If none has been set then default to the first sheet, like Excel.
    fn set_active_worksheets(&mut self) {
        let mut active_index = 0;

        for (i, worksheet) in self.worksheets.iter().enumerate() {
            if worksheet.active {
                active_index = i;
            }
            if worksheet.first_sheet {
                self.first_sheet = i as u16;
            }
        }
        self.worksheets[active_index].set_active(true);
        self.active_tab = active_index as u16;
    }

    // Convert the images in the workbooks into drawing files and rel links.
    fn prepare_drawings(&mut self) {
        let mut drawing_id = 1;
        let mut vml_drawing_id = 1;

        // These are the image ids for each unique image file.
        let mut worksheet_image_ids: HashMap<u64, u32> = HashMap::new();
        let mut header_footer_image_ids: HashMap<u64, u32> = HashMap::new();

        for worksheet in self.worksheets.iter_mut() {
            if !worksheet.images.is_empty() {
                worksheet.prepare_worksheet_images(&mut worksheet_image_ids, drawing_id);
                drawing_id += 1;
            }

            if worksheet.has_header_footer_images() {
                // The header/footer images are counted from the last worksheet id.
                let base_image_id = worksheet_image_ids.len() as u32;

                worksheet.prepare_header_footer_images(
                    &mut header_footer_image_ids,
                    base_image_id,
                    vml_drawing_id,
                );
                vml_drawing_id += 1;
            }
        }
    }

    // Evaluate and clone formats from worksheets into a workbook level vector
    // of unique formats. Also return the index for use in remapping worksheet
    // format indices.
    fn format_index(&mut self, format: &Format) -> u32 {
        let format_key = format.format_key();

        match self.xf_indices.get_mut(&format_key) {
            Some(xf_index) => *xf_index,
            None => {
                let xf_index = self.xf_formats.len() as u32;
                self.xf_formats.push(format.clone());
                self.xf_indices.insert(format_key, xf_index);
                xf_index
            }
        }
    }

    // Prepare all Format properties prior to passing them to styles.rs.
    fn prepare_format_properties(&mut self) {
        // Set the font index for the format objects.
        self.prepare_fonts();

        // Set the fill index for the format objects.
        self.prepare_fills();

        // Set the border index for the format objects.
        self.prepare_borders();

        // Set the number format index for the format objects.
        self.prepare_num_formats();
    }

    // Set the font index for the format objects.
    fn prepare_fonts(&mut self) {
        let mut font_count: u16 = 0;
        let mut font_indices: HashMap<String, u16> = HashMap::new();

        for xf_format in &mut self.xf_formats {
            let font_key = xf_format.font_key();

            match font_indices.get(&font_key) {
                Some(font_index) => {
                    xf_format.set_font_index(*font_index, false);
                }
                None => {
                    font_indices.insert(font_key, font_count);
                    xf_format.set_font_index(font_count, true);
                    font_count += 1;
                }
            }
        }
        self.font_count = font_count;
    }

    // Set the fill index for the format objects.
    fn prepare_fills(&mut self) {
        let mut fill_indices: HashMap<String, u16> = HashMap::new();

        // The user defined fill properties start from 2 since there are 2
        // default fills: patternType="none" and patternType="gray125". The
        // following code adds these 2 default fills.
        let mut fill_count: u16 = 2;

        let temp_format = Format::new();
        let mut fill_key = temp_format.fill_key();
        fill_indices.insert(fill_key, 0);
        fill_key = temp_format
            .set_pattern(crate::XlsxPattern::Gray125)
            .fill_key();
        fill_indices.insert(fill_key, 1);

        for xf_format in &mut self.xf_formats {
            // For a solid fill (pattern == "solid") Excel reverses the role of
            // foreground and background colors, and
            if xf_format.pattern == XlsxPattern::Solid
                && xf_format.background_color.is_not_default()
                && xf_format.foreground_color.is_not_default()
            {
                mem::swap(
                    &mut xf_format.foreground_color,
                    &mut xf_format.background_color,
                );
            }

            // If the user specifies a foreground or background color without a
            // pattern they probably wanted a solid fill, so we fill in the
            // defaults.
            if (xf_format.pattern == XlsxPattern::None || xf_format.pattern == XlsxPattern::Solid)
                && xf_format.background_color.is_not_default()
                && xf_format.foreground_color.is_default()
            {
                xf_format.foreground_color = xf_format.background_color;
                xf_format.background_color = XlsxColor::Automatic;
                xf_format.pattern = XlsxPattern::Solid;
            }

            if (xf_format.pattern == XlsxPattern::None || xf_format.pattern == XlsxPattern::Solid)
                && xf_format.background_color.is_default()
                && xf_format.foreground_color.is_not_default()
            {
                xf_format.background_color = XlsxColor::Automatic;
                xf_format.pattern = XlsxPattern::Solid;
            }

            // Get a unique fill identifier.
            let fill_key = xf_format.fill_key();

            // Find unique or repeated fill ids.
            match fill_indices.get(&fill_key) {
                Some(fill_index) => {
                    xf_format.set_fill_index(*fill_index, false);
                }
                None => {
                    fill_indices.insert(fill_key, fill_count);
                    xf_format.set_fill_index(fill_count, true);
                    fill_count += 1;
                }
            }
        }
        self.fill_count = fill_count;
    }

    // Set the border index for the format objects.
    fn prepare_borders(&mut self) {
        let mut border_count: u16 = 0;
        let mut border_indices: HashMap<String, u16> = HashMap::new();

        for xf_format in &mut self.xf_formats {
            let border_key = xf_format.border_key();

            match border_indices.get(&border_key) {
                Some(border_index) => {
                    xf_format.set_border_index(*border_index, false);
                }
                None => {
                    border_indices.insert(border_key, border_count);
                    xf_format.set_border_index(border_count, true);
                    border_count += 1;
                }
            }
        }
        self.border_count = border_count;
    }

    // Set the number format index for the format objects.
    fn prepare_num_formats(&mut self) {
        let mut num_formats: HashMap<String, u16> = HashMap::new();
        // User defined number formats in Excel start from index 164.
        let mut index = 164;

        for xf_format in &mut self.xf_formats {
            if xf_format.num_format_index > 0 {
                continue;
            }

            if xf_format.num_format.is_empty() {
                continue;
            }

            let num_format_string = xf_format.num_format.clone();

            match num_formats.get(&num_format_string) {
                Some(index) => {
                    xf_format.set_num_format_index_u16(*index);
                }
                None => {
                    num_formats.insert(num_format_string, index);
                    xf_format.set_num_format_index_u16(index);
                    index += 1;
                    self.num_format_count += 1;
                }
            }
        }
    }

    // Collect some workbook level metadata to help generate the xlsx
    // package/file.
    fn set_package_options(
        &mut self,
        mut package_options: PackagerOptions,
    ) -> Result<PackagerOptions, XlsxError> {
        package_options.num_worksheets = self.worksheets.len() as u16;
        package_options.doc_security = self.read_only_mode;

        let mut defined_names = self.user_defined_names.clone();
        let mut sheet_names: HashMap<String, u16> = HashMap::new();

        // Iterate over the worksheets to capture workbook and update the
        // package options metadata.
        for (sheet_index, worksheet) in self.worksheets.iter().enumerate() {
            let sheet_name = worksheet.name.clone();
            let quoted_sheet_name = utility::quote_sheetname(&sheet_name);
            sheet_names.insert(sheet_name.clone(), sheet_index as u16);

            // Check for duplicate sheet names, which aren't allowed by Excel.
            if package_options.worksheet_names.contains(&sheet_name) {
                return Err(XlsxError::SheetnameReused(sheet_name));
            }

            package_options.worksheet_names.push(sheet_name.clone());

            package_options.properties = self.properties.clone();

            if worksheet.uses_string_table {
                package_options.has_sst_table = true;
            }

            if worksheet.has_dynamic_arrays {
                package_options.has_dynamic_arrays = true;
            }

            if worksheet.has_header_footer_images() {
                package_options.has_vml = true;
            }

            if !worksheet.drawing.drawings.is_empty() {
                package_options.num_drawings += 1;
            }

            // Store the autofilter areas which are a category of defined name.
            if worksheet.autofilter_defined_name.in_use {
                let mut defined_name = worksheet.autofilter_defined_name.clone();
                defined_name.initialize(&quoted_sheet_name);
                defined_names.push(defined_name);
            }

            // Store any user defined print areas which are a category of defined name.
            if worksheet.print_area_defined_name.in_use {
                let mut defined_name = worksheet.print_area_defined_name.clone();
                defined_name.initialize(&quoted_sheet_name);
                defined_names.push(defined_name);
            }

            // Store any user defined print repeat rows/columns which are a
            // category of defined name.
            if worksheet.repeat_row_cols_defined_name.in_use {
                let mut defined_name = worksheet.repeat_row_cols_defined_name.clone();
                defined_name.initialize(&quoted_sheet_name);
                defined_names.push(defined_name);
            }

            // Set the used image types.
            for i in 0..NUM_IMAGE_FORMATS {
                if worksheet.image_types[i] {
                    package_options.image_types[i] = true;
                }
            }
        }

        // Map the sheet name and associated index so that we can map a sheet
        // reference in a Local/Sheet defined name to a worksheet index.
        for defined_name in defined_names.iter_mut() {
            let sheet_name = defined_name.unquoted_sheet_name();

            if !sheet_name.is_empty() {
                match sheet_names.get(&sheet_name) {
                    Some(index) => defined_name.index = *index,
                    None => {
                        let error = format!(
                            "Unknown worksheet name '{}' in defined name '{}'",
                            sheet_name, defined_name.name
                        );
                        return Err(XlsxError::ParameterError(error));
                    }
                }
            }
        }

        // Excel stores defined names in a sorted order.
        defined_names.sort_by_key(|n| (n.sort_name.clone(), n.range.clone()));

        // Map the non-Global defined names to App.xml entries.
        for defined_name in defined_names.iter() {
            let app_name = defined_name.app_name();
            if !app_name.is_empty() {
                package_options.defined_names.push(app_name);
            }
        }

        self.defined_names = defined_names;

        Ok(package_options)
    }

    // -----------------------------------------------------------------------
    // XML assembly methods.
    // -----------------------------------------------------------------------

    //  Assemble and write the XML file.
    pub(crate) fn assemble_xml_file(&mut self) {
        self.writer.xml_declaration();

        // Write the workbook element.
        self.write_workbook();

        // Write the fileVersion element.
        self.write_file_version();

        // Write the fileSharing element.
        if self.read_only_mode == 2 {
            self.write_file_sharing();
        }

        // Write the workbookPr element.
        self.write_workbook_pr();

        // Write the bookViews element.
        self.write_book_views();

        // Write the sheets element.
        self.write_sheets();

        // Write the definedNames element.
        if !self.defined_names.is_empty() {
            self.write_defined_names();
        }

        // Write the calcPr element.
        self.write_calc_pr();

        // Close the workbook tag.
        self.writer.xml_end_tag("workbook");
    }

    // Write the <workbook> element.
    fn write_workbook(&mut self) {
        let xmlns = "http://schemas.openxmlformats.org/spreadsheetml/2006/main".to_string();
        let xmlns_r =
            "http://schemas.openxmlformats.org/officeDocument/2006/relationships".to_string();

        let attributes = vec![("xmlns", xmlns), ("xmlns:r", xmlns_r)];

        self.writer.xml_start_tag_attr("workbook", &attributes);
    }

    // Write the <fileVersion> element.
    fn write_file_version(&mut self) {
        let attributes = vec![
            ("appName", "xl".to_string()),
            ("lastEdited", "4".to_string()),
            ("lowestEdited", "4".to_string()),
            ("rupBuild", "4505".to_string()),
        ];

        self.writer.xml_empty_tag_attr("fileVersion", &attributes);
    }

    // Write the <fileSharing> element.
    fn write_file_sharing(&mut self) {
        let attributes = vec![("readOnlyRecommended", "1".to_string())];

        self.writer.xml_empty_tag_attr("fileSharing", &attributes);
    }

    // Write the <workbookPr> element.
    fn write_workbook_pr(&mut self) {
        let attributes = vec![("defaultThemeVersion", "124226".to_string())];

        self.writer.xml_empty_tag_attr("workbookPr", &attributes);
    }

    // Write the <bookViews> element.
    fn write_book_views(&mut self) {
        self.writer.xml_start_tag("bookViews");

        // Write the workbookView element.
        self.write_workbook_view();

        self.writer.xml_end_tag("bookViews");
    }

    // Write the <workbookView> element.
    fn write_workbook_view(&mut self) {
        let mut attributes = vec![
            ("xWindow", "240".to_string()),
            ("yWindow", "15".to_string()),
            ("windowWidth", "16095".to_string()),
            ("windowHeight", "9660".to_string()),
        ];

        // Store the firstSheet attribute when it isn't the first sheet.
        if self.first_sheet > 0 {
            let first_sheet = self.first_sheet + 1;
            attributes.push(("firstSheet", first_sheet.to_string()));
        }

        // Store the activeTab attribute when it isn't the first sheet.
        if self.active_tab > 0 {
            attributes.push(("activeTab", self.active_tab.to_string()));
        }

        self.writer.xml_empty_tag_attr("workbookView", &attributes);
    }

    // Write the <sheets> element.
    fn write_sheets(&mut self) {
        self.writer.xml_start_tag("sheets");

        let mut worksheet_data = vec![];
        for worksheet in self.worksheets.iter() {
            worksheet_data.push((worksheet.name.clone(), worksheet.hidden));
        }

        for (index, data) in worksheet_data.iter().enumerate() {
            // Write the sheet element.
            self.write_sheet(&data.0, data.1, (index + 1) as u16);
        }

        self.writer.xml_end_tag("sheets");
    }

    // Write the <sheet> element.
    fn write_sheet(&mut self, name: &str, is_hidden: bool, index: u16) {
        let sheet_id = format!("{index}");
        let ref_id = format!("rId{index}");

        let mut attributes = vec![("name", name.to_string()), ("sheetId", sheet_id)];

        if is_hidden {
            attributes.push(("state", "hidden".to_string()));
        }

        attributes.push(("r:id", ref_id));

        self.writer.xml_empty_tag_attr("sheet", &attributes);
    }

    // Write the <definedNames> element.
    fn write_defined_names(&mut self) {
        self.writer.xml_start_tag("definedNames");

        for defined_name in self.defined_names.iter() {
            let mut attributes = vec![("name", defined_name.name())];

            match defined_name.name_type {
                DefinedNameType::Global => {}
                _ => {
                    attributes.push(("localSheetId", defined_name.index.to_string()));
                }
            }

            if let DefinedNameType::Autofilter = defined_name.name_type {
                attributes.push(("hidden", "1".to_string()));
            }

            self.writer
                .xml_data_element_attr("definedName", &defined_name.range, &attributes);
        }

        self.writer.xml_end_tag("definedNames");
    }

    // Write the <calcPr> element.
    fn write_calc_pr(&mut self) {
        let attributes = vec![
            ("calcId", "124519".to_string()),
            ("fullCalcOnLoad", "1".to_string()),
        ];

        self.writer.xml_empty_tag_attr("calcPr", &attributes);
    }
}

// -----------------------------------------------------------------------
// Helper enums/structs/functions.
// -----------------------------------------------------------------------

// -----------------------------------------------------------------------
// Tests.
// -----------------------------------------------------------------------
#[cfg(test)]
mod tests {

    use super::Workbook;
    use crate::{test_functions::xml_to_vec, XlsxError};
    use pretty_assertions::assert_eq;

    #[test]
    fn test_assemble() {
        let mut workbook = Workbook::default();
        workbook.add_worksheet();

        workbook.assemble_xml_file();

        let got = workbook.writer.read_to_str();
        let got = xml_to_vec(&got);

        let expected = xml_to_vec(
            r#"
            <?xml version="1.0" encoding="UTF-8" standalone="yes"?>
            <workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
              <fileVersion appName="xl" lastEdited="4" lowestEdited="4" rupBuild="4505"/>
              <workbookPr defaultThemeVersion="124226"/>
              <bookViews>
                <workbookView xWindow="240" yWindow="15" windowWidth="16095" windowHeight="9660"/>
              </bookViews>
              <sheets>
                <sheet name="Sheet1" sheetId="1" r:id="rId1"/>
              </sheets>
              <calcPr calcId="124519" fullCalcOnLoad="1"/>
            </workbook>
            "#,
        );

        assert_eq!(got, expected);
    }

    #[test]
    fn define_name() {
        let mut workbook = Workbook::default();

        // Test invalid defined names.
        let names = vec![
            ".foo",    // Invalid start character.
            "foo bar", // Space in name
            "Foo,",    // Other invalid characters.
            "Foo/", "Foo[", "Foo]", "Foo'", "Foo\"bar", "Foo:", "Foo*",
        ];

        for name in names {
            let result = workbook.define_name(name, "");
            assert!(matches!(result, Err(XlsxError::ParameterError(_))));
        }
    }
}
