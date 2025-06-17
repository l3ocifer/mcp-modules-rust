use crate::error::{Error, Result};
use crate::lifecycle::LifecycleManager;
use crate::tools::ToolDefinition;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Available research tones
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResearchTone {
    /// Objective tone
    #[serde(rename = "objective")]
    Objective,
    /// Critical tone
    #[serde(rename = "critical")]
    Critical,
    /// Optimistic tone
    #[serde(rename = "optimistic")]
    Optimistic,
    /// Balanced tone
    #[serde(rename = "balanced")]
    Balanced,
    /// Skeptical tone
    #[serde(rename = "skeptical")]
    Skeptical,
}

impl Default for ResearchTone {
    fn default() -> Self {
        Self::Objective
    }
}

/// Research report with sections and citations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchReport {
    /// Report title
    pub title: String,
    /// Report description/summary
    pub description: String,
    /// Report sections
    pub sections: Vec<ResearchSection>,
    /// Citations used in the report
    pub citations: Vec<Citation>,
    /// Research tone
    pub tone: ResearchTone,
}

/// Research section in a report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSection {
    /// Section title
    pub title: String,
    /// Section content
    pub content: String,
    /// Subsections
    pub subsections: Option<Vec<ResearchSection>>,
}

/// Citation for research sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    /// Citation ID for reference
    pub id: String,
    /// Citation title
    pub title: String,
    /// Citation authors
    pub authors: Vec<String>,
    /// Publication date
    pub date: Option<String>,
    /// Publication source
    pub source: Option<String>,
    /// URL for online citations
    pub url: Option<String>,
    /// DOI for academic papers
    pub doi: Option<String>,
}

/// Search result with snippets and source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Result title
    pub title: String,
    /// Result URL
    pub url: String,
    /// Snippets from the result
    pub snippets: Vec<String>,
    /// Result source
    pub source: String,
}

/// Deep Research client for comprehensive research
pub struct DeepResearchClient<'a> {
    /// Lifecycle manager
    lifecycle: &'a LifecycleManager,
}

impl<'a> DeepResearchClient<'a> {
    /// Create a new Deep Research client
    pub fn new(lifecycle: &'a LifecycleManager) -> Self {
        Self { lifecycle }
    }

    /// Research a topic in depth
    pub async fn research_topic(&self, topic: &str, depth: u32, tone: ResearchTone) -> Result<ResearchReport> {
        let method = "tools/execute";
        let params = json!({
            "name": "deep_research",
            "args": {
                "topic": topic,
                "depth": depth,
                "tone": tone
            }
        });

        let response = self.lifecycle.send_request(method, Some(params)).await?;
        let report = serde_json::from_value(response)
            .map_err(|e| Error::parsing(format!("Failed to parse research report: {}", e)))?;
        
        Ok(report)
    }

    /// Search for information on a specific topic
    pub async fn search(&self, query: &str, num_results: Option<u32>) -> Result<Vec<SearchResult>> {
        let method = "tools/execute";
        let mut args = json!({
            "query": query
        });

        if let Some(num) = num_results {
            args["num_results"] = json!(num);
        }

        let params = json!({
            "name": "deep_research",
            "args": {
                "topic": query,
                "depth": 1,
                "tone": ResearchTone::Objective
            }
        });

        let response = self.lifecycle.send_request(method, Some(params)).await?;
        let results = serde_json::from_value(response["results"].clone())
            .map_err(|e| Error::parsing(format!("Failed to parse search results: {}", e)))?;
        
        Ok(results)
    }

    /// Summarize a document or research paper
    pub async fn summarize_document(&self, url: &str, summary_length: Option<u32>) -> Result<String> {
        let method = "tools/execute";
        let mut args = json!({
            "url": url
        });

        if let Some(length) = summary_length {
            args["summary_length"] = json!(length);
        }

        let params = json!({
            "name": "deep_research",
            "args": {
                "topic": url,
                "depth": 1,
                "tone": ResearchTone::Objective
            }
        });

        let response = self.lifecycle.send_request(method, Some(params)).await?;
        let summary = response["summary"]
            .as_str()
            .ok_or_else(|| Error::parsing("Failed to parse document summary".to_string()))?
            .to_string();
        
        Ok(summary)
    }

    /// Find citations for a research topic
    pub async fn find_citations(&self, topic: &str, num_citations: Option<u32>) -> Result<Vec<Citation>> {
        let method = "tools/execute";
        let mut args = json!({
            "topic": topic
        });

        if let Some(num) = num_citations {
            args["num_citations"] = json!(num);
        }

        let params = json!({
            "name": "deep_research",
            "args": {
                "topic": topic,
                "depth": 1,
                "tone": ResearchTone::Objective
            }
        });

        let response = self.lifecycle.send_request(method, Some(params)).await?;
        let citations = serde_json::from_value(response["citations"].clone())
            .map_err(|e| Error::parsing(format!("Failed to parse citations: {}", e)))?;
        
        Ok(citations)
    }

    /// Compare multiple topics or research areas
    pub async fn compare_topics(&self, topics: Vec<String>, comparison_points: Option<Vec<String>>) -> Result<Value> {
        let method = "tools/execute";
        let mut args = json!({
            "topics": topics
        });

        if let Some(points) = comparison_points {
            args["comparison_points"] = json!(points);
        }

        let params = json!({
            "name": "deep_research",
            "args": {
                "topic": topics[0],
                "depth": 1,
                "tone": ResearchTone::Objective
            }
        });

        let response = self.lifecycle.send_request(method, Some(params)).await?;
        Ok(response["comparison"].clone())
    }

    /// Generate a detailed research outline
    pub async fn generate_outline(&self, topic: &str, depth: Option<u32>) -> Result<Vec<ResearchSection>> {
        let method = "tools/execute";
        let mut args = json!({
            "topic": topic
        });

        if let Some(d) = depth {
            args["depth"] = json!(d);
        }

        let params = json!({
            "name": "deep_research",
            "args": {
                "topic": topic,
                "depth": depth,
                "tone": ResearchTone::Objective
            }
        });

        let response = self.lifecycle.send_request(method, Some(params)).await?;
        let outline = serde_json::from_value(response["outline"].clone())
            .map_err(|e| Error::parsing(format!("Failed to parse outline: {}", e)))?;
        
        Ok(outline)
    }

    /// Get available tools
    pub fn get_tools(&self) -> Vec<ToolDefinition> {
        vec![
            ToolDefinition::new(
                "research",
                "Research a topic in depth",
            ),
            ToolDefinition::new(
                "search",
                "Search for information on a specific topic",
            ),
            ToolDefinition::new(
                "summarize",
                "Summarize a document or research paper",
            ),
            ToolDefinition::new(
                "find_citations",
                "Find citations for a research topic",
            ),
            ToolDefinition::new(
                "compare_topics",
                "Compare multiple topics or research areas",
            ),
            ToolDefinition::new(
                "generate_outline",
                "Generate a detailed research outline",
            ),
        ]
    }
} 