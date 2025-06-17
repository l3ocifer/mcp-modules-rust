use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::tools::{ToolDefinition, ToolAnnotation};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Text formatting options for Word documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFormatting {
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
}

/// Paragraph alignment
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    /// Left alignment
    Left,
    /// Center alignment
    Center,
    /// Right alignment
    Right,
    /// Justified alignment
    Justified,
}

/// Paragraph model for Word documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paragraph {
    /// Text content of the paragraph
    pub text: String,
    /// Text formatting for this paragraph
    pub formatting: Option<TextFormatting>,
    /// Paragraph alignment
    pub alignment: Option<Alignment>,
    /// Whether this paragraph is a heading
    pub is_heading: Option<bool>,
    /// Heading level (1-6, if is_heading is true)
    pub heading_level: Option<u8>,
}

/// Table cell model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableCell {
    /// Content of the cell
    pub content: String,
    /// Text formatting for this cell
    pub formatting: Option<TextFormatting>,
}

/// Table model for Word documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Table {
    /// Table header row
    pub header: Option<Vec<String>>,
    /// Table rows
    pub rows: Vec<Vec<TableCell>>,
    /// Table caption
    pub caption: Option<String>,
}

/// Image content for documents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    /// Base64-encoded image data
    pub data: String,
    /// Image type (JPEG, PNG, SVG)
    pub image_type: String,
    /// Alternative text for accessibility
    pub alt_text: Option<String>,
    /// Width in pixels
    pub width: Option<u32>,
    /// Height in pixels
    pub height: Option<u32>,
    /// Caption for the image
    pub caption: Option<String>,
}

/// Document section model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    /// Section title
    pub title: Option<String>,
    /// Section paragraphs
    pub paragraphs: Vec<Paragraph>,
    /// Section tables
    pub tables: Option<Vec<Table>>,
    /// Section images
    pub images: Option<Vec<Image>>,
}

/// Document model for Word
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Document title
    pub title: String,
    /// Document author
    pub author: Option<String>,
    /// Document sections
    pub sections: Vec<Section>,
}

/// Word client for managing documents
pub struct WordClient<'a> {
    /// Lifecycle manager
    lifecycle: &'a LifecycleManager,
}

impl<'a> WordClient<'a> {
    /// Create a new Word client
    pub fn new(lifecycle: &'a LifecycleManager) -> Self {
        Self { lifecycle }
    }

    /// Create a new document
    pub async fn create_document(&self, document: Document) -> Result<String> {
        let method = "tools/execute";
        let params = json!({
            "name": "create_document",
            "args": {
                "title": document.title,
                "author": document.author,
            }
        });

        let response = self.lifecycle.send_request(method, Some(params)).await?;
        let document_id = response["document_id"]
            .as_str()
            .ok_or_else(|| Error::parsing("Failed to parse document ID"))?
            .to_string();

        // Create sections
        for section in document.sections {
            let section_id = self.add_section(&document_id, section.clone()).await?;
            
            // Add paragraphs
            for paragraph in &section.paragraphs {
                self.add_paragraph(&document_id, section_id, paragraph.clone()).await?;
            }
            
            // Add tables if present
            if let Some(tables) = &section.tables {
                for table in tables {
                    self.add_table(&document_id, section_id, table.clone()).await?;
                }
            }
            
            // Add images if present
            if let Some(images) = &section.images {
                for image in images {
                    self.add_image(&document_id, section_id, image.clone()).await?;
                }
            }
        }

        Ok(document_id)
    }

    /// Add a section to an existing document
    pub async fn add_section(&self, document_id: &str, section: Section) -> Result<u32> {
        let method = "tools/execute";
        let mut args = json!({
            "document_id": document_id,
        });

        if let Some(title) = &section.title {
            args["title"] = json!(title);
        }

        let params = json!({
            "name": "add_section",
            "args": args
        });

        let response = self.lifecycle.send_request(method, Some(params)).await?;
        let section_id = response["section_id"]
            .as_u64()
            .ok_or_else(|| Error::parsing("Failed to parse section ID"))?;

        // Add content
        for paragraph in section.paragraphs {
            self.add_paragraph(document_id, section_id as u32, paragraph).await?;
        }

        // Add tables if present
        if let Some(tables) = section.tables {
            for table in tables {
                self.add_table(document_id, section_id as u32, table).await?;
            }
        }

        // Add images if present
        if let Some(images) = section.images {
            for image in images {
                self.add_image(document_id, section_id as u32, image).await?;
            }
        }

        Ok(section_id as u32)
    }

    /// Update an existing section
    pub async fn update_section(&self, document_id: &str, section_id: u32, title: Option<String>) -> Result<()> {
        let method = "tools/execute";
        let mut args = json!({
            "document_id": document_id,
            "section_id": section_id,
        });

        if let Some(title) = title {
            args["title"] = json!(title);
        }

        let params = json!({
            "name": "update_section",
            "args": args
        });

        self.lifecycle.send_request(method, Some(params)).await?;
        Ok(())
    }

    /// Delete a section
    pub async fn delete_section(&self, document_id: &str, section_id: u32) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "delete_section",
            "args": {
                "document_id": document_id,
                "section_id": section_id
            }
        });

        self.lifecycle.send_request(method, Some(params)).await?;
        Ok(())
    }

    /// Get all sections in a document
    pub async fn get_sections(&self, document_id: &str) -> Result<Vec<Section>> {
        let method = "tools/execute";
        let params = json!({
            "name": "get_sections",
            "args": {
                "document_id": document_id
            }
        });

        let response = self.lifecycle.send_request(method, Some(params)).await?;
        let sections = serde_json::from_value(response["sections"].clone())
            .map_err(|e| Error::parsing(format!("Failed to parse sections: {}", e)))?;

        Ok(sections)
    }

    /// Add a paragraph to a section
    pub async fn add_paragraph(&self, document_id: &str, section_id: u32, paragraph: Paragraph) -> Result<u32> {
        let method = "tools/execute";
        let params = json!({
            "name": "add_paragraph",
            "args": {
                "document_id": document_id,
                "section_id": section_id,
                "text": paragraph.text,
                "formatting": paragraph.formatting
            }
        });

        let response = self.lifecycle.send_request(method, Some(params)).await?;
        let paragraph_id = response["paragraph_id"]
            .as_u64()
            .ok_or_else(|| Error::parsing("Failed to parse paragraph ID"))?;

        Ok(paragraph_id as u32)
    }

    /// Add a table to a section
    pub async fn add_table(&self, document_id: &str, section_id: u32, table: Table) -> Result<u32> {
        let method = "tools/execute";
        let params = json!({
            "name": "add_table",
            "args": {
                "document_id": document_id,
                "section_id": section_id,
                "header": table.header,
                "rows": table.rows,
                "caption": table.caption
            }
        });

        let response = self.lifecycle.send_request(method, Some(params)).await?;
        let table_id = response["table_id"]
            .as_u64()
            .ok_or_else(|| Error::parsing("Failed to parse table ID"))?;

        Ok(table_id as u32)
    }

    /// Add an image to a section
    pub async fn add_image(&self, document_id: &str, section_id: u32, image: Image) -> Result<u32> {
        let method = "tools/execute";
        let mut args = json!({
            "document_id": document_id,
            "section_id": section_id,
            "image_data": image.data,
        });

        if let Some(caption) = image.caption {
            args["caption"] = json!(caption);
        }

        let params = json!({
            "name": "add_image",
            "args": args
        });

        let response = self.lifecycle.send_request(method, Some(params)).await?;
        let image_id = response["image_id"]
            .as_u64()
            .ok_or_else(|| Error::parsing("Failed to parse image ID"))?;

        Ok(image_id as u32)
    }

    /// Save the document to a file
    pub async fn save_document(&self, document_id: &str, filepath: &str) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "save_document",
            "args": {
                "document_id": document_id,
                "filepath": filepath
            }
        });

        self.lifecycle.send_request(method, Some(params)).await?;
        Ok(())
    }

    /// Load a document from a file
    pub async fn load_document(&self, filepath: &str) -> Result<String> {
        let method = "tools/execute";
        let params = json!({
            "name": "load_document",
            "args": {
                "filepath": filepath
            }
        });

        let response = self.lifecycle.send_request(method, Some(params)).await?;
        let document_id = response["document_id"]
            .as_str()
            .ok_or_else(|| Error::parsing("Failed to parse document ID"))?
            .to_string();

        Ok(document_id)
    }

    /// Generate a document from a template using AI
    pub async fn generate_document(&self, topic: &str, length: u32) -> Result<String> {
        let method = "tools/execute";
        let params = json!({
            "name": "generate_document",
            "args": {
                "topic": topic,
                "length": length
            }
        });

        let response = self.lifecycle.send_request(method, Some(params)).await?;
        let document_id = response["document_id"]
            .as_str()
            .ok_or_else(|| Error::parsing("Failed to parse document ID"))?
            .to_string();

        Ok(document_id)
    }

    /// Get available tools
    pub fn get_tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition::from_json_schema(
                "create_document",
                "Create a new Word document",
                "document",
                json!({
                    "type": "object",
                    "properties": {
                        "title": {"type": "string", "description": "Title of the document"},
                        "author": {"type": "string", "description": "Author of the document (optional)"}
                    },
                    "required": ["title"]
                }),
                Some(ToolAnnotation::new("document_creator", "Creates a new document"))
            ),
            ToolDefinition::from_json_schema(
                "add_section",
                "Add a section to a document",
                "document",
                json!({
                    "type": "object",
                    "properties": {
                        "document_id": {"type": "string", "description": "ID of the document"},
                        "title": {"type": "string", "description": "Section title (optional)"}
                    },
                    "required": ["document_id"]
                }),
                Some(ToolAnnotation::new("section_manager", "Adds a section to a document")),
            ),
            ToolDefinition::from_json_schema(
                "update_section",
                "Update an existing section",
                "document",
                json!({
                    "type": "object",
                    "properties": {
                        "document_id": {"type": "string", "description": "ID of the document"},
                        "section_id": {"type": "integer", "description": "ID of the section to update"},
                        "title": {"type": "string", "description": "New section title (optional)"}
                    },
                    "required": ["document_id", "section_id"]
                }),
                Some(ToolAnnotation::new("section_manager", "Updates an existing section")),
            ),
            ToolDefinition::from_json_schema(
                "delete_section",
                "Delete a section from a document",
                "document",
                json!({
                    "type": "object",
                    "properties": {
                        "document_id": {"type": "string", "description": "ID of the document"},
                        "section_id": {"type": "integer", "description": "ID of the section to delete"}
                    },
                    "required": ["document_id", "section_id"]
                }),
                Some(ToolAnnotation::new("section_manager", "Deletes a section")),
            ),
            ToolDefinition::from_json_schema(
                "get_sections",
                "Get all sections in a document",
                "document",
                json!({
                    "type": "object",
                    "properties": {
                        "document_id": {"type": "string", "description": "ID of the document"}
                    },
                    "required": ["document_id"]
                }),
                Some(ToolAnnotation::new("section_manager", "Gets all sections in a document")),
            ),
            ToolDefinition::from_json_schema(
                "add_paragraph",
                "Add a paragraph to a section",
                "document",
                json!({
                    "type": "object",
                    "properties": {
                        "document_id": {"type": "string", "description": "ID of the document"},
                        "section_id": {"type": "integer", "description": "ID of the section"},
                        "text": {"type": "string", "description": "Paragraph text"}
                    },
                    "required": ["document_id", "section_id", "text"]
                }),
                Some(ToolAnnotation::new("content_manager", "Adds a paragraph to a section")),
            ),
            ToolDefinition::from_json_schema(
                "add_table",
                "Add a table to a section",
                "document",
                json!({
                    "type": "object",
                    "properties": {
                        "document_id": {"type": "string", "description": "ID of the document"},
                        "section_id": {"type": "integer", "description": "ID of the section"},
                        "header": {"type": "array", "description": "Table header row", "items": {"type": "string"}},
                        "rows": {"type": "array", "description": "Table rows", "items": {"type": "array", "items": {"type": "string"}}}
                    },
                    "required": ["document_id", "section_id", "rows"]
                }),
                Some(ToolAnnotation::new("content_manager", "Adds a table to a section")),
            ),
            ToolDefinition::from_json_schema(
                "add_image",
                "Add an image to a section",
                "document",
                json!({
                    "type": "object",
                    "properties": {
                        "document_id": {"type": "string", "description": "ID of the document"},
                        "section_id": {"type": "integer", "description": "ID of the section"},
                        "image": {"type": "string", "description": "Base64 encoded image data"},
                        "caption": {"type": "string", "description": "Image caption (optional)"}
                    },
                    "required": ["document_id", "section_id", "image"]
                }),
                Some(ToolAnnotation::new("content_manager", "Adds an image to a section")),
            ),
            ToolDefinition::from_json_schema(
                "save_document",
                "Save a document to a file",
                "document",
                json!({
                    "type": "object",
                    "properties": {
                        "document_id": {"type": "string", "description": "ID of the document"},
                        "filepath": {"type": "string", "description": "Path to save the document"}
                    },
                    "required": ["document_id", "filepath"]
                }),
                Some(ToolAnnotation::new("document_manager", "Saves a document to a file")),
            ),
            ToolDefinition::from_json_schema(
                "load_document",
                "Load a document from a file",
                "document",
                json!({
                    "type": "object",
                    "properties": {
                        "filepath": {"type": "string", "description": "Path to load the document from"}
                    },
                    "required": ["filepath"]
                }),
                Some(ToolAnnotation::new("document_manager", "Loads a document from a file")),
            ),
            ToolDefinition::from_json_schema(
                "generate_document",
                "Generate a document using AI",
                "document",
                json!({
                    "type": "object",
                    "properties": {
                        "topic": {"type": "string", "description": "Document topic"},
                        "length": {"type": "integer", "description": "Approximate document length (in words)"}
                    },
                    "required": ["topic", "length"]
                }),
                Some(ToolAnnotation::new("document_creator", "Generates a document using AI")),
            ),
        ]
    }
}