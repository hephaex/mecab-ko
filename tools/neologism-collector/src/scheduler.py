"""Scheduler for automated neologism collection."""

from __future__ import annotations

import signal
import sys
from datetime import datetime
from typing import Any, Callable

from apscheduler.schedulers.blocking import BlockingScheduler
from apscheduler.triggers.cron import CronTrigger
from apscheduler.triggers.interval import IntervalTrigger
from loguru import logger

from .classifier import NeologismClassifier
from .config import Config, get_config
from .crawler import NaverNewsCrawler, WikipediaCrawler
from .detector import NeologismDetector
from .exporter import MeCabExporter
from .reddit_collector import RedditCollector
from .scorer import NeologismScorer
from .storage import NeologismStorage
from .twitter_collector import TwitterCollector


class CollectionScheduler:
    """Scheduler for automated neologism collection pipeline."""

    def __init__(self, config: Config | None = None) -> None:
        """Initialize scheduler.

        Args:
            config: Configuration object. If None, uses global config.
        """
        self.config = config or get_config()
        self.scheduler_config = self.config.scheduler

        # Initialize storage
        self.storage = NeologismStorage(self.config)
        self.storage.initialize_database()

        # Initialize scheduler
        self.scheduler = BlockingScheduler()

        # Register signal handlers for graceful shutdown
        signal.signal(signal.SIGINT, self._signal_handler)
        signal.signal(signal.SIGTERM, self._signal_handler)

        logger.info("Collection scheduler initialized")

    def _signal_handler(self, signum: int, frame: Any) -> None:
        """Handle shutdown signals.

        Args:
            signum: Signal number.
            frame: Current stack frame.
        """
        logger.info(f"Received signal {signum}, shutting down gracefully...")
        self.shutdown()
        sys.exit(0)

    def setup_jobs(self) -> None:
        """Setup scheduled jobs from configuration."""
        if not self.scheduler_config.enabled:
            logger.warning("Scheduler is disabled in configuration")
            return

        schedules = self.scheduler_config.schedules

        for schedule in schedules:
            job_name = schedule.get("name")
            job_func_name = schedule.get("job")
            trigger_type = schedule.get("trigger")

            if not all([job_name, job_func_name, trigger_type]):
                logger.warning(f"Invalid schedule configuration: {schedule}")
                continue

            # Get job function
            job_func = self._get_job_function(job_func_name)
            if not job_func:
                logger.warning(f"Unknown job function: {job_func_name}")
                continue

            # Create trigger
            trigger = self._create_trigger(trigger_type, schedule)
            if not trigger:
                logger.warning(f"Failed to create trigger for {job_name}")
                continue

            # Add job to scheduler
            self.scheduler.add_job(
                job_func,
                trigger=trigger,
                id=job_name,
                name=job_name,
                replace_existing=True,
            )

            logger.info(f"Scheduled job: {job_name} ({trigger_type})")

    def _get_job_function(self, job_name: str) -> Callable[[], None] | None:
        """Get job function by name.

        Args:
            job_name: Name of the job function.

        Returns:
            Job function or None if not found.
        """
        job_map = {
            "collect_naver_news": self.collect_naver_news,
            "collect_wikipedia": self.collect_wikipedia,
            "collect_twitter": self.collect_twitter,
            "collect_reddit": self.collect_reddit,
            "detect_neologisms": self.detect_neologisms,
            "classify_candidates": self.classify_candidates,
            "score_neologisms": self.score_neologisms,
            "export_mecab_csv": self.export_mecab_csv,
            "cleanup_old_data": self.cleanup_old_data,
            "update_temporal_scores": self.update_temporal_scores,
        }

        return job_map.get(job_name)

    def _create_trigger(
        self,
        trigger_type: str,
        schedule: dict[str, Any],
    ) -> IntervalTrigger | CronTrigger | None:
        """Create APScheduler trigger from configuration.

        Args:
            trigger_type: Type of trigger (interval or cron).
            schedule: Schedule configuration dictionary.

        Returns:
            Trigger object or None if creation failed.
        """
        if trigger_type == "interval":
            # Interval trigger
            kwargs = {}
            for key in ["weeks", "days", "hours", "minutes", "seconds"]:
                if key in schedule:
                    kwargs[key] = schedule[key]

            if not kwargs:
                return None

            return IntervalTrigger(**kwargs)

        elif trigger_type == "cron":
            # Cron trigger
            kwargs = {}
            for key in ["year", "month", "day", "week", "day_of_week", "hour", "minute", "second"]:
                if key in schedule:
                    kwargs[key] = schedule[key]

            if not kwargs:
                return None

            return CronTrigger(**kwargs)

        return None

    # Job functions

    def collect_naver_news(self) -> None:
        """Collect data from Naver News."""
        logger.info("Starting Naver News collection")
        session = self.storage.get_session()

        try:
            job = self.storage.create_collection_job(
                "collect_naver_news",
                "naver_news",
            )

            crawler = NaverNewsCrawler(self.config)
            articles = crawler.collect()

            # Save to database
            saved_count = self.storage.save_raw_texts(articles)

            self.storage.update_collection_job(
                job.id,
                status="completed",
                items_collected=saved_count,
            )

            logger.info(f"Naver News collection completed: {saved_count} articles")

        except Exception as e:
            logger.error(f"Naver News collection failed: {e}")
            if "job" in locals():
                self.storage.update_collection_job(
                    job.id,
                    status="failed",
                    error_message=str(e),
                )

        finally:
            session.close()

    def collect_wikipedia(self) -> None:
        """Collect data from Wikipedia."""
        logger.info("Starting Wikipedia collection")
        session = self.storage.get_session()

        try:
            job = self.storage.create_collection_job(
                "collect_wikipedia",
                "wikipedia",
            )

            crawler = WikipediaCrawler(self.config)
            pages = crawler.collect()

            saved_count = self.storage.save_raw_texts(pages)

            self.storage.update_collection_job(
                job.id,
                status="completed",
                items_collected=saved_count,
            )

            logger.info(f"Wikipedia collection completed: {saved_count} pages")

        except Exception as e:
            logger.error(f"Wikipedia collection failed: {e}")
            if "job" in locals():
                self.storage.update_collection_job(
                    job.id,
                    status="failed",
                    error_message=str(e),
                )

        finally:
            session.close()

    def collect_twitter(self) -> None:
        """Collect data from Twitter/X."""
        if not self.config.sources.get("twitter", {}).get("enabled", False):
            logger.info("Twitter collection is disabled")
            return

        logger.info("Starting Twitter collection")
        session = self.storage.get_session()

        try:
            job = self.storage.create_collection_job(
                "collect_twitter",
                "twitter",
            )

            collector = TwitterCollector(self.config)
            tweets = collector.collect()

            saved_count = self.storage.save_raw_texts(tweets)

            self.storage.update_collection_job(
                job.id,
                status="completed",
                items_collected=saved_count,
            )

            logger.info(f"Twitter collection completed: {saved_count} tweets")

        except Exception as e:
            logger.error(f"Twitter collection failed: {e}")
            if "job" in locals():
                self.storage.update_collection_job(
                    job.id,
                    status="failed",
                    error_message=str(e),
                )

        finally:
            session.close()

    def collect_reddit(self) -> None:
        """Collect data from Reddit."""
        if not self.config.sources.get("reddit", {}).get("enabled", False):
            logger.info("Reddit collection is disabled")
            return

        logger.info("Starting Reddit collection")
        session = self.storage.get_session()

        try:
            job = self.storage.create_collection_job(
                "collect_reddit",
                "reddit",
            )

            collector = RedditCollector(self.config)
            posts = collector.collect()

            saved_count = self.storage.save_raw_texts(posts)

            self.storage.update_collection_job(
                job.id,
                status="completed",
                items_collected=saved_count,
            )

            logger.info(f"Reddit collection completed: {saved_count} posts")

        except Exception as e:
            logger.error(f"Reddit collection failed: {e}")
            if "job" in locals():
                self.storage.update_collection_job(
                    job.id,
                    status="failed",
                    error_message=str(e),
                )

        finally:
            session.close()

    def detect_neologisms(self) -> None:
        """Detect neologisms from collected texts."""
        logger.info("Starting neologism detection")
        session = self.storage.get_session()

        try:
            detector = NeologismDetector(session, self.config)
            count = detector.detect_from_raw_texts(batch_size=500)
            logger.info(f"Neologism detection completed: {count} candidates")

        except Exception as e:
            logger.error(f"Neologism detection failed: {e}")

        finally:
            session.close()

    def classify_candidates(self) -> None:
        """Classify candidate words."""
        logger.info("Starting candidate classification")
        session = self.storage.get_session()

        try:
            classifier = NeologismClassifier(session, self.config)
            count = classifier.classify_candidates(batch_size=200)
            logger.info(f"Classification completed: {count} candidates")

        except Exception as e:
            logger.error(f"Classification failed: {e}")

        finally:
            session.close()

    def score_neologisms(self) -> None:
        """Score confirmed neologisms."""
        logger.info("Starting neologism scoring")
        session = self.storage.get_session()

        try:
            scorer = NeologismScorer(session, self.config)
            count = scorer.score_neologisms(batch_size=200)
            logger.info(f"Scoring completed: {count} neologisms")

        except Exception as e:
            logger.error(f"Scoring failed: {e}")

        finally:
            session.close()

    def export_mecab_csv(self) -> None:
        """Export neologisms to MeCab CSV format."""
        logger.info("Starting MeCab CSV export")
        session = self.storage.get_session()

        try:
            exporter = MeCabExporter(session, self.config)
            output_path = exporter.export_neologisms(min_confidence="medium")
            logger.info(f"Export completed: {output_path}")

        except Exception as e:
            logger.error(f"Export failed: {e}")

        finally:
            session.close()

    def cleanup_old_data(self) -> None:
        """Clean up old data."""
        logger.info("Starting data cleanup")

        try:
            deleted = self.storage.cleanup_old_data()
            logger.info(f"Cleanup completed: {deleted}")

            # Vacuum database
            self.storage.vacuum_database()

        except Exception as e:
            logger.error(f"Cleanup failed: {e}")

    def update_temporal_scores(self) -> None:
        """Update temporal scores for all neologisms."""
        logger.info("Starting temporal score update")
        session = self.storage.get_session()

        try:
            scorer = NeologismScorer(session, self.config)
            count = scorer.update_temporal_scores()
            logger.info(f"Temporal score update completed: {count} neologisms")

        except Exception as e:
            logger.error(f"Temporal score update failed: {e}")

        finally:
            session.close()

    def run(self) -> None:
        """Start the scheduler."""
        self.setup_jobs()

        if not self.scheduler.get_jobs():
            logger.warning("No jobs scheduled!")
            return

        logger.info("Starting scheduler...")
        logger.info(f"Scheduled jobs: {[job.id for job in self.scheduler.get_jobs()]}")

        try:
            self.scheduler.start()
        except (KeyboardInterrupt, SystemExit):
            logger.info("Scheduler stopped")

    def shutdown(self) -> None:
        """Shutdown the scheduler gracefully."""
        if self.scheduler.running:
            self.scheduler.shutdown(wait=True)
            logger.info("Scheduler shutdown complete")

    def run_once(self, job_name: str) -> None:
        """Run a single job immediately.

        Args:
            job_name: Name of the job to run.
        """
        job_func = self._get_job_function(job_name)

        if not job_func:
            logger.error(f"Unknown job: {job_name}")
            return

        logger.info(f"Running job: {job_name}")
        job_func()
        logger.info(f"Job completed: {job_name}")
