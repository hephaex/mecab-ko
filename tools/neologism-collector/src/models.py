"""Database models for neologism collector."""

from __future__ import annotations

from datetime import datetime
from typing import Any

from sqlalchemy import (
    JSON,
    Boolean,
    DateTime,
    Float,
    Integer,
    String,
    Text,
    UniqueConstraint,
    func,
)
from sqlalchemy.orm import DeclarativeBase, Mapped, mapped_column


class Base(DeclarativeBase):
    """Base class for all database models."""

    pass


class RawText(Base):
    """Raw text collected from sources."""

    __tablename__ = "raw_texts"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    source: Mapped[str] = mapped_column(String(50), nullable=False, index=True)
    url: Mapped[str | None] = mapped_column(String(500))
    title: Mapped[str | None] = mapped_column(String(500))
    content: Mapped[str] = mapped_column(Text, nullable=False)
    metadata: Mapped[dict[str, Any] | None] = mapped_column(JSON)
    collected_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, default=func.now(), index=True
    )
    processed: Mapped[bool] = mapped_column(Boolean, default=False, index=True)
    processed_at: Mapped[datetime | None] = mapped_column(DateTime)

    def __repr__(self) -> str:
        """Return string representation."""
        return (
            f"<RawText(id={self.id}, source={self.source}, "
            f"collected_at={self.collected_at})>"
        )


class CandidateWord(Base):
    """Candidate neologism extracted from text."""

    __tablename__ = "candidate_words"
    __table_args__ = (UniqueConstraint("surface", name="uix_surface"),)

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    surface: Mapped[str] = mapped_column(String(100), nullable=False, unique=True)
    frequency: Mapped[int] = mapped_column(Integer, default=1, nullable=False)
    first_seen: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, default=func.now()
    )
    last_seen: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, default=func.now()
    )
    contexts: Mapped[list[str]] = mapped_column(JSON, default=list)
    sources: Mapped[list[str]] = mapped_column(JSON, default=list)

    # Classification status
    is_neologism: Mapped[bool | None] = mapped_column(Boolean, index=True)
    is_typo: Mapped[bool | None] = mapped_column(Boolean)
    confidence_score: Mapped[float | None] = mapped_column(Float)

    # Features for classification
    length: Mapped[int] = mapped_column(Integer, nullable=False)
    has_hangul: Mapped[bool] = mapped_column(Boolean, default=True)
    has_english: Mapped[bool] = mapped_column(Boolean, default=False)
    has_number: Mapped[bool] = mapped_column(Boolean, default=False)
    has_special: Mapped[bool] = mapped_column(Boolean, default=False)

    # Additional metadata
    metadata: Mapped[dict[str, Any] | None] = mapped_column(JSON)
    created_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, default=func.now()
    )
    updated_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, default=func.now(), onupdate=func.now()
    )

    def __repr__(self) -> str:
        """Return string representation."""
        return (
            f"<CandidateWord(surface={self.surface}, frequency={self.frequency}, "
            f"is_neologism={self.is_neologism})>"
        )


class Neologism(Base):
    """Confirmed neologism with scores and metadata."""

    __tablename__ = "neologisms"
    __table_args__ = (UniqueConstraint("surface", name="uix_neologism_surface"),)

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    surface: Mapped[str] = mapped_column(String(100), nullable=False, unique=True)

    # Scores
    total_score: Mapped[float] = mapped_column(Float, nullable=False, index=True)
    frequency_score: Mapped[float] = mapped_column(Float, default=0.0)
    context_diversity_score: Mapped[float] = mapped_column(Float, default=0.0)
    morphological_score: Mapped[float] = mapped_column(Float, default=0.0)
    social_spread_score: Mapped[float] = mapped_column(Float, default=0.0)
    temporal_trend_score: Mapped[float] = mapped_column(Float, default=0.0)

    # Statistics
    frequency: Mapped[int] = mapped_column(Integer, default=1)
    context_count: Mapped[int] = mapped_column(Integer, default=0)
    source_count: Mapped[int] = mapped_column(Integer, default=0)

    # MeCab dictionary fields
    pos: Mapped[str] = mapped_column(String(20), default="NNG")
    semantic1: Mapped[str | None] = mapped_column(String(20))
    semantic2: Mapped[str | None] = mapped_column(String(20))
    reading: Mapped[str | None] = mapped_column(String(100))
    cost: Mapped[int] = mapped_column(Integer, default=5000)

    # Status
    confidence_level: Mapped[str] = mapped_column(
        String(20), nullable=False, index=True
    )  # high, medium, low
    exported: Mapped[bool] = mapped_column(Boolean, default=False, index=True)
    exported_at: Mapped[datetime | None] = mapped_column(DateTime)

    # Timestamps
    first_detected: Mapped[datetime] = mapped_column(DateTime, nullable=False)
    last_updated: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, default=func.now(), onupdate=func.now()
    )
    created_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, default=func.now()
    )

    # Additional data
    examples: Mapped[list[str]] = mapped_column(JSON, default=list)
    metadata: Mapped[dict[str, Any] | None] = mapped_column(JSON)

    def __repr__(self) -> str:
        """Return string representation."""
        return (
            f"<Neologism(surface={self.surface}, total_score={self.total_score:.2f}, "
            f"confidence={self.confidence_level})>"
        )


class CollectionJob(Base):
    """Collection job execution history."""

    __tablename__ = "collection_jobs"

    id: Mapped[int] = mapped_column(Integer, primary_key=True, autoincrement=True)
    job_name: Mapped[str] = mapped_column(String(100), nullable=False, index=True)
    source: Mapped[str] = mapped_column(String(50), nullable=False)
    started_at: Mapped[datetime] = mapped_column(DateTime, nullable=False)
    completed_at: Mapped[datetime | None] = mapped_column(DateTime)
    status: Mapped[str] = mapped_column(
        String(20), nullable=False
    )  # running, completed, failed
    items_collected: Mapped[int] = mapped_column(Integer, default=0)
    error_message: Mapped[str | None] = mapped_column(Text)
    metadata: Mapped[dict[str, Any] | None] = mapped_column(JSON)

    def __repr__(self) -> str:
        """Return string representation."""
        return (
            f"<CollectionJob(job_name={self.job_name}, source={self.source}, "
            f"status={self.status})>"
        )
