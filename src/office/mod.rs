/// Office module for managing office-related applications and documents
pub mod powerpoint;
pub mod word;
pub mod excel;

// Re-export specific items instead of using glob imports
// PowerPoint module
pub use powerpoint::{
    PowerPointClient, Presentation, Slide, SlideLayout, PresentationTheme,
    BulletPoint, ImageType,
};

// Word module
pub use word::{
    WordClient, Document, Section, Paragraph, Alignment, Table, TableCell,
};

// Excel module
pub use excel::{
    ExcelClient, Workbook, Worksheet, Row, Column, Cell, CellValue, CellFormat, Chart, ChartType,
};

// Re-export with namespaced structs to avoid ambiguity
pub use powerpoint::TextFormatting as PowerPointTextFormatting;
pub use powerpoint::Image as PowerPointImage;
pub use word::TextFormatting as WordTextFormatting;
pub use word::Image as WordImage; 