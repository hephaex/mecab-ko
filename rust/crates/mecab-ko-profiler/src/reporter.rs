//! Report generation for memory profiling results.
//!
//! This module provides multiple output formats for profiling data including
//! JSON, human-readable text, and optional flamegraph generation.

use crate::stats::{ComponentStats, DetailedStats};
use humansize::{format_size, BINARY};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fmt::Write as _;
use std::io::Write;
use tabled::{settings::Style, Table, Tabled};

/// Profiling report containing all collected data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfilingReport {
    /// Report metadata.
    pub metadata: ReportMetadata,
    /// Detailed statistics.
    pub stats: DetailedStats,
    /// Analysis and recommendations.
    pub analysis: Analysis,
}

impl ProfilingReport {
    /// Creates a new profiling report.
    #[must_use]
    pub fn new(stats: DetailedStats) -> Self {
        let analysis = Analysis::from_stats(&stats);
        Self {
            metadata: ReportMetadata::default(),
            stats,
            analysis,
        }
    }

    /// Exports the report as JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Exports the report as human-readable text.
    #[must_use]
    pub fn to_text(&self) -> String {
        let mut output = String::new();

        // Header
        output.push_str("═══════════════════════════════════════════════════════════\n");
        output.push_str("           MeCab-Ko Memory Profiling Report\n");
        output.push_str("═══════════════════════════════════════════════════════════\n\n");

        // Metadata
        let _ = writeln!(output, "Generated: {}", self.metadata.timestamp);
        let _ = writeln!(output, "Version: {}\n", self.metadata.version);

        // Overall statistics
        output.push_str("Overall Statistics:\n");
        output.push_str("───────────────────────────────────────────────────────────\n");
        output.push_str(&self.format_overall_stats());
        output.push_str("\n\n");

        // Component breakdown
        output.push_str("Component Breakdown:\n");
        output.push_str("───────────────────────────────────────────────────────────\n");
        output.push_str(&self.format_component_table());
        output.push_str("\n\n");

        // Top consumers
        let top = self.stats.top_components(5);
        if !top.is_empty() {
            output.push_str("Top Memory Consumers:\n");
            output.push_str("───────────────────────────────────────────────────────────\n");
            for (i, (name, stats)) in top.iter().enumerate() {
                let _ = writeln!(
                    output,
                    "{}. {} - {} (peak: {})",
                    i + 1,
                    name,
                    format_size(stats.current_usage, BINARY),
                    format_size(stats.peak_usage, BINARY)
                );
            }
            output.push_str("\n\n");
        }

        // Analysis
        output.push_str("Analysis & Recommendations:\n");
        output.push_str("───────────────────────────────────────────────────────────\n");
        output.push_str(&self.analysis.to_string());
        output.push('\n');

        output.push_str("═══════════════════════════════════════════════════════════\n");

        output
    }

    /// Writes the report to a writer in the specified format.
    ///
    /// # Errors
    ///
    /// Returns an error if writing fails or serialization fails.
    pub fn write_to<W: Write>(
        &self,
        writer: &mut W,
        format: ReportFormat,
    ) -> Result<(), ReportError> {
        match format {
            ReportFormat::Json => {
                let json = self.to_json()?;
                writer.write_all(json.as_bytes())?;
            }
            ReportFormat::Text => {
                let text = self.to_text();
                writer.write_all(text.as_bytes())?;
            }
        }
        Ok(())
    }

    fn format_overall_stats(&self) -> String {
        let overall = &self.stats.overall;
        format!(
            "  Total Allocated:  {}\n\
             Total Deallocated: {}\n\
             Current Usage:     {}\n\
             Peak Usage:        {}\n\
             Components:        {}\n\
             Efficiency:        {:.1}%",
            format_size(overall.total_allocated, BINARY),
            format_size(overall.total_deallocated, BINARY),
            format_size(overall.current_usage, BINARY),
            format_size(overall.peak_usage, BINARY),
            overall.component_count,
            overall.efficiency() * 100.0
        )
    }

    fn format_component_table(&self) -> String {
        if self.stats.components.is_empty() {
            return "  No component data available\n".to_string();
        }

        let rows: Vec<ComponentRow> = self
            .stats
            .components
            .iter()
            .map(|(name, stats)| ComponentRow::from_stats(name, stats))
            .collect();

        let table = Table::new(rows).with(Style::rounded()).to_string();
        table
    }
}

/// Report metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Report generation timestamp.
    pub timestamp: String,
    /// Tool version.
    pub version: String,
    /// Target platform.
    pub platform: String,
}

impl Default for ReportMetadata {
    fn default() -> Self {
        Self {
            timestamp: chrono::Utc::now().to_rfc3339(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: std::env::consts::OS.to_string(),
        }
    }
}

/// Report output format.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// JSON format.
    Json,
    /// Human-readable text format.
    Text,
}

impl ReportFormat {
    /// Parses a format from a string.
    #[must_use]
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "json" => Some(Self::Json),
            "text" | "txt" => Some(Self::Text),
            _ => None,
        }
    }
}

/// Analysis and recommendations based on profiling data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Analysis {
    /// Memory efficiency score (0-100).
    pub efficiency_score: f64,
    /// Fragmentation indicators per component.
    pub fragmentation: Vec<(String, f64)>,
    /// Recommendations for optimization.
    pub recommendations: Vec<String>,
    /// Warnings and issues detected.
    pub warnings: Vec<String>,
}

impl Analysis {
    /// Generates analysis from detailed statistics.
    #[must_use]
    pub fn from_stats(stats: &DetailedStats) -> Self {
        let efficiency_score = stats.overall.efficiency() * 100.0;

        let mut fragmentation: Vec<_> = stats
            .components
            .iter()
            .map(|(name, comp)| (name.clone(), comp.fragmentation_score()))
            .collect();
        fragmentation.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        let mut recommendations = Vec::new();
        let mut warnings = Vec::new();

        // Analyze efficiency
        if efficiency_score < 50.0 {
            warnings.push(format!(
                "Low memory efficiency ({efficiency_score:.1}%). Consider reducing peak allocations."
            ));
            recommendations
                .push("Review component initialization to reduce peak memory usage".to_string());
        }

        // Analyze fragmentation
        for (name, score) in &fragmentation {
            if *score > 0.5 {
                warnings.push(format!(
                    "High fragmentation in component '{name}' ({score:.2})"
                ));
                recommendations.push(format!("Consider using an arena allocator for '{name}'"));
            }
        }

        // Analyze allocation patterns
        for (name, comp) in &stats.components {
            let active = comp.allocations.saturating_sub(comp.deallocations);
            if active > 10000 {
                recommendations.push(format!(
                    "Component '{name}' has {active} active allocations. Consider batch allocation."
                ));
            }

            if comp.avg_allocation_size < 64 {
                recommendations.push(format!(
                    "Component '{name}' has small average allocation size ({}B). Consider pooling.",
                    comp.avg_allocation_size
                ));
            }
        }

        Self {
            efficiency_score,
            fragmentation,
            recommendations,
            warnings,
        }
    }
}

impl fmt::Display for Analysis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  Efficiency Score: {:.1}%", self.efficiency_score)?;

        if !self.warnings.is_empty() {
            writeln!(f, "\n  Warnings:")?;
            for warning in &self.warnings {
                writeln!(f, "    ⚠ {warning}")?;
            }
        }

        if !self.recommendations.is_empty() {
            writeln!(f, "\n  Recommendations:")?;
            for (i, rec) in self.recommendations.iter().enumerate() {
                writeln!(f, "    {}. {rec}", i + 1)?;
            }
        }

        Ok(())
    }
}

/// Table row for component statistics.
#[derive(Debug, Tabled)]
struct ComponentRow {
    #[tabled(rename = "Component")]
    name: String,
    #[tabled(rename = "Allocations")]
    allocations: usize,
    #[tabled(rename = "Current")]
    current: String,
    #[tabled(rename = "Peak")]
    peak: String,
    #[tabled(rename = "Avg Size")]
    avg_size: String,
    #[tabled(rename = "Efficiency")]
    efficiency: String,
}

impl ComponentRow {
    fn from_stats(name: &str, stats: &ComponentStats) -> Self {
        Self {
            name: name.to_string(),
            allocations: stats.allocations,
            current: format_size(stats.current_usage, BINARY),
            peak: format_size(stats.peak_usage, BINARY),
            avg_size: format_size(stats.avg_allocation_size, BINARY),
            efficiency: format!("{:.1}%", stats.efficiency() * 100.0),
        }
    }
}

/// Errors that can occur during report generation.
#[derive(Debug, thiserror::Error)]
pub enum ReportError {
    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error.
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::allocator::MemorySnapshot;

    fn create_test_stats() -> DetailedStats {
        let mut stats = DetailedStats::new();

        let snapshot = MemorySnapshot {
            allocations: 100,
            deallocations: 50,
            total_allocated: 10000,
            total_deallocated: 5000,
            current_usage: 5000,
            peak_usage: 8000,
        };

        stats.add_component(
            "test_component",
            ComponentStats::from_snapshot("test_component", snapshot),
        );
        stats.compute_overall();
        stats
    }

    #[test]
    fn test_report_json() {
        let stats = create_test_stats();
        let report = ProfilingReport::new(stats);

        let json = report.to_json();
        assert!(json.is_ok());
        let json_str = json.unwrap();
        assert!(json_str.contains("test_component"));
    }

    #[test]
    fn test_report_text() {
        let stats = create_test_stats();
        let report = ProfilingReport::new(stats);

        let text = report.to_text();
        assert!(text.contains("Memory Profiling Report"));
        assert!(text.contains("test_component"));
        assert!(text.contains("Overall Statistics"));
    }

    #[test]
    fn test_analysis() {
        let stats = create_test_stats();
        let analysis = Analysis::from_stats(&stats);

        assert!(analysis.efficiency_score > 0.0);
        assert!(analysis.efficiency_score <= 100.0);
    }

    #[test]
    fn test_report_format_parsing() {
        assert_eq!(ReportFormat::parse("json"), Some(ReportFormat::Json));
        assert_eq!(ReportFormat::parse("text"), Some(ReportFormat::Text));
        assert_eq!(ReportFormat::parse("txt"), Some(ReportFormat::Text));
        assert_eq!(ReportFormat::parse("invalid"), None);
    }
}
