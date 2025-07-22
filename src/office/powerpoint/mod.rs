use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::tools::{ToolDefinition, ToolAnnotation};
use serde::{Deserialize, Serialize};
use serde_json::json;

/// Text formatting options for PowerPoint slides
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
    /// Text color in hex (e.g., '#000000' for black)
    pub color: Option<String>,
}

/// Model for a bullet point with text and level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletPoint {
    /// Text content of the bullet point
    pub text: String,
    /// Indentation level of the bullet point (0 for top level)
    pub level: u32,
    /// Text formatting for this bullet point
    pub formatting: Option<TextFormatting>,
}

/// Slide layout type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlideLayout {
    /// Title slide
    Title,
    /// Title and content slide
    TitleAndContent,
    /// Section header slide
    SectionHeader,
    /// Two content slide
    TwoContent,
    /// Comparison slide
    Comparison,
    /// Title only slide
    TitleOnly,
    /// Blank slide
    Blank,
    /// Content with caption slide
    ContentWithCaption,
    /// Picture with caption slide
    PictureWithCaption,
}

/// Presentation theme
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresentationTheme {
    /// Default theme
    Default,
    /// Office theme
    Office,
    /// Organic theme
    Organic,
    /// Modern theme
    Modern,
    /// Technical theme
    Technical,
    /// Dark theme
    Dark,
    /// Light theme
    Light,
    /// Bold theme
    Bold,
    /// Minimalist theme
    Minimalist,
}

/// Image content type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageType {
    /// JPEG image
    Jpeg,
    /// PNG image
    Png,
    /// SVG image
    Svg,
}

/// Image content for slides
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    /// Base64-encoded image data
    pub data: String,
    /// Image type (JPEG, PNG, SVG)
    pub image_type: ImageType,
    /// Alternative text for accessibility
    pub alt_text: Option<String>,
    /// Width in pixels
    pub width: Option<u32>,
    /// Height in pixels
    pub height: Option<u32>,
}

/// Slide model for PowerPoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slide {
    /// Slide title
    pub title: String,
    /// Slide subtitle (for title slides)
    pub subtitle: Option<String>,
    /// Slide content (for content slides)
    pub content: Option<String>,
    /// Slide layout
    pub layout: SlideLayout,
    /// List of bullet points for the slide
    pub bullets: Option<Vec<BulletPoint>>,
    /// Image for the slide
    pub image: Option<Image>,
    /// Speaker notes for the slide
    pub notes: Option<String>,
}

/// Presentation model for PowerPoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {
    /// Presentation title
    pub title: String,
    /// Presentation author
    pub author: Option<String>,
    /// Presentation theme
    pub theme: PresentationTheme,
    /// Slides in the presentation
    pub slides: Vec<Slide>,
}

/// PowerPoint client for managing presentations
pub struct PowerPointClient<'a> {
    /// Lifecycle manager
    lifecycle: &'a LifecycleManager,
}

impl<'a> PowerPointClient<'a> {
    /// Create a new PowerPoint client
    pub fn new(lifecycle: &'a LifecycleManager) -> Self {
        Self { lifecycle }
    }

    /// Create a new presentation
    pub async fn create_presentation(&self, presentation: Presentation) -> Result<String> {
        let method = "tools/execute";
        let params = json!({
            "name": "create_presentation",
            "args": {
                "title": presentation.title,
                "author": presentation.author,
                "theme": presentation.theme,
            }
        });

        let response = self.lifecycle.call_method(method, Some(params)).await?;
        let presentation_id = response["presentation_id"]
            .as_str()
            .ok_or_else(|| Error::parsing("Failed to parse presentation ID"))?
            .to_string();

        // Add slides
        for slide in presentation.slides {
            self.add_slide(&presentation_id, slide).await?;
        }

        Ok(presentation_id)
    }

    /// Add a slide to an existing presentation
    pub async fn add_slide(&self, presentation_id: &str, slide: Slide) -> Result<u32> {
        let method = "tools/execute";
        let mut args = json!({
            "presentation_id": presentation_id,
            "title": slide.title,
            "layout": slide.layout,
        });

        if let Some(subtitle) = slide.subtitle {
            args["subtitle"] = json!(subtitle);
        }

        if let Some(content) = slide.content {
            args["content"] = json!(content);
        }

        if let Some(bullets) = slide.bullets {
            args["bullets"] = json!(bullets);
        }

        if let Some(image) = slide.image {
            args["image"] = json!(image);
        }

        if let Some(notes) = slide.notes {
            args["notes"] = json!(notes);
        }

        let params = json!({
            "name": "add_slide",
            "args": args
        });

        let response = self.lifecycle.call_method(method, Some(params)).await?;
        let slide_id = response["slide_id"]
            .as_u64()
            .ok_or_else(|| Error::parsing("Failed to parse slide ID"))?;

        Ok(slide_id as u32)
    }

    /// Update an existing slide
    pub async fn update_slide(&self, presentation_id: &str, slide_id: u32, slide: Slide) -> Result<()> {
        let method = "tools/execute";
        let mut args = json!({
            "presentation_id": presentation_id,
            "slide_id": slide_id,
            "title": slide.title,
            "layout": slide.layout,
        });

        if let Some(subtitle) = slide.subtitle {
            args["subtitle"] = json!(subtitle);
        }

        if let Some(content) = slide.content {
            args["content"] = json!(content);
        }

        if let Some(bullets) = slide.bullets {
            args["bullets"] = json!(bullets);
        }

        if let Some(image) = slide.image {
            args["image"] = json!(image);
        }

        if let Some(notes) = slide.notes {
            args["notes"] = json!(notes);
        }

        let params = json!({
            "name": "update_slide",
            "args": args
        });

        self.lifecycle.call_method(method, Some(params)).await?;
        Ok(())
    }

    /// Delete a slide
    pub async fn delete_slide(&self, presentation_id: &str, slide_id: u32) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "delete_slide",
            "args": {
                "presentation_id": presentation_id,
                "slide_id": slide_id
            }
        });

        self.lifecycle.call_method(method, Some(params)).await?;
        Ok(())
    }

    /// Reorder slides
    pub async fn reorder_slides(&self, presentation_id: &str, slide_ids: Vec<u32>) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "reorder_slides",
            "args": {
                "presentation_id": presentation_id,
                "slide_ids": slide_ids
            }
        });

        self.lifecycle.call_method(method, Some(params)).await?;
        Ok(())
    }

    /// Get all slides in a presentation
    pub async fn get_slides(&self, presentation_id: &str) -> Result<Vec<Slide>> {
        let method = "tools/execute";
        let params = json!({
            "name": "get_slides",
            "args": {
                "presentation_id": presentation_id
            }
        });

        let response = self.lifecycle.call_method(method, Some(params)).await?;
        let slides = serde_json::from_value(response["slides"].clone())
            .map_err(|e| Error::parsing(format!("Failed to parse slides: {}", e)))?;

        Ok(slides)
    }

    /// Save the presentation to a file
    pub async fn save_presentation(&self, presentation_id: &str, filepath: &str) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "save_presentation",
            "args": {
                "presentation_id": presentation_id,
                "filepath": filepath
            }
        });

        self.lifecycle.call_method(method, Some(params)).await?;
        Ok(())
    }

    /// Load a presentation from a file
    pub async fn load_presentation(&self, filepath: &str) -> Result<String> {
        let method = "tools/execute";
        let params = json!({
            "name": "load_presentation",
            "args": {
                "filepath": filepath
            }
        });

        let response = self.lifecycle.call_method(method, Some(params)).await?;
        let presentation_id = response["presentation_id"]
            .as_str()
            .ok_or_else(|| Error::parsing("Failed to parse presentation ID"))?
            .to_string();

        Ok(presentation_id)
    }

    /// Generate a presentation from a template using AI
    pub async fn generate_presentation(&self, topic: &str, num_slides: u32, theme: PresentationTheme) -> Result<String> {
        let method = "tools/execute";
        let params = json!({
            "name": "generate_presentation",
            "args": {
                "topic": topic,
                "num_slides": num_slides,
                "theme": theme
            }
        });

        let response = self.lifecycle.call_method(method, Some(params)).await?;
        let presentation_id = response["presentation_id"]
            .as_str()
            .ok_or_else(|| Error::parsing("Failed to parse presentation ID"))?
            .to_string();

        Ok(presentation_id)
    }

    /// Add an image to a slide
    pub async fn add_image_to_slide(&self, presentation_id: &str, slide_id: u32, image: Image) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "add_image_to_slide",
            "args": {
                "presentation_id": presentation_id,
                "slide_id": slide_id,
                "image": image
            }
        });

        self.lifecycle.call_method(method, Some(params)).await?;
        Ok(())
    }

    /// Change the theme of a presentation
    pub async fn change_theme(&self, presentation_id: &str, theme: PresentationTheme) -> Result<()> {
        let method = "tools/execute";
        let params = json!({
            "name": "change_theme",
            "args": {
                "presentation_id": presentation_id,
                "theme": theme
            }
        });

        self.lifecycle.call_method(method, Some(params)).await?;
        Ok(())
    }

    /// Get available tools
    pub fn get_tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition::from_json_schema(
                "create_presentation",
                "Create a new PowerPoint presentation",
                "presentation",
                json!({
                    "type": "object",
                    "properties": {
                        "title": {"type": "string", "description": "Title of the presentation"},
                        "template": {"type": "string", "description": "Template to use (optional)"}
                    },
                    "required": ["title"]
                }),
                Some(ToolAnnotation::new("presentation_creator").with_description("Creates a new presentation"))
            ),
            ToolDefinition::from_json_schema(
                "add_slide",
                "Add a slide to a presentation",
                "presentation",
                json!({
                    "type": "object",
                    "properties": {
                        "presentation_id": {"type": "string", "description": "ID of the presentation"},
                        "title": {"type": "string", "description": "Slide title"},
                        "content": {"type": "string", "description": "Slide content"},
                        "layout": {"type": "string", "description": "Slide layout", "enum": ["Title", "TitleAndContent", "SectionHeader", "TwoContent", "Comparison", "TitleOnly", "Blank", "ContentWithCaption", "PictureWithCaption"]}
                    },
                    "required": ["presentation_id", "title", "layout"]
                }),
                Some(ToolAnnotation::new("slide_manager").with_description("Adds a slide to a presentation")),
            ),
            ToolDefinition::from_json_schema(
                "update_slide",
                "Update an existing slide",
                "presentation",
                json!({
                    "type": "object",
                    "properties": {
                        "presentation_id": {"type": "string", "description": "ID of the presentation"},
                        "slide_id": {"type": "integer", "description": "ID of the slide to update"},
                        "title": {"type": "string", "description": "New slide title"},
                        "content": {"type": "string", "description": "New slide content"},
                        "layout": {"type": "string", "description": "New slide layout", "enum": ["Title", "TitleAndContent", "SectionHeader", "TwoContent", "Comparison", "TitleOnly", "Blank", "ContentWithCaption", "PictureWithCaption"]}
                    },
                    "required": ["presentation_id", "slide_id"]
                }),
                Some(ToolAnnotation::new("slide_manager").with_description("Updates an existing slide")),
            ),
            ToolDefinition::from_json_schema(
                "delete_slide",
                "Delete a slide from a presentation",
                "presentation",
                json!({
                    "type": "object",
                    "properties": {
                        "presentation_id": {"type": "string", "description": "ID of the presentation"},
                        "slide_id": {"type": "integer", "description": "ID of the slide to delete"}
                    },
                    "required": ["presentation_id", "slide_id"]
                }),
                Some(ToolAnnotation::new("slide_manager").with_description("Deletes a slide")),
            ),
            ToolDefinition::from_json_schema(
                "reorder_slides",
                "Reorder slides in a presentation",
                "presentation",
                json!({
                    "type": "object",
                    "properties": {
                        "presentation_id": {"type": "string", "description": "ID of the presentation"},
                        "slide_ids": {"type": "array", "description": "List of slide IDs in new order", "items": {"type": "integer"}}
                    },
                    "required": ["presentation_id", "slide_ids"]
                }),
                Some(ToolAnnotation::new("slide_manager").with_description("Reorders slides in a presentation")),
            ),
            ToolDefinition::from_json_schema(
                "get_slides",
                "Get all slides in a presentation",
                "presentation",
                json!({
                    "type": "object",
                    "properties": {
                        "presentation_id": {"type": "string", "description": "ID of the presentation"}
                    },
                    "required": ["presentation_id"]
                }),
                Some(ToolAnnotation::new("slide_manager").with_description("Gets all slides in a presentation")),
            ),
            ToolDefinition::from_json_schema(
                "save_presentation",
                "Save a presentation to a file",
                "presentation",
                json!({
                    "type": "object",
                    "properties": {
                        "presentation_id": {"type": "string", "description": "ID of the presentation"},
                        "filepath": {"type": "string", "description": "Path to save the presentation"}
                    },
                    "required": ["presentation_id", "filepath"]
                }),
                Some(ToolAnnotation::new("presentation_manager").with_description("Saves a presentation to a file")),
            ),
            ToolDefinition::from_json_schema(
                "load_presentation",
                "Load a presentation from a file",
                "presentation",
                json!({
                    "type": "object",
                    "properties": {
                        "filepath": {"type": "string", "description": "Path to load the presentation from"}
                    },
                    "required": ["filepath"]
                }),
                Some(ToolAnnotation::new("presentation_manager").with_description("Loads a presentation from a file")),
            ),
            ToolDefinition::from_json_schema(
                "generate_presentation",
                "Generate a presentation using AI",
                "presentation",
                json!({
                    "type": "object",
                    "properties": {
                        "topic": {"type": "string", "description": "Presentation topic"},
                        "num_slides": {"type": "integer", "description": "Number of slides to generate"}
                    },
                    "required": ["topic", "num_slides"]
                }),
                Some(ToolAnnotation::new("presentation_creator").with_description("Generates a presentation using AI")),
            ),
            ToolDefinition::from_json_schema(
                "add_image_to_slide",
                "Add an image to a slide",
                "presentation",
                json!({
                    "type": "object",
                    "properties": {
                        "presentation_id": {"type": "string", "description": "ID of the presentation"},
                        "slide_id": {"type": "integer", "description": "ID of the slide"},
                        "image": {"type": "string", "description": "Base64 encoded image data"}
                    },
                    "required": ["presentation_id", "slide_id", "image"]
                }),
                Some(ToolAnnotation::new("slide_manager").with_description("Adds an image to a slide")),
            ),
            ToolDefinition::from_json_schema(
                "change_theme",
                "Change the theme of a presentation",
                "presentation",
                json!({
                    "type": "object",
                    "properties": {
                        "presentation_id": {"type": "string", "description": "ID of the presentation"},
                        "theme": {"type": "string", "description": "New theme name"}
                    },
                    "required": ["presentation_id", "theme"]
                }),
                Some(ToolAnnotation::new("presentation_manager").with_description("Changes the theme of a presentation")),
            ),
        ]
    }
} 