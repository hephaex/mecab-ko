//! Dictionary quality analysis and statistics.
//!
//! This module provides advanced statistical analysis for dictionary entries,
//! including POS distribution, cost distribution, outlier detection, and recommendations.

#![allow(clippy::cast_precision_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_sign_loss)]

use crate::validator::DictEntry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dictionary analyzer for statistical analysis and quality metrics.
pub struct DictAnalyzer;

impl DictAnalyzer {
    /// Analyzes a collection of dictionary entries.
    #[must_use]
    pub fn analyze(entries: &[DictEntry]) -> AnalysisReport {
        let mut report = AnalysisReport::default();

        if entries.is_empty() {
            return report;
        }

        report.total_entries = entries.len();
        report.pos_distribution = Self::analyze_pos_distribution(entries);
        report.cost_distribution = Self::analyze_cost_distribution(entries);
        report.consistency_issues = Self::check_consistency(entries);
        report.recommendations = Self::generate_recommendations(&report);

        report
    }

    /// Analyzes POS tag distribution.
    fn analyze_pos_distribution(entries: &[DictEntry]) -> PosDistribution {
        let mut tag_counts: HashMap<String, usize> = HashMap::new();

        for entry in entries {
            *tag_counts.entry(entry.pos_tag.clone()).or_insert(0) += 1;
        }

        let total = entries.len();
        let mut tag_stats: Vec<PosTagStat> = tag_counts
            .into_iter()
            .map(|(tag, count)| {
                let percentage = (count as f64 / total as f64) * 100.0;
                PosTagStat {
                    tag,
                    count,
                    percentage,
                }
            })
            .collect();

        // Sort by count descending
        tag_stats.sort_by(|a, b| b.count.cmp(&a.count));

        PosDistribution { tags: tag_stats }
    }

    /// Analyzes cost distribution.
    fn analyze_cost_distribution(entries: &[DictEntry]) -> CostDistribution {
        let mut costs: Vec<i32> = entries.iter().map(|e| e.cost).collect();

        if costs.is_empty() {
            return CostDistribution::default();
        }

        costs.sort_unstable();

        let min = costs[0];
        let max = costs[costs.len() - 1];
        let mean = f64::from(costs.iter().sum::<i32>()) / costs.len() as f64;
        let median = Self::calculate_median(&costs);
        let std_dev = Self::calculate_std_dev(&costs, mean);
        let histogram = Self::generate_histogram(&costs, 20);
        let outliers = Self::detect_outliers(&costs, mean, std_dev);

        CostDistribution {
            min,
            max,
            mean,
            median,
            std_dev,
            histogram,
            outliers,
        }
    }

    /// Calculates median value.
    fn calculate_median(sorted_values: &[i32]) -> f64 {
        let len = sorted_values.len();
        if len == 0 {
            return 0.0;
        }

        if len % 2 == 0 {
            let mid1 = sorted_values[len / 2 - 1];
            let mid2 = sorted_values[len / 2];
            (f64::from(mid1) + f64::from(mid2)) / 2.0
        } else {
            f64::from(sorted_values[len / 2])
        }
    }

    /// Calculates standard deviation.
    fn calculate_std_dev(values: &[i32], mean: f64) -> f64 {
        if values.is_empty() {
            return 0.0;
        }

        let variance = values
            .iter()
            .map(|&v| {
                let diff = f64::from(v) - mean;
                diff * diff
            })
            .sum::<f64>()
            / values.len() as f64;

        variance.sqrt()
    }

    /// Generates histogram bins.
    fn generate_histogram(sorted_values: &[i32], num_bins: usize) -> Vec<HistogramBin> {
        if sorted_values.is_empty() || num_bins == 0 {
            return Vec::new();
        }

        let min = sorted_values[0];
        let max = sorted_values[sorted_values.len() - 1];

        if min == max {
            return vec![HistogramBin {
                range_start: min,
                range_end: max,
                count: sorted_values.len(),
            }];
        }

        let bin_width = f64::from(max - min) / num_bins as f64;
        let mut bins = vec![0; num_bins];

        for &value in sorted_values {
            let bin_index = ((f64::from(value - min) / bin_width) as usize).min(num_bins - 1);
            bins[bin_index] += 1;
        }

        (0..num_bins)
            .map(|i| {
                let range_start = min + (i as f64 * bin_width) as i32;
                let range_end = if i == num_bins - 1 {
                    max
                } else {
                    min + ((i + 1) as f64 * bin_width) as i32 - 1
                };

                HistogramBin {
                    range_start,
                    range_end,
                    count: bins[i],
                }
            })
            .filter(|bin| bin.count > 0)
            .collect()
    }

    /// Detects outlier values using IQR method.
    fn detect_outliers(sorted_values: &[i32], mean: f64, std_dev: f64) -> Vec<OutlierInfo> {
        if sorted_values.is_empty() {
            return Vec::new();
        }

        // Use 3 standard deviations as threshold
        let lower_bound = 3.0f64.mul_add(-std_dev, mean);
        let upper_bound = 3.0f64.mul_add(std_dev, mean);

        let mut outliers = Vec::new();

        for &value in sorted_values {
            let val_f64 = f64::from(value);
            if val_f64 < lower_bound || val_f64 > upper_bound {
                let deviation = ((val_f64 - mean) / std_dev).abs();
                outliers.push(OutlierInfo {
                    value,
                    deviation,
                    reason: if val_f64 < lower_bound {
                        format!("Below 3σ threshold ({lower_bound:.1})")
                    } else {
                        format!("Above 3σ threshold ({upper_bound:.1})")
                    },
                });
            }
        }

        // Limit to most extreme 50 outliers
        outliers.sort_by(|a, b| {
            b.deviation
                .partial_cmp(&a.deviation)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        outliers.truncate(50);

        outliers
    }

    /// Checks dictionary consistency.
    fn check_consistency(entries: &[DictEntry]) -> ConsistencyIssues {
        let mut issues = ConsistencyIssues::default();

        // Detect exact duplicates
        let mut seen_keys: HashMap<String, Vec<usize>> = HashMap::new();
        for entry in entries {
            let key = format!(
                "{}|{}|{}|{}|{}",
                entry.surface, entry.left_id, entry.right_id, entry.cost, entry.pos_tag
            );
            seen_keys.entry(key).or_default().push(entry.line_num);
        }

        issues.duplicate_entries = seen_keys
            .into_iter()
            .filter(|(_, lines)| lines.len() > 1)
            .count();

        // Check invalid POS tags (using Sejong tagset)
        let valid_tags = get_sejong_pos_tags();
        for entry in entries {
            if !entry.pos_tag.is_empty() && !valid_tags.contains(&entry.pos_tag.as_str()) {
                issues.invalid_pos_tags += 1;
            }
        }

        // Check ID validity (should be positive and within reasonable range)
        for entry in entries {
            if entry.left_id < 0 || entry.left_id > 10000 {
                issues.invalid_left_ids += 1;
            }
            if entry.right_id < 0 || entry.right_id > 10000 {
                issues.invalid_right_ids += 1;
            }
        }

        // Check for unusual cost values
        let unusual_threshold_low = -8000;
        let unusual_threshold_high = 8000;

        for entry in entries {
            if entry.cost < unusual_threshold_low || entry.cost > unusual_threshold_high {
                issues.unusual_cost_values += 1;
            }
        }

        issues
    }

    /// Generates recommendations based on analysis.
    fn generate_recommendations(report: &AnalysisReport) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // Check POS distribution balance
        if let Some(top_pos) = report.pos_distribution.tags.first() {
            if top_pos.percentage > 60.0 {
                recommendations.push(Recommendation {
                    category: "POS Distribution".to_string(),
                    message: format!(
                        "'{}' 품사가 전체의 {:.1}%를 차지합니다. 품사 분포가 불균형할 수 있습니다.",
                        top_pos.tag, top_pos.percentage
                    ),
                    severity: RecommendationSeverity::Warning,
                });
            }
        }

        // Check for rare POS tags
        let rare_threshold = 0.1; // Less than 0.1%
        let rare_pos: Vec<_> = report
            .pos_distribution
            .tags
            .iter()
            .filter(|s| s.percentage < rare_threshold && s.count > 0)
            .collect();

        if !rare_pos.is_empty() {
            recommendations.push(Recommendation {
                category: "POS Distribution".to_string(),
                message: format!(
                    "{}개의 품사가 전체의 {}% 미만입니다. 데이터 부족 가능성 검토가 필요합니다.",
                    rare_pos.len(),
                    rare_threshold
                ),
                severity: RecommendationSeverity::Info,
            });
        }

        // Check cost outliers
        if !report.cost_distribution.outliers.is_empty() {
            recommendations.push(Recommendation {
                category: "Cost Values".to_string(),
                message: format!(
                    "{}개의 비용 이상치가 발견되었습니다. 검토가 필요합니다.",
                    report.cost_distribution.outliers.len()
                ),
                severity: RecommendationSeverity::Warning,
            });
        }

        // Check consistency issues
        if report.consistency_issues.duplicate_entries > 0 {
            recommendations.push(Recommendation {
                category: "Consistency".to_string(),
                message: format!(
                    "{}개의 중복 엔트리가 발견되었습니다.",
                    report.consistency_issues.duplicate_entries
                ),
                severity: RecommendationSeverity::Error,
            });
        }

        if report.consistency_issues.invalid_pos_tags > 0 {
            recommendations.push(Recommendation {
                category: "Consistency".to_string(),
                message: format!(
                    "{}개의 유효하지 않은 품사 태그가 발견되었습니다 (세종 품사 태그 기준).",
                    report.consistency_issues.invalid_pos_tags
                ),
                severity: RecommendationSeverity::Error,
            });
        }

        if report.consistency_issues.unusual_cost_values > 0 {
            recommendations.push(Recommendation {
                category: "Cost Values".to_string(),
                message: format!(
                    "{}개의 비정상적인 비용 값이 발견되었습니다 (범위: -8000 ~ 8000).",
                    report.consistency_issues.unusual_cost_values
                ),
                severity: RecommendationSeverity::Warning,
            });
        }

        // Add positive recommendation if everything is good
        if recommendations.is_empty() {
            recommendations.push(Recommendation {
                category: "Overall".to_string(),
                message: "사전 품질이 양호합니다. 특별한 문제가 발견되지 않았습니다.".to_string(),
                severity: RecommendationSeverity::Info,
            });
        }

        recommendations
    }
}

/// Returns the set of valid Sejong POS tags.
fn get_sejong_pos_tags() -> Vec<&'static str> {
    vec![
        // 체언 (Nominals)
        "NNG", "NNP", "NNB", "NP", "NR",
        // 용언 (Predicates)
        "VV", "VA", "VX", "VCP", "VCN",
        // 관형사 (Determiners)
        "MM",
        // 부사 (Adverbs)
        "MAG", "MAJ",
        // 감탄사 (Interjections)
        "IC",
        // 조사 (Particles)
        "JKS", "JKC", "JKG", "JKO", "JKB", "JKV", "JKQ", "JX", "JC",
        // 선어말어미 (Pre-final endings)
        "EP",
        // 어말어미 (Final endings)
        "EF", "EC", "ETN", "ETM",
        // 접두사 (Prefixes)
        "XPN",
        // 접미사 (Suffixes)
        "XSN", "XSV", "XSA",
        // 어근 (Roots)
        "XR",
        // 부호 (Symbols)
        "SF", "SE", "SSO", "SSC", "SC", "SY",
        // 외국어 (Foreign words)
        "SL",
        // 한자 (Chinese characters)
        "SH",
        // 숫자 (Numbers)
        "SN",
        // 기타 (Others)
        "UNA", "NNBC", "NA", "NV", "NF",
    ]
}

/// Analysis report containing all statistical information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisReport {
    /// Total number of entries analyzed
    pub total_entries: usize,
    /// POS tag distribution
    pub pos_distribution: PosDistribution,
    /// Cost distribution
    pub cost_distribution: CostDistribution,
    /// Consistency issues
    pub consistency_issues: ConsistencyIssues,
    /// Recommendations for improvement
    pub recommendations: Vec<Recommendation>,
}

impl AnalysisReport {
    /// Formats the report as human-readable text.
    #[must_use]
    pub fn to_text(&self) -> String {
        use std::fmt::Write;
        let mut output = String::new();

        output.push_str("═══ 사전 품질 리포트 ═══\n\n");
        let _ = writeln!(output, "총 엔트리: {}", self.total_entries);
        let _ = writeln!(
            output,
            "중복 엔트리: {}",
            self.consistency_issues.duplicate_entries
        );
        let _ = writeln!(
            output,
            "유효하지 않은 품사: {}",
            self.consistency_issues.invalid_pos_tags
        );

        // POS distribution
        output.push_str("\n품사 분포:\n");
        for stat in self.pos_distribution.tags.iter().take(15) {
            let _ = writeln!(
                output,
                "  {:8} : {:>8} ({:>5.1}%)",
                stat.tag, stat.count, stat.percentage
            );
        }

        if self.pos_distribution.tags.len() > 15 {
            let _ = writeln!(
                output,
                "  ... ({} more)",
                self.pos_distribution.tags.len() - 15
            );
        }

        // Cost distribution
        output.push_str("\n비용 분포:\n");
        let _ = writeln!(
            output,
            "  범위       : {} ~ {}",
            self.cost_distribution.min, self.cost_distribution.max
        );
        let _ = writeln!(output, "  평균       : {:.1}", self.cost_distribution.mean);
        let _ = writeln!(
            output,
            "  중앙값     : {:.1}",
            self.cost_distribution.median
        );
        let _ = writeln!(
            output,
            "  표준편차   : {:.1}",
            self.cost_distribution.std_dev
        );
        let _ = writeln!(
            output,
            "  이상치     : {}개",
            self.cost_distribution.outliers.len()
        );

        // Show histogram if available
        if !self.cost_distribution.histogram.is_empty() {
            output.push_str("\n비용 히스토그램 (상위 10개 구간):\n");
            let mut hist = self.cost_distribution.histogram.clone();
            hist.sort_by(|a, b| b.count.cmp(&a.count));
            for bin in hist.iter().take(10) {
                let bar = "█".repeat((bin.count as f64 / 100.0).min(50.0) as usize);
                let _ = writeln!(
                    output,
                    "  [{:>6} ~ {:>6}]: {:>6} {}",
                    bin.range_start, bin.range_end, bin.count, bar
                );
            }
        }

        // Recommendations
        if !self.recommendations.is_empty() {
            output.push_str("\n권장 사항:\n");
            for rec in &self.recommendations {
                let prefix = match rec.severity {
                    RecommendationSeverity::Error => "  ❌",
                    RecommendationSeverity::Warning => "  ⚠️ ",
                    RecommendationSeverity::Info => "  ℹ️ ",
                };
                let _ = writeln!(output, "{} [{}] {}", prefix, rec.category, rec.message);
            }
        }

        output
    }
}

/// POS tag distribution information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PosDistribution {
    /// Statistics for each POS tag
    pub tags: Vec<PosTagStat>,
}

/// Statistics for a single POS tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PosTagStat {
    /// POS tag
    pub tag: String,
    /// Number of occurrences
    pub count: usize,
    /// Percentage of total entries
    pub percentage: f64,
}

/// Cost distribution information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostDistribution {
    /// Minimum cost value
    pub min: i32,
    /// Maximum cost value
    pub max: i32,
    /// Mean cost value
    pub mean: f64,
    /// Median cost value
    pub median: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Histogram bins
    pub histogram: Vec<HistogramBin>,
    /// Detected outliers
    pub outliers: Vec<OutlierInfo>,
}

/// A histogram bin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramBin {
    /// Start of range (inclusive)
    pub range_start: i32,
    /// End of range (inclusive)
    pub range_end: i32,
    /// Count of values in this range
    pub count: usize,
}

/// Information about an outlier value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlierInfo {
    /// The outlier value
    pub value: i32,
    /// Number of standard deviations from mean
    pub deviation: f64,
    /// Reason for being an outlier
    pub reason: String,
}

/// Consistency issues found in the dictionary.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsistencyIssues {
    /// Number of duplicate entries
    pub duplicate_entries: usize,
    /// Number of invalid POS tags
    pub invalid_pos_tags: usize,
    /// Number of invalid left context IDs
    pub invalid_left_ids: usize,
    /// Number of invalid right context IDs
    pub invalid_right_ids: usize,
    /// Number of unusual cost values
    pub unusual_cost_values: usize,
}

/// A recommendation for improving dictionary quality.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation category
    pub category: String,
    /// Recommendation message
    pub message: String,
    /// Severity level
    pub severity: RecommendationSeverity,
}

/// Severity level for recommendations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecommendationSeverity {
    /// Informational
    Info,
    /// Warning
    Warning,
    /// Error (should be fixed)
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_entries() -> Vec<DictEntry> {
        vec![
            DictEntry {
                surface: "테스트".to_string(),
                left_id: 100,
                right_id: 200,
                cost: 500,
                pos_tag: "NNG".to_string(),
                features: vec![],
                line_num: 1,
            },
            DictEntry {
                surface: "분석".to_string(),
                left_id: 101,
                right_id: 201,
                cost: 600,
                pos_tag: "NNG".to_string(),
                features: vec![],
                line_num: 2,
            },
            DictEntry {
                surface: "하다".to_string(),
                left_id: 102,
                right_id: 202,
                cost: 300,
                pos_tag: "VV".to_string(),
                features: vec![],
                line_num: 3,
            },
        ]
    }

    #[test]
    fn test_analyze_basic() {
        let entries = create_test_entries();
        let report = DictAnalyzer::analyze(&entries);

        assert_eq!(report.total_entries, 3);
        assert!(!report.pos_distribution.tags.is_empty());
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_pos_distribution() {
        let entries = create_test_entries();
        let dist = DictAnalyzer::analyze_pos_distribution(&entries);

        assert_eq!(dist.tags.len(), 2); // NNG and VV
        let nng_stat = dist.tags.iter().find(|s| s.tag == "NNG").unwrap();
        assert_eq!(nng_stat.count, 2);
        assert!((nng_stat.percentage - 66.67).abs() < 0.1);
    }

    #[test]
    fn test_cost_distribution() {
        let entries = create_test_entries();
        let dist = DictAnalyzer::analyze_cost_distribution(&entries);

        assert_eq!(dist.min, 300);
        assert_eq!(dist.max, 600);
        assert!((dist.mean - 466.67).abs() < 0.1);
        assert!((dist.median - 500.0).abs() < 0.1);
    }

    #[test]
    fn test_median_calculation() {
        let odd = vec![1, 2, 3, 4, 5];
        assert_eq!(DictAnalyzer::calculate_median(&odd), 3.0);

        let even = vec![1, 2, 3, 4];
        assert_eq!(DictAnalyzer::calculate_median(&even), 2.5);

        let empty: Vec<i32> = vec![];
        assert_eq!(DictAnalyzer::calculate_median(&empty), 0.0);
    }

    #[test]
    fn test_std_dev_calculation() {
        let values = vec![2, 4, 4, 4, 5, 5, 7, 9];
        let mean = values.iter().sum::<i32>() as f64 / values.len() as f64;
        let std_dev = DictAnalyzer::calculate_std_dev(&values, mean);
        assert!((std_dev - 2.0).abs() < 0.1);
    }

    #[test]
    fn test_histogram_generation() {
        let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let hist = DictAnalyzer::generate_histogram(&values, 5);
        assert!(!hist.is_empty());
        assert!(hist.len() <= 5);
    }

    #[test]
    fn test_outlier_detection() {
        let mut values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        values.push(1000); // Clear outlier
        values.sort_unstable();

        let mean = values.iter().sum::<i32>() as f64 / values.len() as f64;
        let std_dev = DictAnalyzer::calculate_std_dev(&values, mean);
        let outliers = DictAnalyzer::detect_outliers(&values, mean, std_dev);

        assert!(!outliers.is_empty());
        assert!(outliers.iter().any(|o| o.value == 1000));
    }

    #[test]
    fn test_consistency_check() {
        let mut entries = create_test_entries();
        // Add a duplicate
        entries.push(entries[0].clone());

        let issues = DictAnalyzer::check_consistency(&entries);
        assert_eq!(issues.duplicate_entries, 1);
        assert_eq!(issues.invalid_pos_tags, 0);
    }

    #[test]
    fn test_invalid_pos_tags() {
        let entries = vec![DictEntry {
            surface: "테스트".to_string(),
            left_id: 100,
            right_id: 200,
            cost: 500,
            pos_tag: "INVALID".to_string(),
            features: vec![],
            line_num: 1,
        }];

        let issues = DictAnalyzer::check_consistency(&entries);
        assert_eq!(issues.invalid_pos_tags, 1);
    }
}
