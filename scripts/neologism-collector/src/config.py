"""Configuration management for neologism collector."""

from __future__ import annotations

from pathlib import Path
from typing import Any

import yaml
from loguru import logger
from pydantic import Field, field_validator
from pydantic_settings import BaseSettings, SettingsConfigDict


class DatabaseConfig(BaseSettings):
    """Database configuration."""

    url: str = "sqlite:///data/neologisms.db"
    echo: bool = False
    pool_size: int = 5
    max_overflow: int = 10


class RateLimitConfig(BaseSettings):
    """Rate limiting configuration."""

    requests_per_second: float = 1.0
    burst: int = 3


class TimeoutConfig(BaseSettings):
    """Timeout configuration."""

    connect: int = 10
    read: int = 30


class RetryConfig(BaseSettings):
    """Retry configuration."""

    max_attempts: int = 3
    backoff_factor: float = 2.0


class CrawlerConfig(BaseSettings):
    """Crawler configuration."""

    user_agent: str = "MeCab-Ko Neologism Collector"
    rate_limit: RateLimitConfig = Field(default_factory=RateLimitConfig)
    timeout: TimeoutConfig = Field(default_factory=TimeoutConfig)
    retry: RetryConfig = Field(default_factory=RetryConfig)
    respect_robots_txt: bool = True


class DetectorConfig(BaseSettings):
    """Detector configuration."""

    min_frequency: int = 5
    min_length: int = 2
    max_length: int = 20
    allow_mixed_script: bool = True
    allow_special_chars: bool = False
    thresholds: dict[str, int | float] = Field(
        default_factory=lambda: {
            "min_occurrences": 10,
            "tfidf_threshold": 0.1,
            "min_context_diversity": 3,
        }
    )


class ClassifierConfig(BaseSettings):
    """Classifier configuration."""

    features: list[str] = Field(
        default_factory=lambda: [
            "length",
            "char_type_ratio",
            "keyboard_distance",
            "phonetic_similarity",
            "context_coherence",
        ]
    )
    weights: dict[str, float] = Field(
        default_factory=lambda: {
            "frequency": 0.3,
            "context_diversity": 0.25,
            "morphological_validity": 0.2,
            "social_spread": 0.15,
            "temporal_trend": 0.1,
        }
    )


class ScorerConfig(BaseSettings):
    """Scorer configuration."""

    algorithm: str = "weighted_sum"
    thresholds: dict[str, float] = Field(
        default_factory=lambda: {
            "high_confidence": 0.8,
            "medium_confidence": 0.5,
            "low_confidence": 0.3,
        }
    )
    temporal_decay_days: int = 30


class StorageConfig(BaseSettings):
    """Storage configuration."""

    retention_days: int = 365
    batch_size: int = 1000
    compress: bool = True


class ExporterConfig(BaseSettings):
    """Exporter configuration."""

    output_dir: str = "data/export"
    default_pos: str = "NNG"
    cost_base: int = 5000
    cost_adjustment_factor: int = 100


class SchedulerConfig(BaseSettings):
    """Scheduler configuration."""

    enabled: bool = True
    schedules: list[dict[str, Any]] = Field(default_factory=list)


class LoggingConfig(BaseSettings):
    """Logging configuration."""

    level: str = "INFO"
    format: str = (
        "{time:YYYY-MM-DD HH:mm:ss} | {level: <8} | "
        "{name}:{function}:{line} - {message}"
    )
    files: dict[str, str] = Field(
        default_factory=lambda: {
            "main": "logs/neologism_collector.log",
            "crawler": "logs/crawler.log",
            "detector": "logs/detector.log",
        }
    )
    rotation: str = "100 MB"
    retention: str = "30 days"
    compression: str = "zip"


class Config(BaseSettings):
    """Main configuration class."""

    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        env_nested_delimiter="__",
        extra="ignore",
    )

    database: DatabaseConfig = Field(default_factory=DatabaseConfig)
    crawler: CrawlerConfig = Field(default_factory=CrawlerConfig)
    detector: DetectorConfig = Field(default_factory=DetectorConfig)
    classifier: ClassifierConfig = Field(default_factory=ClassifierConfig)
    scorer: ScorerConfig = Field(default_factory=ScorerConfig)
    storage: StorageConfig = Field(default_factory=StorageConfig)
    exporter: ExporterConfig = Field(default_factory=ExporterConfig)
    scheduler: SchedulerConfig = Field(default_factory=SchedulerConfig)
    logging: LoggingConfig = Field(default_factory=LoggingConfig)
    sources: dict[str, Any] = Field(default_factory=dict)

    @field_validator("database", mode="before")
    @classmethod
    def validate_database(cls, v: dict[str, Any] | DatabaseConfig) -> DatabaseConfig:
        """Validate database configuration."""
        if isinstance(v, dict):
            return DatabaseConfig(**v)
        return v

    @field_validator("crawler", mode="before")
    @classmethod
    def validate_crawler(cls, v: dict[str, Any] | CrawlerConfig) -> CrawlerConfig:
        """Validate crawler configuration."""
        if isinstance(v, dict):
            rate_limit = v.pop("rate_limit", {})
            timeout = v.pop("timeout", {})
            retry = v.pop("retry", {})
            return CrawlerConfig(
                **v,
                rate_limit=RateLimitConfig(**rate_limit),
                timeout=TimeoutConfig(**timeout),
                retry=RetryConfig(**retry),
            )
        return v

    @classmethod
    def load_from_yaml(cls, config_path: str | Path) -> Config:
        """Load configuration from YAML file.

        Args:
            config_path: Path to configuration YAML file.

        Returns:
            Loaded configuration object.

        Raises:
            FileNotFoundError: If configuration file doesn't exist.
            yaml.YAMLError: If YAML parsing fails.
        """
        config_file = Path(config_path)
        if not config_file.exists():
            msg = f"Configuration file not found: {config_path}"
            raise FileNotFoundError(msg)

        logger.info(f"Loading configuration from {config_path}")
        with config_file.open(encoding="utf-8") as f:
            config_data = yaml.safe_load(f)

        return cls(**config_data)


# Global configuration instance
_config: Config | None = None


def get_config(config_path: str | Path | None = None) -> Config:
    """Get global configuration instance.

    Args:
        config_path: Optional path to configuration file.
                    If None, uses default config.yaml.

    Returns:
        Configuration instance.
    """
    global _config

    if _config is None:
        if config_path is None:
            config_path = Path(__file__).parent.parent / "config.yaml"

        _config = Config.load_from_yaml(config_path)

    return _config


def reset_config() -> None:
    """Reset global configuration instance (mainly for testing)."""
    global _config
    _config = None
