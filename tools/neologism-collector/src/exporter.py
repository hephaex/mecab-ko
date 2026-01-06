"""MeCab dictionary format exporter."""

from __future__ import annotations

import csv
from datetime import datetime
from pathlib import Path
from typing import Any

from loguru import logger
from sqlalchemy import select
from sqlalchemy.orm import Session

from .config import Config, get_config
from .models import Neologism


class MeCabExporter:
    """Exporter for MeCab dictionary CSV format."""

    def __init__(self, session: Session, config: Config | None = None) -> None:
        """Initialize exporter.

        Args:
            session: SQLAlchemy database session.
            config: Configuration object. If None, uses global config.
        """
        self.session = session
        self.config = config or get_config()
        self.exporter_config = self.config.exporter

        # Ensure output directory exists
        output_dir = Path(self.exporter_config.output_dir)
        output_dir.mkdir(parents=True, exist_ok=True)

    def export_neologisms(
        self,
        min_confidence: str = "medium",
        output_file: str | None = None,
    ) -> Path:
        """Export neologisms to MeCab CSV format.

        Args:
            min_confidence: Minimum confidence level (low, medium, high).
            output_file: Output file path. If None, generates timestamped filename.

        Returns:
            Path to exported file.
        """
        # Determine confidence filter
        confidence_levels = {
            "low": ["low", "medium", "high"],
            "medium": ["medium", "high"],
            "high": ["high"],
        }
        allowed_levels = confidence_levels.get(min_confidence, ["medium", "high"])

        # Get neologisms
        stmt = (
            select(Neologism)
            .where(Neologism.confidence_level.in_(allowed_levels))
            .order_by(Neologism.total_score.desc())
        )
        neologisms = self.session.execute(stmt).scalars().all()

        if not neologisms:
            logger.warning("No neologisms found to export")
            return Path()

        # Generate output filename
        if output_file is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            output_file = f"neologisms_{min_confidence}_{timestamp}.csv"

        output_path = Path(self.exporter_config.output_dir) / output_file

        # Write CSV
        logger.info(f"Exporting {len(neologisms)} neologisms to {output_path}")

        with output_path.open("w", encoding="utf-8", newline="") as f:
            writer = csv.writer(f)

            for neologism in neologisms:
                row = self._format_mecab_row(neologism)
                writer.writerow(row)

                # Mark as exported
                neologism.exported = True
                neologism.exported_at = datetime.now()

        self.session.commit()

        logger.info(f"Successfully exported to {output_path}")
        return output_path

    def _format_mecab_row(self, neologism: Neologism) -> list[str]:
        """Format neologism as MeCab dictionary row.

        MeCab-Ko dictionary format:
        표층형,좌문맥ID,우문맥ID,비용,품사,의미분류1,의미분류2,종성유무,읽기,타입,첫번째품사,마지막품사,원형,*

        Args:
            neologism: Neologism to format.

        Returns:
            List of field values.
        """
        surface = neologism.surface

        # Calculate left/right context IDs (simplified)
        # In practice, these would come from mecab-ko-dic build process
        left_id = 1780  # Default for NNG
        right_id = 3540  # Default for NNG

        # Cost
        cost = neologism.cost

        # POS tag
        pos = neologism.pos or self.exporter_config.default_pos

        # Semantic categories
        semantic1 = neologism.semantic1 or "*"
        semantic2 = neologism.semantic2 or "*"

        # Check if has jongseong (final consonant)
        has_jongseong = self._has_jongseong(surface)
        jongseong_marker = "T" if has_jongseong else "F"

        # Reading (usually same as surface for Korean)
        reading = neologism.reading or surface

        # Type markers
        type_marker = "*"
        first_pos = pos
        last_pos = pos
        original = surface

        return [
            surface,
            str(left_id),
            str(right_id),
            str(cost),
            pos,
            semantic1,
            semantic2,
            jongseong_marker,
            reading,
            type_marker,
            first_pos,
            last_pos,
            original,
            "*",
        ]

    @staticmethod
    def _has_jongseong(word: str) -> bool:
        """Check if last character has jongseong (final consonant).

        Args:
            word: Korean word to check.

        Returns:
            True if has jongseong, False otherwise.
        """
        if not word:
            return False

        last_char = word[-1]

        # Check if it's a Korean character
        if not ("\uac00" <= last_char <= "\ud7a3"):
            return False

        # Calculate jongseong
        # Korean Unicode: 0xAC00 + (초성 * 588) + (중성 * 28) + 종성
        char_code = ord(last_char) - 0xAC00
        jongseong = char_code % 28

        return jongseong != 0

    def export_by_source(self, source: str, output_file: str | None = None) -> Path:
        """Export neologisms from a specific source.

        Args:
            source: Source name to filter by.
            output_file: Output file path.

        Returns:
            Path to exported file.
        """
        # This would require tracking source in Neologism model
        # For now, we'll export all and note this as a TODO
        logger.warning("Export by source not fully implemented yet")
        return self.export_neologisms(output_file=output_file)

    def export_statistics(self, output_file: str | None = None) -> Path:
        """Export statistics about neologisms.

        Args:
            output_file: Output file path.

        Returns:
            Path to exported file.
        """
        if output_file is None:
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            output_file = f"neologism_stats_{timestamp}.csv"

        output_path = Path(self.exporter_config.output_dir) / output_file

        stmt = select(Neologism).order_by(Neologism.total_score.desc())
        neologisms = self.session.execute(stmt).scalars().all()

        with output_path.open("w", encoding="utf-8", newline="") as f:
            writer = csv.writer(f)

            # Header
            writer.writerow([
                "Surface",
                "Total Score",
                "Frequency",
                "Contexts",
                "Sources",
                "Confidence",
                "POS",
                "First Detected",
                "Last Updated",
            ])

            # Data
            for neologism in neologisms:
                writer.writerow([
                    neologism.surface,
                    f"{neologism.total_score:.4f}",
                    neologism.frequency,
                    neologism.context_count,
                    neologism.source_count,
                    neologism.confidence_level,
                    neologism.pos,
                    neologism.first_detected.strftime("%Y-%m-%d"),
                    neologism.last_updated.strftime("%Y-%m-%d"),
                ])

        logger.info(f"Statistics exported to {output_path}")
        return output_path

    def get_export_summary(self) -> dict[str, Any]:
        """Get summary of export status.

        Returns:
            Dictionary with export statistics.
        """
        total = self.session.query(Neologism).count()
        exported = (
            self.session.query(Neologism)
            .filter(Neologism.exported == True)  # noqa: E712
            .count()
        )

        by_confidence = {}
        for level in ["high", "medium", "low"]:
            count = (
                self.session.query(Neologism)
                .filter(Neologism.confidence_level == level)
                .count()
            )
            exported_count = (
                self.session.query(Neologism)
                .filter(
                    Neologism.confidence_level == level,
                    Neologism.exported == True,  # noqa: E712
                )
                .count()
            )
            by_confidence[level] = {
                "total": count,
                "exported": exported_count,
                "pending": count - exported_count,
            }

        return {
            "total_neologisms": total,
            "total_exported": exported,
            "pending_export": total - exported,
            "by_confidence": by_confidence,
        }
