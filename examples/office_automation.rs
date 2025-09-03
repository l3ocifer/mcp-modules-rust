/// Office Automation Example
/// 
/// This example demonstrates how to use the Office modules to:
/// 1. Create PowerPoint presentations
/// 2. Generate Word documents
/// 3. Work with Excel spreadsheets
/// 4. Automate document creation workflows

use devops_mcp::{new, Config};
use devops_mcp::office::powerpoint::{PowerPointClient, Presentation, Slide, SlideLayout, PresentationTheme, BulletPoint, Image, ImageType};
use devops_mcp::office::word::{WordClient, Document, Paragraph, Section, TextFormatting, Alignment};
use devops_mcp::office::excel::{ExcelClient, Workbook, Worksheet, Cell, CellValue, CellFormat, Row, Column};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("devops_mcp=debug")
        .init();
    
    println!("📊 MCP Modules Rust - Office Automation Example");
    
    // Initialize the client
    let config = Config::default();
    let mut client = new(config)?;
    client.initialize().await?;
    let lifecycle = client.lifecycle()?;
    
    // Example 1: PowerPoint Presentation Creation
    println!("\n🎯 Creating PowerPoint Presentation");
    
    let powerpoint = PowerPointClient::new(&lifecycle);
    
    let presentation = Presentation {
        title: "Q4 2024 Performance Report".to_string(),
        author: Some("DevOps Team".to_string()),
        theme: PresentationTheme::Office,
        slides: vec![
            // Title slide
            Slide {
                title: "Q4 2024 Performance Report".to_string(),
                layout: SlideLayout::Title,
                subtitle: Some("DevOps Infrastructure Metrics".to_string()),
                content: None,
                bullets: None,
                image: None,
                notes: Some("Welcome everyone to our quarterly review".to_string()),
            },
            
            // Overview slide
            Slide {
                title: "Executive Summary".to_string(),
                layout: SlideLayout::TitleAndContent,
                subtitle: None,
                content: Some("Our infrastructure improvements have delivered significant results this quarter.".to_string()),
                bullets: Some(vec![
                    BulletPoint {
                        text: "99.9% uptime achieved across all services".to_string(),
                        level: 0,
                        formatting: None,
                    },
                    BulletPoint {
                        text: "Response time improved by 45%".to_string(),
                        level: 0,
                        formatting: None,
                    },
                    BulletPoint {
                        text: "Infrastructure costs reduced by 30%".to_string(),
                        level: 0,
                        formatting: None,
                    },
                    BulletPoint {
                        text: "Zero security incidents reported".to_string(),
                        level: 0,
                        formatting: None,
                    },
                ]),
                image: None,
                notes: Some("Highlight the key achievements and metrics".to_string()),
            },
            
            // Performance metrics slide
            Slide {
                title: "Performance Metrics".to_string(),
                layout: SlideLayout::TitleAndContent,
                subtitle: None,
                content: Some("Key performance indicators for Q4 2024:".to_string()),
                bullets: Some(vec![
                    BulletPoint {
                        text: "Average response time: 120ms (down from 220ms)".to_string(),
                        level: 0,
                        formatting: None,
                    },
                    BulletPoint {
                        text: "Throughput: 10,000 req/sec (up from 6,500)".to_string(),
                        level: 0,
                        formatting: None,
                    },
                    BulletPoint {
                        text: "Error rate: 0.01% (down from 0.05%)".to_string(),
                        level: 0,
                        formatting: None,
                    },
                    BulletPoint {
                        text: "Memory usage: 65% average (optimized from 85%)".to_string(),
                        level: 0,
                        formatting: None,
                    },
                ]),
                image: Some(Image {
                    data: "".to_string(), // Base64 encoded image data would go here
                    image_type: ImageType::Png,
                    alt_text: Some("Performance chart showing metrics improvement".to_string()),
                    width: Some(600),
                    height: Some(400),
                }),
                notes: Some("Discuss the technical improvements that led to these gains".to_string()),
            },
            
            // Future plans slide
            Slide {
                title: "Q1 2025 Roadmap".to_string(),
                layout: SlideLayout::TitleAndContent,
                subtitle: None,
                content: Some("Strategic initiatives for the upcoming quarter:".to_string()),
                bullets: Some(vec![
                    BulletPoint {
                        text: "Migrate to Kubernetes for better scalability".to_string(),
                        level: 0,
                        formatting: None,
                    },
                    BulletPoint {
                        text: "Implement observability stack with OpenTelemetry".to_string(),
                        level: 0,
                        formatting: None,
                    },
                    BulletPoint {
                        text: "Establish GitOps workflows with ArgoCD".to_string(),
                        level: 0,
                        formatting: None,
                    },
                    BulletPoint {
                        text: "Enhance security with zero-trust architecture".to_string(),
                        level: 0,
                        formatting: None,
                    },
                ]),
                image: None,
                notes: Some("Outline the technical strategy for next quarter".to_string()),
            },
        ],
    };
    
    match powerpoint.create_presentation(presentation).await {
        Ok(presentation_id) => {
            println!("✅ PowerPoint presentation created successfully!");
            println!("   Presentation ID: {}", presentation_id);
            println!("   Slides: 4");
            println!("   Theme: Office");
        },
        Err(e) => {
            println!("⚠️  Failed to create presentation: {}", e);
        }
    }
    
    // Example 2: Word Document Generation
    println!("\n📄 Creating Word Document");
    
    let word = WordClient::new(&lifecycle);
    
    let document = Document {
        title: "Infrastructure Deployment Guide".to_string(),
        author: Some("DevOps Team".to_string()),
        sections: vec![
            Section {
                title: Some("Overview".to_string()),
                paragraphs: vec![
                    Paragraph {
                        text: "This document provides comprehensive guidelines for deploying and managing our infrastructure using modern DevOps practices.".to_string(),
                        formatting: Some(TextFormatting {
                            font_name: None,
                            font_size: None,
                            bold: None,
                            italic: None,
                            underline: None,
                            color: None,
                        }),
                        alignment: Some(Alignment::Left),
                        is_heading: None,
                        heading_level: None,
                    },
                    Paragraph {
                        text: "Our infrastructure is built on cloud-native principles with emphasis on automation, monitoring, and security.".to_string(),
                        formatting: Some(TextFormatting {
                            font_name: None,
                            font_size: None,
                            bold: None,
                            italic: None,
                            underline: None,
                            color: None,
                        }),
                        alignment: Some(Alignment::Left),
                        is_heading: None,
                        heading_level: None,
                    },
                ],
                tables: None,
                images: None,
            },
            Section {
                title: Some("Prerequisites".to_string()),
                paragraphs: vec![
                    Paragraph {
                        text: "Before beginning the deployment process, ensure you have:".to_string(),
                        formatting: Some(TextFormatting {
                            font_name: None,
                            font_size: None,
                            bold: None,
                            italic: None,
                            underline: None,
                            color: None,
                        }),
                        alignment: Some(Alignment::Left),
                        is_heading: None,
                        heading_level: None,
                    },
                    Paragraph {
                        text: "• Docker installed and configured\n• Kubernetes cluster access\n• Required cloud provider credentials\n• Terraform >= 1.0 installed".to_string(),
                        formatting: Some(TextFormatting {
                            font_name: None,
                            font_size: None,
                            bold: None,
                            italic: None,
                            underline: None,
                            color: None,
                        }),
                        alignment: Some(Alignment::Left),
                        is_heading: None,
                        heading_level: None,
                    },
                ],
                tables: None,
                images: None,
            },
            Section {
                title: Some("Deployment Steps".to_string()),
                paragraphs: vec![
                    Paragraph {
                        text: "Step 1: Infrastructure Provisioning".to_string(),
                        formatting: Some(TextFormatting {
                            font_name: None,
                            font_size: Some(16),
                            bold: Some(true),
                            italic: None,
                            underline: None,
                            color: None,
                        }),
                        alignment: Some(Alignment::Left),
                        is_heading: Some(true),
                        heading_level: Some(3),
                    },
                    Paragraph {
                        text: "Use Terraform to provision the base infrastructure components including VPC, subnets, and security groups.".to_string(),
                        formatting: Some(TextFormatting {
                            font_name: None,
                            font_size: None,
                            bold: None,
                            italic: None,
                            underline: None,
                            color: None,
                        }),
                        alignment: Some(Alignment::Left),
                        is_heading: None,
                        heading_level: None,
                    },
                    Paragraph {
                        text: "terraform init\nterraform plan\nterraform apply".to_string(),
                        formatting: Some(TextFormatting {
                            font_name: Some("Courier New".to_string()),
                            font_size: Some(10),
                            bold: None,
                            italic: None,
                            underline: None,
                            color: Some("#333333".to_string()),
                        }),
                        alignment: Some(Alignment::Left),
                        is_heading: None,
                        heading_level: None,
                    },
                ],
                tables: None,
                images: None,
            },
        ],
    };
    
    match word.create_document(document).await {
        Ok(document_id) => {
            println!("✅ Word document created successfully!");
            println!("   Document ID: {}", document_id);
            println!("   Sections: 3");
            println!("   Title: Infrastructure Deployment Guide");
        },
        Err(e) => {
            println!("⚠️  Failed to create document: {}", e);
        }
    }
    
    // Example 3: Excel Spreadsheet with Performance Data
    println!("\n📈 Creating Excel Spreadsheet");
    
    let excel = ExcelClient::new(&lifecycle);
    
    let workbook = Workbook {
        title: "Q4 Performance Metrics".to_string(),
        author: Some("DevOps Team".to_string()),
        worksheets: vec![
            Worksheet {
                name: "Performance Data".to_string(),
                columns: Some(vec![
                    Column { index: 0, width: Some(150.0), format: None },
                    Column { index: 1, width: Some(100.0), format: None },
                    Column { index: 2, width: Some(100.0), format: None },
                    Column { index: 3, width: Some(120.0), format: None },
                ]),
                rows: vec![
                    Row {
                        index: 0,
                        cells: vec![
                            Cell { value: CellValue::Text("Metric".to_string()), format: Some(CellFormat { bold: Some(true), background_color: Some("#CCCCCC".to_string()), font_name: None, font_size: None, italic: None, underline: None, color: None, number_format: None, alignment: None }) },
                            Cell { value: CellValue::Text("Q3 2024".to_string()), format: Some(CellFormat { bold: Some(true), background_color: Some("#CCCCCC".to_string()), font_name: None, font_size: None, italic: None, underline: None, color: None, number_format: None, alignment: None }) },
                            Cell { value: CellValue::Text("Q4 2024".to_string()), format: Some(CellFormat { bold: Some(true), background_color: Some("#CCCCCC".to_string()), font_name: None, font_size: None, italic: None, underline: None, color: None, number_format: None, alignment: None }) },
                            Cell { value: CellValue::Text("Improvement".to_string()), format: Some(CellFormat { bold: Some(true), background_color: Some("#CCCCCC".to_string()), font_name: None, font_size: None, italic: None, underline: None, color: None, number_format: None, alignment: None }) },
                        ],
                        height: None,
                    },
                    Row {
                        index: 1,
                        cells: vec![
                            Cell { value: CellValue::Text("Response Time (ms)".to_string()), format: None },
                            Cell { value: CellValue::Number(220.0), format: None },
                            Cell { value: CellValue::Number(120.0), format: None },
                            Cell { value: CellValue::Text("45%".to_string()), format: Some(CellFormat { color: Some("#00AA00".to_string()), font_name: None, font_size: None, bold: None, italic: None, underline: None, background_color: None, number_format: None, alignment: None }) },
                        ],
                        height: None,
                    },
                    Row {
                        index: 1,
                        cells: vec![
                            Cell { value: CellValue::Text("Throughput (req/sec)".to_string()), format: None },
                        Cell { value: CellValue::Number(6500.0), format: None },
                        Cell { value: CellValue::Number(10000.0), format: None },
                        Cell { value: CellValue::Text("54%".to_string()), format: Some(CellFormat { color: Some("#008000".to_string()), font_name: None, font_size: None, bold: None, italic: None, underline: None, background_color: None, number_format: None, alignment: None }) },
                        ],
                        height: None,
                    },
                    Row {
                        index: 2,
                        cells: vec![
                            Cell { value: CellValue::Text("Error Rate (%)".to_string()), format: None },
                        Cell { value: CellValue::Number(0.05), format: None },
                        Cell { value: CellValue::Number(0.01), format: None },
                        Cell { value: CellValue::Text("80%".to_string()), format: Some(CellFormat { color: Some("#008000".to_string()), font_name: None, font_size: None, bold: None, italic: None, underline: None, background_color: None, number_format: None, alignment: None }) },
                        ],
                        height: None,
                    },
                    Row {
                        index: 3,
                        cells: vec![
                            Cell { value: CellValue::Text("Memory Usage (%)".to_string()), format: None },
                        Cell { value: CellValue::Number(85.0), format: None },
                        Cell { value: CellValue::Number(65.0), format: None },
                        Cell { value: CellValue::Text("24%".to_string()), format: Some(CellFormat { color: Some("#008000".to_string()), font_name: None, font_size: None, bold: None, italic: None, underline: None, background_color: None, number_format: None, alignment: None }) },
                        ],
                        height: None,
                    },
                ],
                charts: Some(vec![]),
            },
            Worksheet {
                name: "Cost Analysis".to_string(),
                columns: Some(vec![
                    Column { index: 0, width: Some(120.0), format: None },
                    Column { index: 1, width: Some(100.0), format: None },
                    Column { index: 2, width: Some(100.0), format: None },
                    Column { index: 3, width: Some(100.0), format: None },
                ]),
                rows: vec![
                    Row {
                        index: 0,
                        cells: vec![
                        Cell { value: CellValue::Text("Service".to_string()), format: Some(CellFormat { bold: Some(true), font_name: None, font_size: None, italic: None, underline: None, color: None, background_color: None, number_format: None, alignment: None }) },
                        Cell { value: CellValue::Text("Q3 Cost".to_string()), format: Some(CellFormat { bold: Some(true), font_name: None, font_size: None, italic: None, underline: None, color: None, background_color: None, number_format: None, alignment: None }) },
                        Cell { value: CellValue::Text("Q4 Cost".to_string()), format: Some(CellFormat { bold: Some(true), font_name: None, font_size: None, italic: None, underline: None, color: None, background_color: None, number_format: None, alignment: None }) },
                        Cell { value: CellValue::Text("Savings".to_string()), format: Some(CellFormat { bold: Some(true), font_name: None, font_size: None, italic: None, underline: None, color: None, background_color: None, number_format: None, alignment: None }) },
                        ],
                        height: None,
                    },
                    Row {
                        index: 1,
                        cells: vec![
                            Cell { value: CellValue::Text("Compute".to_string()), format: None },
                        Cell { value: CellValue::Number(15000.0), format: None },
                        Cell { value: CellValue::Number(12000.0), format: None },
                        Cell { value: CellValue::Number(3000.0), format: Some(CellFormat { color: Some("#008000".to_string()), font_name: None, font_size: None, bold: None, italic: None, underline: None, background_color: None, number_format: None, alignment: None }) },
                        ],
                        height: None,
                    },
                    Row {
                        index: 2,
                        cells: vec![
                            Cell { value: CellValue::Text("Storage".to_string()), format: None },
                        Cell { value: CellValue::Number(5000.0), format: None },
                        Cell { value: CellValue::Number(3500.0), format: None },
                        Cell { value: CellValue::Number(1500.0), format: Some(CellFormat { color: Some("#008000".to_string()), font_name: None, font_size: None, bold: None, italic: None, underline: None, background_color: None, number_format: None, alignment: None }) },
                        ],
                        height: None,
                    },
                    Row {
                        index: 3,
                        cells: vec![
                            Cell { value: CellValue::Text("Network".to_string()), format: None },
                        Cell { value: CellValue::Number(2000.0), format: None },
                        Cell { value: CellValue::Number(1800.0), format: None },
                        Cell { value: CellValue::Number(200.0), format: Some(CellFormat { color: Some("#008000".to_string()), font_name: None, font_size: None, bold: None, italic: None, underline: None, background_color: None, number_format: None, alignment: None }) },
                        ],
                        height: None,
                    },
                ],
                charts: Some(vec![]),
            },
        ],
    };
    
    match excel.create_workbook(workbook).await {
        Ok(workbook_id) => {
            println!("✅ Excel workbook created successfully!");
            println!("   Workbook ID: {}", workbook_id);
            println!("   Worksheets: 2");
            println!("   Data points: 8 performance metrics");
        },
        Err(e) => {
            println!("⚠️  Failed to create workbook: {}", e);
        }
    }
    
    // Example 4: Automated Report Generation Workflow
    println!("\n🔄 Automated Workflow Example");
    
    // This would typically be triggered by a scheduler or event
    let workflow_results = generate_monthly_report(&powerpoint, &word, &excel).await;
    
    match workflow_results {
        Ok(report_ids) => {
            println!("✅ Automated report generation completed!");
            println!("   Generated {} documents", report_ids.len());
            for (doc_type, id) in report_ids {
                println!("   - {}: {}", doc_type, id);
            }
        },
        Err(e) => {
            println!("⚠️  Automated workflow failed: {}", e);
        }
    }
    
    // Example 5: Bulk Operations
    println!("\n📦 Bulk Operations Example");
    
    // Generate multiple presentations for different teams
    let teams = vec!["Backend", "Frontend", "DevOps", "Security"];
    
    for team in teams {
        let team_presentation = create_team_presentation(team);
        
        match powerpoint.create_presentation(team_presentation).await {
            Ok(id) => {
                println!("✅ Created presentation for {} team (ID: {})", team, id);
            },
            Err(e) => {
                println!("⚠️  Failed to create presentation for {} team: {}", team, e);
            }
        }
    }
    
    println!("\n🎉 Office automation example completed!");
    println!("💡 This example showed how to:");
    println!("   - Create comprehensive PowerPoint presentations");
    println!("   - Generate structured Word documents");
    println!("   - Build Excel spreadsheets with data");
    println!("   - Automate document workflows");
    println!("   - Perform bulk operations");
    
    Ok(())
}

// Helper function to generate monthly reports
async fn generate_monthly_report(
    powerpoint: &PowerPointClient<'_>,
    word: &WordClient<'_>,
    excel: &ExcelClient<'_>,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let mut results = Vec::new();
    
    // Generate executive summary presentation
    let exec_summary = Presentation {
        title: "Monthly Executive Summary".to_string(),
        author: Some("Automated Report Generator".to_string()),
        theme: PresentationTheme::Modern,
        slides: vec![
            Slide {
                title: "Monthly Performance Summary".to_string(),
                layout: SlideLayout::Title,
                subtitle: Some("Automated Infrastructure Report".to_string()),
                content: None,
                bullets: None,
                image: None,
                notes: None,
            },
        ],
    };
    
    let ppt_id = powerpoint.create_presentation(exec_summary).await?;
    results.push(("PowerPoint".to_string(), ppt_id));
    
    // Generate detailed technical document
    let tech_doc = Document {
        title: "Monthly Technical Report".to_string(),
        author: Some("Automated Report Generator".to_string()),
        sections: vec![
            Section {
                title: Some("System Status".to_string()),
                paragraphs: vec![
                    Paragraph {
                        text: "All systems operational with excellent performance metrics.".to_string(),
                        formatting: Some(TextFormatting {
                            font_name: None,
                            font_size: None,
                            bold: None,
                            italic: None,
                            underline: None,
                            color: None,
                        }),
                        alignment: Some(Alignment::Left),
                        is_heading: None,
                        heading_level: None,
                    },
                ],
                tables: None,
                images: None,
            },
        ],
    };
    
    let doc_id = word.create_document(tech_doc).await?;
    results.push(("Word".to_string(), doc_id));
    
    // Generate metrics spreadsheet
    let metrics_workbook = Workbook {
        title: "Monthly Metrics".to_string(),
        author: Some("Automated Report Generator".to_string()),
        worksheets: vec![
            Worksheet {
                name: "Summary".to_string(),
                columns: Some(vec![
                    Column { index: 0, width: Some(120.0), format: None },
                    Column { index: 1, width: Some(100.0), format: None },
                ]),
                rows: vec![
                    Row {
                        index: 0,
                        cells: vec![
                        Cell { value: CellValue::Text("Metric".to_string()), format: Some(CellFormat { bold: Some(true), font_name: None, font_size: None, italic: None, underline: None, color: None, background_color: None, number_format: None, alignment: None }) },
                        Cell { value: CellValue::Text("Value".to_string()), format: Some(CellFormat { bold: Some(true), font_name: None, font_size: None, italic: None, underline: None, color: None, background_color: None, number_format: None, alignment: None }) },
                        ],
                        height: None,
                    },
                    Row {
                        index: 1,
                        cells: vec![
                            Cell { value: CellValue::Text("Uptime".to_string()), format: None },
                        Cell { value: CellValue::Text("99.9%".to_string()), format: None },
                        ],
                        height: None,
                    },
                ],
                charts: None,
            },
        ],
    };
    
    let excel_id = excel.create_workbook(metrics_workbook).await?;
    results.push(("Excel".to_string(), excel_id));
    
    Ok(results)
}

// Helper function to create team-specific presentations
fn create_team_presentation(team: &str) -> Presentation {
    Presentation {
        title: format!("{} Team Status Update", team),
        author: Some("Team Lead".to_string()),
        theme: PresentationTheme::Technical,
        slides: vec![
            Slide {
                title: format!("{} Team Update", team),
                layout: SlideLayout::Title,
                subtitle: Some("Weekly Status Review".to_string()),
                content: None,
                bullets: None,
                image: None,
                notes: None,
            },
            Slide {
                title: "Key Achievements".to_string(),
                layout: SlideLayout::TitleAndContent,
                subtitle: None,
                content: Some(format!("Updates from the {} team this week.", team)),
                bullets: Some(vec![
                    BulletPoint {
                        text: "Completed sprint objectives".to_string(),
                        level: 0,
                        formatting: None,
                    },
                    BulletPoint {
                        text: "Resolved critical issues".to_string(),
                        level: 0,
                        formatting: None,
                    },
                    BulletPoint {
                        text: "Improved team processes".to_string(),
                        level: 0,
                        formatting: None,
                    },
                ]),
                image: None,
                notes: None,
            },
        ],
    }
}