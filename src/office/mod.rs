pub mod excel;
/// Office module for managing office-related applications and documents
pub mod powerpoint;
pub mod word;

// Re-export specific items instead of using glob imports
// PowerPoint module
pub use powerpoint::{
    BulletPoint, ImageType, PowerPointClient, Presentation, PresentationTheme, Slide, SlideLayout,
};

// Word module
pub use word::{Alignment, Document, Paragraph, Section, Table, TableCell, WordClient};

// Excel module
pub use excel::{
    Cell, CellFormat, CellValue, Chart, ChartType, Column, ExcelClient, Row, Workbook, Worksheet,
};

// Re-export with namespaced structs to avoid ambiguity
pub use powerpoint::Image as PowerPointImage;
pub use powerpoint::TextFormatting as PowerPointTextFormatting;
pub use word::Image as WordImage;
pub use word::TextFormatting as WordTextFormatting;
