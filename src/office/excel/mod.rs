use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::tools::{ToolAnnotation, ToolDefinition};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Cell format options for Excel spreadsheets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellFormat {
    /// Font name (e.g., 'Calibri', 'Arial')
    pub font_name: Option<String>,
    /// Font size in points (e.g., 12, 18, 24)
    pub font_size: Option<u32>,
    /// Bold text
    pub bold: Option<bool>,
    /// Italic text
    pub italic: Option<bool>,
    /// Underline text
    pub underline: Option<bool>,
    /// Text color in hex (e.g., '#000000' for black)
    pub color: Option<String>,
    /// Background color in hex
    pub background_color: Option<String>,
    /// Number format (e.g., "#,##0.00", "mm/dd/yyyy")
    pub number_format: Option<String>,
    /// Text alignment (left, center, right)
    pub alignment: Option<String>,
}

/// Cell value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum CellValue {
    /// String value
    Text(String),
    /// Numeric value
    Number(f64),
    /// Boolean value
    Boolean(bool),
    /// Date value (ISO-8601 format)
    Date(String),
    /// Formula
    Formula(String),
    /// Empty cell
    Empty,
}

/// Cell model for spreadsheets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cell {
    /// Cell value
    pub value: CellValue,
    /// Cell formatting
    pub format: Option<CellFormat>,
}

/// Row model for spreadsheets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Row {
    /// Row index (0-based)
    pub index: u32,
    /// Row cells
    pub cells: Vec<Cell>,
    /// Row height
    pub height: Option<f64>,
}

/// Column model for spreadsheets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Column {
    /// Column index (0-based)
    pub index: u32,
    /// Column width
    pub width: Option<f64>,
    /// Column format
    pub format: Option<CellFormat>,
}

/// Chart types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    /// Bar chart
    Bar,
    /// Column chart
    Column,
    /// Line chart
    Line,
    /// Pie chart
    Pie,
    /// Area chart
    Area,
    /// Scatter chart
    Scatter,
}

/// Chart model for spreadsheets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chart {
    /// Chart type
    pub chart_type: ChartType,
    /// Chart title
    pub title: String,
    /// Data range (e.g., "A1:B10")
    pub data_range: String,
    /// Whether to include a legend
    pub has_legend: Option<bool>,
}

/// Worksheet model for Excel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worksheet {
    /// Worksheet name
    pub name: String,
    /// Worksheet rows
    pub rows: Vec<Row>,
    /// Worksheet columns
    pub columns: Option<Vec<Column>>,
    /// Worksheet charts
    pub charts: Option<Vec<Chart>>,
}

/// Workbook model for Excel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workbook {
    /// Workbook title
    pub title: String,
    /// Workbook author
    pub author: Option<String>,
    /// Workbook worksheets
    pub worksheets: Vec<Worksheet>,
}

/// Excel client for managing spreadsheets
pub struct ExcelClient<'a> {
    /// Lifecycle manager
    lifecycle: &'a LifecycleManager,
}

impl<'a> ExcelClient<'a> {
    /// Create a new Excel client
    pub fn new(lifecycle: &'a LifecycleManager) -> Self {
        Self { lifecycle }
    }

    /// Create a new workbook
    pub async fn create_workbook(&self, workbook: Workbook) -> Result<String> {
        let method = "tools/execute";
        let params = json!({
            "name": "create_workbook",
            "args": {
                "title": workbook.title,
                "author": workbook.author,
            }
        });

        let response = self.lifecycle.call_method(method, Some(params)).await?;
        let workbook_id = response["workbook_id"]
            .as_str()
            .ok_or_else(|| Error::parsing("Failed to parse workbook ID"))?
            .to_string();

        // Add worksheets
        for worksheet in workbook.worksheets {
            self.add_worksheet(&workbook_id, worksheet).await?;
        }

        Ok(workbook_id)
    }

    /// Add a worksheet to an existing workbook
    pub async fn add_worksheet(&self, workbook_id: &str, worksheet: Worksheet) -> Result<String> {
        let method = "tools/execute";
        let mut args = json!({
            "workbook_id": workbook_id,
            "name": worksheet.name,
        });

        if !worksheet.rows.is_empty() {
            args["rows"] = json!(worksheet.rows);
        }

        if let Some(columns) = worksheet.columns {
            if !columns.is_empty() {
                args["columns"] = json!(columns);
            }
        }

        if let Some(charts) = worksheet.charts {
            if !charts.is_empty() {
                args["charts"] = json!(charts);
            }
        }

        let params = json!({
            "name": "add_worksheet",
            "args": args
        });

        let response = self.lifecycle.call_method(method, Some(params)).await?;
        let worksheet_id = response["worksheet_id"]
            .as_str()
            .ok_or_else(|| Error::parsing("Failed to parse worksheet ID"))?
            .to_string();

        Ok(worksheet_id)
    }

    /// Delete a worksheet
    pub async fn delete_worksheet(&self, workbook_id: &str, worksheet_id: &str) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "delete_worksheet",
            "args": {
                "workbook_id": workbook_id,
                "worksheet_id": worksheet_id
            }
        });

        self.lifecycle.call_method(method, Some(params)).await?;
        Ok(())
    }

    /// Get all worksheets in a workbook
    pub async fn get_worksheets(&self, workbook_id: &str) -> Result<Vec<Worksheet>> {
        let method = "tools/execute";
        let params = json!({
            "name": "get_worksheets",
            "args": {
                "workbook_id": workbook_id
            }
        });

        let response = self.lifecycle.call_method(method, Some(params)).await?;
        let worksheets = serde_json::from_value(response["worksheets"].clone())
            .map_err(|e| Error::parsing(format!("Failed to parse worksheets: {}", e)))?;

        Ok(worksheets)
    }

    /// Update cell values
    pub async fn update_cells(
        &self,
        workbook_id: &str,
        worksheet_id: &str,
        cells: Vec<(String, CellValue)>,
    ) -> Result<()> {
        let method = "tools/execute";
        let cell_updates = cells
            .iter()
            .map(|(cell_ref, value)| {
                json!({
                    "cell_ref": cell_ref,
                    "value": value
                })
            })
            .collect::<Vec<Value>>();

        let params = json!({
            "name": "update_cells",
            "args": {
                "workbook_id": workbook_id,
                "worksheet_id": worksheet_id,
                "cells": cell_updates
            }
        });

        self.lifecycle.call_method(method, Some(params)).await?;
        Ok(())
    }

    /// Add a row to a worksheet
    pub async fn add_row(&self, workbook_id: &str, worksheet_id: &str, row: Row) -> Result<u32> {
        let method = "tools/execute";
        let params = json!({
            "name": "add_row",
            "args": {
                "workbook_id": workbook_id,
                "worksheet_id": worksheet_id,
                "row": row
            }
        });

        let response = self.lifecycle.call_method(method, Some(params)).await?;
        let row_index = response["row_index"]
            .as_u64()
            .ok_or_else(|| Error::parsing("Failed to parse row index"))?;

        Ok(row_index as u32)
    }

    /// Delete a row from a worksheet
    pub async fn delete_row(
        &self,
        workbook_id: &str,
        worksheet_id: &str,
        row_index: u32,
    ) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "delete_row",
            "args": {
                "workbook_id": workbook_id,
                "worksheet_id": worksheet_id,
                "row_index": row_index
            }
        });

        self.lifecycle.call_method(method, Some(params)).await?;
        Ok(())
    }

    /// Add a chart to a worksheet
    pub async fn add_chart(
        &self,
        workbook_id: &str,
        worksheet_id: &str,
        chart: Chart,
    ) -> Result<String> {
        let method = "tools/execute";
        let params = json!({
            "name": "add_chart",
            "args": {
                "workbook_id": workbook_id,
                "worksheet_id": worksheet_id,
                "chart": chart
            }
        });

        let response = self.lifecycle.call_method(method, Some(params)).await?;
        let chart_id = response["chart_id"]
            .as_str()
            .ok_or_else(|| Error::parsing("Failed to parse chart ID"))?
            .to_string();

        Ok(chart_id)
    }

    /// Apply formula to a range of cells
    pub async fn apply_formula(
        &self,
        workbook_id: &str,
        worksheet_id: &str,
        cell_range: &str,
        formula: &str,
    ) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "apply_formula",
            "args": {
                "workbook_id": workbook_id,
                "worksheet_id": worksheet_id,
                "cell_range": cell_range,
                "formula": formula
            }
        });

        self.lifecycle.call_method(method, Some(params)).await?;
        Ok(())
    }

    /// Format a range of cells
    pub async fn format_cells(
        &self,
        workbook_id: &str,
        worksheet_id: &str,
        cell_range: &str,
        format: CellFormat,
    ) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "format_cells",
            "args": {
                "workbook_id": workbook_id,
                "worksheet_id": worksheet_id,
                "cell_range": cell_range,
                "format": format
            }
        });

        self.lifecycle.call_method(method, Some(params)).await?;
        Ok(())
    }

    /// Save the workbook to a file
    pub async fn save_workbook(&self, workbook_id: &str, filepath: &str) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "save_workbook",
            "args": {
                "workbook_id": workbook_id,
                "filepath": filepath
            }
        });

        self.lifecycle.call_method(method, Some(params)).await?;
        Ok(())
    }

    /// Load a workbook from a file
    pub async fn load_workbook(&self, filepath: &str) -> Result<String> {
        let method = "tools/execute";
        let params = json!({
            "name": "load_workbook",
            "args": {
                "filepath": filepath
            }
        });

        let response = self.lifecycle.call_method(method, Some(params)).await?;
        let workbook_id = response["workbook_id"]
            .as_str()
            .ok_or_else(|| Error::parsing("Failed to parse workbook ID"))?
            .to_string();

        Ok(workbook_id)
    }

    /// Generate a spreadsheet from data using AI
    pub async fn generate_spreadsheet(&self, data_description: &str) -> Result<String> {
        let method = "tools/execute";
        let params = json!({
            "name": "generate_spreadsheet",
            "args": {
                "data_description": data_description
            }
        });

        let response = self.lifecycle.call_method(method, Some(params)).await?;
        let workbook_id = response["workbook_id"]
            .as_str()
            .ok_or_else(|| Error::parsing("Failed to parse workbook ID"))?
            .to_string();

        Ok(workbook_id)
    }

    /// Get available tools
    pub fn get_tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition::from_json_schema(
                "create_workbook",
                "Create a new Excel workbook",
                "spreadsheet",
                json!({
                    "type": "object",
                    "properties": {
                        "title": {"type": "string", "description": "Title of the workbook"},
                        "author": {"type": "string", "description": "Author of the workbook (optional)"}
                    },
                    "required": ["title"]
                }),
                Some(
                    ToolAnnotation::new("spreadsheet_creator")
                        .with_description("Creates a new workbook"),
                ),
            ),
            ToolDefinition::from_json_schema(
                "add_worksheet",
                "Add a worksheet to a workbook",
                "spreadsheet",
                json!({
                    "type": "object",
                    "properties": {
                        "workbook_id": {"type": "string", "description": "ID of the workbook"},
                        "name": {"type": "string", "description": "Name of the worksheet"}
                    },
                    "required": ["workbook_id", "name"]
                }),
                Some(
                    ToolAnnotation::new("worksheet_manager")
                        .with_description("Adds a worksheet to a workbook"),
                ),
            ),
            ToolDefinition::from_json_schema(
                "delete_worksheet",
                "Delete a worksheet from a workbook",
                "spreadsheet",
                json!({
                    "type": "object",
                    "properties": {
                        "workbook_id": {"type": "string", "description": "ID of the workbook"},
                        "worksheet_id": {"type": "string", "description": "ID of the worksheet to delete"}
                    },
                    "required": ["workbook_id", "worksheet_id"]
                }),
                Some(
                    ToolAnnotation::new("worksheet_manager")
                        .with_description("Deletes a worksheet"),
                ),
            ),
            ToolDefinition::from_json_schema(
                "get_worksheets",
                "Get all worksheets in a workbook",
                "spreadsheet",
                json!({
                    "type": "object",
                    "properties": {
                        "workbook_id": {"type": "string", "description": "ID of the workbook"}
                    },
                    "required": ["workbook_id"]
                }),
                Some(
                    ToolAnnotation::new("worksheet_manager")
                        .with_description("Gets all worksheets in a workbook"),
                ),
            ),
            ToolDefinition::from_json_schema(
                "update_cells",
                "Update cell values in a worksheet",
                "spreadsheet",
                json!({
                    "type": "object",
                    "properties": {
                        "workbook_id": {"type": "string", "description": "ID of the workbook"},
                        "worksheet_id": {"type": "string", "description": "ID of the worksheet"},
                        "cells": {
                            "type": "array",
                            "description": "Cell updates",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "cell_ref": {"type": "string", "description": "Cell reference (e.g. 'A1')"},
                                    "value": {"description": "Cell value"}
                                },
                                "required": ["cell_ref", "value"]
                            }
                        }
                    },
                    "required": ["workbook_id", "worksheet_id", "cells"]
                }),
                Some(ToolAnnotation::new("cell_manager").with_description("Updates cell values")),
            ),
            ToolDefinition::from_json_schema(
                "add_row",
                "Add a row to a worksheet",
                "spreadsheet",
                json!({
                    "type": "object",
                    "properties": {
                        "workbook_id": {"type": "string", "description": "ID of the workbook"},
                        "worksheet_id": {"type": "string", "description": "ID of the worksheet"},
                        "cells": {"type": "array", "description": "Row cells", "items": {"description": "Cell value"}}
                    },
                    "required": ["workbook_id", "worksheet_id", "cells"]
                }),
                Some(
                    ToolAnnotation::new("row_manager")
                        .with_description("Adds a row to a worksheet"),
                ),
            ),
            ToolDefinition::from_json_schema(
                "delete_row",
                "Delete a row from a worksheet",
                "spreadsheet",
                json!({
                    "type": "object",
                    "properties": {
                        "workbook_id": {"type": "string", "description": "ID of the workbook"},
                        "worksheet_id": {"type": "string", "description": "ID of the worksheet"},
                        "row_index": {"type": "integer", "description": "Index of the row to delete (0-based)"}
                    },
                    "required": ["workbook_id", "worksheet_id", "row_index"]
                }),
                Some(
                    ToolAnnotation::new("row_manager")
                        .with_description("Deletes a row from a worksheet"),
                ),
            ),
            ToolDefinition::from_json_schema(
                "add_chart",
                "Add a chart to a worksheet",
                "spreadsheet",
                json!({
                    "type": "object",
                    "properties": {
                        "workbook_id": {"type": "string", "description": "ID of the workbook"},
                        "worksheet_id": {"type": "string", "description": "ID of the worksheet"},
                        "chart_type": {"type": "string", "description": "Chart type (bar, column, line, pie, area, scatter)"},
                        "title": {"type": "string", "description": "Chart title"},
                        "data_range": {"type": "string", "description": "Data range (e.g. 'A1:B10')"}
                    },
                    "required": ["workbook_id", "worksheet_id", "chart_type", "title", "data_range"]
                }),
                Some(
                    ToolAnnotation::new("chart_manager")
                        .with_description("Adds a chart to a worksheet"),
                ),
            ),
            ToolDefinition::from_json_schema(
                "apply_formula",
                "Apply a formula to a range of cells",
                "spreadsheet",
                json!({
                    "type": "object",
                    "properties": {
                        "workbook_id": {"type": "string", "description": "ID of the workbook"},
                        "worksheet_id": {"type": "string", "description": "ID of the worksheet"},
                        "cell_range": {"type": "string", "description": "Cell range (e.g. 'A1:B10')"},
                        "formula": {"type": "string", "description": "Formula to apply"}
                    },
                    "required": ["workbook_id", "worksheet_id", "cell_range", "formula"]
                }),
                Some(
                    ToolAnnotation::new("formula_manager")
                        .with_description("Applies a formula to cells"),
                ),
            ),
            ToolDefinition::from_json_schema(
                "format_cells",
                "Format a range of cells",
                "spreadsheet",
                json!({
                    "type": "object",
                    "properties": {
                        "workbook_id": {"type": "string", "description": "ID of the workbook"},
                        "worksheet_id": {"type": "string", "description": "ID of the worksheet"},
                        "cell_range": {"type": "string", "description": "Cell range (e.g. 'A1:B10')"},
                        "format": {"type": "object", "description": "Cell formatting"}
                    },
                    "required": ["workbook_id", "worksheet_id", "cell_range", "format"]
                }),
                Some(ToolAnnotation::new("formatting_manager").with_description("Formats cells")),
            ),
            ToolDefinition::from_json_schema(
                "save_workbook",
                "Save a workbook to a file",
                "spreadsheet",
                json!({
                    "type": "object",
                    "properties": {
                        "workbook_id": {"type": "string", "description": "ID of the workbook"},
                        "filepath": {"type": "string", "description": "Path to save the workbook"}
                    },
                    "required": ["workbook_id", "filepath"]
                }),
                Some(
                    ToolAnnotation::new("workbook_manager")
                        .with_description("Saves a workbook to a file"),
                ),
            ),
            ToolDefinition::from_json_schema(
                "load_workbook",
                "Load a workbook from a file",
                "spreadsheet",
                json!({
                    "type": "object",
                    "properties": {
                        "filepath": {"type": "string", "description": "Path to load the workbook from"}
                    },
                    "required": ["filepath"]
                }),
                Some(
                    ToolAnnotation::new("workbook_manager")
                        .with_description("Loads a workbook from a file"),
                ),
            ),
            ToolDefinition::from_json_schema(
                "generate_spreadsheet",
                "Generate a spreadsheet using AI",
                "spreadsheet",
                json!({
                    "type": "object",
                    "properties": {
                        "data_description": {"type": "string", "description": "Description of the data to generate"}
                    },
                    "required": ["data_description"]
                }),
                Some(
                    ToolAnnotation::new("spreadsheet_creator")
                        .with_description("Generates a spreadsheet using AI"),
                ),
            ),
        ]
    }
}
