"""Database storage management for neologism collector."""

from __future__ import annotations

from datetime import datetime, timedelta
from pathlib import Path
from typing import Any

from loguru import logger
from sqlalchemy import create_engine, func
from sqlalchemy.orm import Session, sessionmaker

from .config import Config, get_config
from .models import Base, CandidateWord, CollectionJob, Neologism, RawText


class NeologismStorage:
    """Storage manager for neologism data."""

    def __init__(self, config: Config | None = None) -> None:
        """Initialize storage manager.

        Args:
            config: Configuration object. If None, uses global config.
        """
        self.config = config or get_config()
        self.storage_config = self.config.storage

        # Create database engine
        db_url = self.config.database.url

        # Ensure database directory exists for SQLite
        if db_url.startswith("sqlite:///"):
            db_path = db_url.replace("sqlite:///", "")
            Path(db_path).parent.mkdir(parents=True, exist_ok=True)

        self.engine = create_engine(
            db_url,
            echo=self.config.database.echo,
            pool_size=self.config.database.pool_size,
            max_overflow=self.config.database.max_overflow,
        )

        # Create session factory
        self.SessionLocal = sessionmaker(
            autocommit=False,
            autoflush=False,
            bind=self.engine,
        )

        logger.info(f"Database initialized: {db_url}")

    def initialize_database(self) -> None:
        """Create all database tables."""
        Base.metadata.create_all(bind=self.engine)
        logger.info("Database tables created")

    def get_session(self) -> Session:
        """Get a new database session.

        Returns:
            SQLAlchemy session.
        """
        return self.SessionLocal()

    def save_raw_texts(self, texts: list[dict[str, Any]]) -> int:
        """Save raw texts to database.

        Args:
            texts: List of text dictionaries with source, content, etc.

        Returns:
            Number of texts saved.
        """
        if not texts:
            return 0

        session = self.get_session()
        saved_count = 0

        try:
            batch_size = self.storage_config.batch_size

            for i in range(0, len(texts), batch_size):
                batch = texts[i : i + batch_size]

                for text_data in batch:
                    raw_text = RawText(
                        source=text_data.get("source", "unknown"),
                        url=text_data.get("url"),
                        title=text_data.get("title"),
                        content=text_data["content"],
                        metadata=text_data.get("metadata"),
                        collected_at=text_data.get("collected_at", datetime.now()),
                    )
                    session.add(raw_text)
                    saved_count += 1

                session.commit()
                logger.info(f"Saved batch of {len(batch)} texts")

        except Exception as e:
            logger.error(f"Failed to save raw texts: {e}")
            session.rollback()
            raise

        finally:
            session.close()

        return saved_count

    def create_collection_job(
        self,
        job_name: str,
        source: str,
    ) -> CollectionJob:
        """Create a new collection job record.

        Args:
            job_name: Name of the collection job.
            source: Source being collected from.

        Returns:
            Created CollectionJob instance.
        """
        session = self.get_session()

        try:
            job = CollectionJob(
                job_name=job_name,
                source=source,
                started_at=datetime.now(),
                status="running",
            )
            session.add(job)
            session.commit()
            session.refresh(job)

            logger.info(f"Created collection job: {job_name} (ID: {job.id})")
            return job

        finally:
            session.close()

    def update_collection_job(
        self,
        job_id: int,
        status: str,
        items_collected: int | None = None,
        error_message: str | None = None,
        metadata: dict[str, Any] | None = None,
    ) -> None:
        """Update collection job status.

        Args:
            job_id: Job ID to update.
            status: New status (running, completed, failed).
            items_collected: Number of items collected.
            error_message: Error message if failed.
            metadata: Additional metadata.
        """
        session = self.get_session()

        try:
            job = session.query(CollectionJob).filter(CollectionJob.id == job_id).first()

            if not job:
                logger.warning(f"Collection job {job_id} not found")
                return

            job.status = status
            job.completed_at = datetime.now()

            if items_collected is not None:
                job.items_collected = items_collected

            if error_message is not None:
                job.error_message = error_message

            if metadata is not None:
                job.metadata = metadata

            session.commit()
            logger.info(f"Updated collection job {job_id}: {status}")

        finally:
            session.close()

    def cleanup_old_data(self) -> dict[str, int]:
        """Clean up old data based on retention policy.

        Returns:
            Dictionary with counts of deleted items.
        """
        session = self.get_session()
        deleted_counts = {
            "raw_texts": 0,
            "candidate_words": 0,
        }

        try:
            retention_days = self.storage_config.retention_days
            cutoff_date = datetime.now() - timedelta(days=retention_days)

            logger.info(f"Cleaning up data older than {cutoff_date}")

            # Delete old processed raw texts
            deleted_raw = (
                session.query(RawText)
                .filter(
                    RawText.processed == True,  # noqa: E712
                    RawText.collected_at < cutoff_date,
                )
                .delete()
            )
            deleted_counts["raw_texts"] = deleted_raw

            # Delete old rejected candidates (typos, low confidence)
            deleted_candidates = (
                session.query(CandidateWord)
                .filter(
                    CandidateWord.is_typo == True,  # noqa: E712
                    CandidateWord.created_at < cutoff_date,
                )
                .delete()
            )
            deleted_counts["candidate_words"] = deleted_candidates

            session.commit()

            logger.info(f"Cleanup complete: {deleted_counts}")

        except Exception as e:
            logger.error(f"Failed to cleanup old data: {e}")
            session.rollback()
            raise

        finally:
            session.close()

        return deleted_counts

    def get_statistics(self) -> dict[str, Any]:
        """Get database statistics.

        Returns:
            Dictionary containing various statistics.
        """
        session = self.get_session()

        try:
            stats = {
                "raw_texts": {
                    "total": session.query(RawText).count(),
                    "unprocessed": session.query(RawText)
                    .filter(RawText.processed == False)  # noqa: E712
                    .count(),
                },
                "candidate_words": {
                    "total": session.query(CandidateWord).count(),
                    "neologisms": session.query(CandidateWord)
                    .filter(CandidateWord.is_neologism == True)  # noqa: E712
                    .count(),
                    "typos": session.query(CandidateWord)
                    .filter(CandidateWord.is_typo == True)  # noqa: E712
                    .count(),
                    "unclassified": session.query(CandidateWord)
                    .filter(CandidateWord.is_neologism.is_(None))
                    .count(),
                },
                "neologisms": {
                    "total": session.query(Neologism).count(),
                    "high_confidence": session.query(Neologism)
                    .filter(Neologism.confidence_level == "high")
                    .count(),
                    "medium_confidence": session.query(Neologism)
                    .filter(Neologism.confidence_level == "medium")
                    .count(),
                    "low_confidence": session.query(Neologism)
                    .filter(Neologism.confidence_level == "low")
                    .count(),
                    "exported": session.query(Neologism)
                    .filter(Neologism.exported == True)  # noqa: E712
                    .count(),
                },
                "collection_jobs": {
                    "total": session.query(CollectionJob).count(),
                    "completed": session.query(CollectionJob)
                    .filter(CollectionJob.status == "completed")
                    .count(),
                    "failed": session.query(CollectionJob)
                    .filter(CollectionJob.status == "failed")
                    .count(),
                },
            }

            return stats

        finally:
            session.close()

    def vacuum_database(self) -> None:
        """Optimize database (SQLite only).

        Reclaims unused space and optimizes database file.
        """
        if not self.config.database.url.startswith("sqlite"):
            logger.warning("VACUUM only supported for SQLite databases")
            return

        try:
            with self.engine.connect() as conn:
                conn.execute("VACUUM")
            logger.info("Database vacuumed successfully")

        except Exception as e:
            logger.error(f"Failed to vacuum database: {e}")
