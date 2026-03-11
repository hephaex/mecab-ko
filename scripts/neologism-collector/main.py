#!/usr/bin/env python3
"""Main entry point for neologism collector."""

from __future__ import annotations

import sys
from pathlib import Path

import click
from loguru import logger

from src.config import get_config, reset_config
from src.scheduler import CollectionScheduler
from src.storage import NeologismStorage


def setup_logging(config_path: str | None = None) -> None:
    """Setup logging configuration.

    Args:
        config_path: Path to configuration file.
    """
    config = get_config(config_path)
    log_config = config.logging

    # Remove default handler
    logger.remove()

    # Add console handler
    logger.add(
        sys.stderr,
        level=log_config.level,
        format=log_config.format,
        colorize=True,
    )

    # Add file handlers
    for log_type, log_file in log_config.files.items():
        Path(log_file).parent.mkdir(parents=True, exist_ok=True)
        logger.add(
            log_file,
            level=log_config.level,
            format=log_config.format,
            rotation=log_config.rotation,
            retention=log_config.retention,
            compression=log_config.compression,
        )


@click.group()
@click.option(
    "--config",
    "-c",
    type=click.Path(exists=True),
    help="Path to configuration file",
)
@click.pass_context
def cli(ctx: click.Context, config: str | None) -> None:
    """Neologism Collector for MeCab-Ko.

    Automated pipeline for collecting, detecting, and exporting Korean neologisms.
    """
    ctx.ensure_object(dict)
    ctx.obj["config_path"] = config

    # Reset config if different path provided
    if config:
        reset_config()

    setup_logging(config)


@cli.command()
@click.pass_context
def init(ctx: click.Context) -> None:
    """Initialize database and create tables."""
    config_path = ctx.obj.get("config_path")
    config = get_config(config_path)

    storage = NeologismStorage(config)
    storage.initialize_database()

    logger.info("Database initialized successfully")
    click.echo("Database initialized successfully!")


@cli.command()
@click.pass_context
def scheduler(ctx: click.Context) -> None:
    """Start the collection scheduler."""
    config_path = ctx.obj.get("config_path")
    config = get_config(config_path)

    scheduler = CollectionScheduler(config)
    scheduler.run()


@cli.command()
@click.argument("job_name")
@click.pass_context
def run(ctx: click.Context, job_name: str) -> None:
    """Run a single job immediately.

    JOB_NAME: Name of the job to run (e.g., collect_naver_news, detect_neologisms)
    """
    config_path = ctx.obj.get("config_path")
    config = get_config(config_path)

    scheduler = CollectionScheduler(config)
    scheduler.run_once(job_name)


@cli.command()
@click.option(
    "--source",
    "-s",
    type=click.Choice(["naver_news", "wikipedia", "twitter", "reddit", "all"]),
    default="all",
    help="Source to collect from",
)
@click.pass_context
def collect(ctx: click.Context, source: str) -> None:
    """Collect data from sources."""
    config_path = ctx.obj.get("config_path")
    config = get_config(config_path)

    scheduler = CollectionScheduler(config)

    sources_to_run = {
        "naver_news": "collect_naver_news",
        "wikipedia": "collect_wikipedia",
        "twitter": "collect_twitter",
        "reddit": "collect_reddit",
    }

    if source == "all":
        for job_name in sources_to_run.values():
            try:
                scheduler.run_once(job_name)
            except Exception as e:
                logger.error(f"Failed to run {job_name}: {e}")
    else:
        job_name = sources_to_run[source]
        scheduler.run_once(job_name)


@cli.command()
@click.pass_context
def detect(ctx: click.Context) -> None:
    """Detect neologisms from collected texts."""
    config_path = ctx.obj.get("config_path")
    config = get_config(config_path)

    scheduler = CollectionScheduler(config)
    scheduler.run_once("detect_neologisms")


@cli.command()
@click.pass_context
def classify(ctx: click.Context) -> None:
    """Classify candidate words."""
    config_path = ctx.obj.get("config_path")
    config = get_config(config_path)

    scheduler = CollectionScheduler(config)
    scheduler.run_once("classify_candidates")


@cli.command()
@click.pass_context
def score(ctx: click.Context) -> None:
    """Score confirmed neologisms."""
    config_path = ctx.obj.get("config_path")
    config = get_config(config_path)

    scheduler = CollectionScheduler(config)
    scheduler.run_once("score_neologisms")


@cli.command()
@click.option(
    "--min-confidence",
    "-m",
    type=click.Choice(["low", "medium", "high"]),
    default="medium",
    help="Minimum confidence level",
)
@click.option(
    "--output",
    "-o",
    type=click.Path(),
    help="Output file path",
)
@click.pass_context
def export(ctx: click.Context, min_confidence: str, output: str | None) -> None:
    """Export neologisms to MeCab CSV format."""
    config_path = ctx.obj.get("config_path")
    config = get_config(config_path)

    from src.exporter import MeCabExporter

    storage = NeologismStorage(config)
    session = storage.get_session()

    try:
        exporter = MeCabExporter(session, config)
        output_path = exporter.export_neologisms(
            min_confidence=min_confidence,
            output_file=output,
        )
        click.echo(f"Exported to: {output_path}")

    finally:
        session.close()


@cli.command()
@click.pass_context
def stats(ctx: click.Context) -> None:
    """Display collection statistics."""
    config_path = ctx.obj.get("config_path")
    config = get_config(config_path)

    storage = NeologismStorage(config)
    statistics = storage.get_statistics()

    click.echo("\n=== Neologism Collection Statistics ===\n")

    click.echo("Raw Texts:")
    click.echo(f"  Total: {statistics['raw_texts']['total']}")
    click.echo(f"  Unprocessed: {statistics['raw_texts']['unprocessed']}")

    click.echo("\nCandidate Words:")
    click.echo(f"  Total: {statistics['candidate_words']['total']}")
    click.echo(f"  Neologisms: {statistics['candidate_words']['neologisms']}")
    click.echo(f"  Typos: {statistics['candidate_words']['typos']}")
    click.echo(f"  Unclassified: {statistics['candidate_words']['unclassified']}")

    click.echo("\nNeologisms:")
    click.echo(f"  Total: {statistics['neologisms']['total']}")
    click.echo(f"  High Confidence: {statistics['neologisms']['high_confidence']}")
    click.echo(f"  Medium Confidence: {statistics['neologisms']['medium_confidence']}")
    click.echo(f"  Low Confidence: {statistics['neologisms']['low_confidence']}")
    click.echo(f"  Exported: {statistics['neologisms']['exported']}")

    click.echo("\nCollection Jobs:")
    click.echo(f"  Total: {statistics['collection_jobs']['total']}")
    click.echo(f"  Completed: {statistics['collection_jobs']['completed']}")
    click.echo(f"  Failed: {statistics['collection_jobs']['failed']}")


@cli.command()
@click.pass_context
def cleanup(ctx: click.Context) -> None:
    """Clean up old data."""
    config_path = ctx.obj.get("config_path")
    config = get_config(config_path)

    scheduler = CollectionScheduler(config)
    scheduler.run_once("cleanup_old_data")

    click.echo("Cleanup completed!")


@cli.command()
@click.pass_context
def pipeline(ctx: click.Context) -> None:
    """Run the complete pipeline: collect -> detect -> classify -> score -> export."""
    config_path = ctx.obj.get("config_path")
    config = get_config(config_path)

    scheduler = CollectionScheduler(config)

    steps = [
        ("Collecting from all sources", "collect_naver_news"),
        ("Collecting from Wikipedia", "collect_wikipedia"),
        ("Detecting neologisms", "detect_neologisms"),
        ("Classifying candidates", "classify_candidates"),
        ("Scoring neologisms", "score_neologisms"),
        ("Exporting to MeCab format", "export_mecab_csv"),
    ]

    for description, job_name in steps:
        click.echo(f"\n{description}...")
        try:
            scheduler.run_once(job_name)
            click.echo(f"✓ {description} completed")
        except Exception as e:
            click.echo(f"✗ {description} failed: {e}")
            logger.error(f"{description} failed: {e}")

    click.echo("\n✓ Pipeline completed!")


if __name__ == "__main__":
    cli()
