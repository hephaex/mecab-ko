"""Neologism scoring system."""

from __future__ import annotations

import math
from datetime import datetime, timedelta
from typing import Any

from loguru import logger
from sqlalchemy import func, select
from sqlalchemy.orm import Session

from .config import Config, get_config
from .models import CandidateWord, Neologism


class NeologismScorer:
    """Scorer for calculating neologism quality scores."""

    def __init__(self, session: Session, config: Config | None = None) -> None:
        """Initialize scorer.

        Args:
            session: SQLAlchemy database session.
            config: Configuration object. If None, uses global config.
        """
        self.session = session
        self.config = config or get_config()
        self.scorer_config = self.config.scorer

    def score_neologisms(self, batch_size: int = 100) -> int:
        """Score classified neologisms and create Neologism entries.

        Args:
            batch_size: Number of candidates to process in one batch.

        Returns:
            Number of neologisms scored.
        """
        # Get confirmed neologisms that haven't been scored yet
        stmt = (
            select(CandidateWord)
            .where(
                CandidateWord.is_neologism == True,  # noqa: E712
                CandidateWord.confidence_score >= self.scorer_config.thresholds["low_confidence"],
            )
            .order_by(CandidateWord.frequency.desc())
            .limit(batch_size)
        )
        candidates = self.session.execute(stmt).scalars().all()

        if not candidates:
            logger.info("No neologisms to score")
            return 0

        logger.info(f"Scoring {len(candidates)} neologisms")
        scored_count = 0

        for candidate in candidates:
            try:
                # Calculate scores
                scores = self._calculate_scores(candidate)

                # Create or update Neologism entry
                self._create_or_update_neologism(candidate, scores)
                scored_count += 1

            except Exception as e:
                logger.error(f"Failed to score {candidate.surface}: {e}")
                continue

        self.session.commit()
        logger.info(f"Scored {scored_count} neologisms")

        return scored_count

    def _calculate_scores(self, candidate: CandidateWord) -> dict[str, float]:
        """Calculate all component scores for a neologism.

        Args:
            candidate: Candidate word to score.

        Returns:
            Dictionary of individual and total scores.
        """
        scores = {}

        # 1. Frequency score
        scores["frequency_score"] = self._calculate_frequency_score(candidate)

        # 2. Context diversity score
        scores["context_diversity_score"] = self._calculate_context_diversity_score(
            candidate
        )

        # 3. Morphological score
        scores["morphological_score"] = self._calculate_morphological_score(candidate)

        # 4. Social spread score
        scores["social_spread_score"] = self._calculate_social_spread_score(candidate)

        # 5. Temporal trend score
        scores["temporal_trend_score"] = self._calculate_temporal_trend_score(candidate)

        # Calculate weighted total
        weights = self.config.classifier.weights
        total_score = (
            scores["frequency_score"] * weights.get("frequency", 0.3)
            + scores["context_diversity_score"] * weights.get("context_diversity", 0.25)
            + scores["morphological_score"] * weights.get("morphological_validity", 0.2)
            + scores["social_spread_score"] * weights.get("social_spread", 0.15)
            + scores["temporal_trend_score"] * weights.get("temporal_trend", 0.1)
        )

        scores["total_score"] = total_score

        return scores

    def _calculate_frequency_score(self, candidate: CandidateWord) -> float:
        """Calculate score based on frequency.

        Args:
            candidate: Candidate word.

        Returns:
            Frequency score (0.0 to 1.0).
        """
        # Log scale to handle wide range of frequencies
        if candidate.frequency <= 0:
            return 0.0

        # Score increases with log of frequency
        score = math.log(candidate.frequency + 1) / math.log(10000)
        return min(1.0, score)

    def _calculate_context_diversity_score(self, candidate: CandidateWord) -> float:
        """Calculate score based on context diversity.

        Args:
            candidate: Candidate word.

        Returns:
            Context diversity score (0.0 to 1.0).
        """
        contexts = candidate.contexts or []
        num_contexts = len(contexts)

        if num_contexts == 0:
            return 0.0

        # More diverse contexts = higher score
        score = math.log(num_contexts + 1) / math.log(100)
        return min(1.0, score)

    def _calculate_morphological_score(self, candidate: CandidateWord) -> float:
        """Calculate score based on morphological validity.

        Args:
            candidate: Candidate word.

        Returns:
            Morphological score (0.0 to 1.0).
        """
        # Use metadata from classifier if available
        metadata = candidate.metadata or {}
        if "morphological_validity" in metadata:
            return metadata["morphological_validity"]

        # Fallback: simple heuristics
        length = candidate.length

        # Prefer words of reasonable length
        if 2 <= length <= 4:
            length_score = 1.0
        elif length == 1 or length > 8:
            length_score = 0.3
        else:
            length_score = 0.7

        # Prefer pure Korean or Korean+English
        if candidate.has_hangul and not candidate.has_number:
            type_score = 1.0
        elif candidate.has_hangul and candidate.has_english:
            type_score = 0.8
        else:
            type_score = 0.5

        return (length_score + type_score) / 2.0

    def _calculate_social_spread_score(self, candidate: CandidateWord) -> float:
        """Calculate score based on social spread across sources.

        Args:
            candidate: Candidate word.

        Returns:
            Social spread score (0.0 to 1.0).
        """
        sources = candidate.sources or []
        num_sources = len(sources)

        if num_sources == 0:
            return 0.0

        # More sources = wider spread
        score = math.log(num_sources + 1) / math.log(10)
        return min(1.0, score)

    def _calculate_temporal_trend_score(self, candidate: CandidateWord) -> float:
        """Calculate score based on temporal trend.

        Args:
            candidate: Candidate word.

        Returns:
            Temporal trend score (0.0 to 1.0).
        """
        # Calculate days since first seen
        days_since_first = (datetime.now() - candidate.first_seen).days

        if days_since_first <= 0:
            return 1.0  # Brand new

        # Apply temporal decay
        decay_days = self.scorer_config.temporal_decay_days
        decay_factor = math.exp(-days_since_first / decay_days)

        # Calculate recency boost
        days_since_last = (datetime.now() - candidate.last_seen).days
        if days_since_last == 0:
            recency_boost = 1.0
        else:
            recency_boost = math.exp(-days_since_last / 7)  # 7-day half-life

        # Combine decay and recency
        score = (decay_factor + recency_boost) / 2.0
        return min(1.0, score)

    def _create_or_update_neologism(
        self,
        candidate: CandidateWord,
        scores: dict[str, float],
    ) -> Neologism:
        """Create or update Neologism entry from candidate.

        Args:
            candidate: Candidate word.
            scores: Calculated scores.

        Returns:
            Created or updated Neologism instance.
        """
        # Check if neologism already exists
        stmt = select(Neologism).where(Neologism.surface == candidate.surface)
        neologism = self.session.execute(stmt).scalar_one_or_none()

        # Determine confidence level
        total_score = scores["total_score"]
        thresholds = self.scorer_config.thresholds

        if total_score >= thresholds["high_confidence"]:
            confidence_level = "high"
        elif total_score >= thresholds["medium_confidence"]:
            confidence_level = "medium"
        else:
            confidence_level = "low"

        # Extract examples from contexts
        examples = (candidate.contexts or [])[:10]  # Keep top 10 examples

        if neologism:
            # Update existing
            neologism.total_score = total_score
            neologism.frequency_score = scores["frequency_score"]
            neologism.context_diversity_score = scores["context_diversity_score"]
            neologism.morphological_score = scores["morphological_score"]
            neologism.social_spread_score = scores["social_spread_score"]
            neologism.temporal_trend_score = scores["temporal_trend_score"]

            neologism.frequency = candidate.frequency
            neologism.context_count = len(candidate.contexts or [])
            neologism.source_count = len(candidate.sources or [])

            neologism.confidence_level = confidence_level
            neologism.examples = examples
            neologism.last_updated = datetime.now()

        else:
            # Create new
            cost = self._calculate_mecab_cost(total_score)

            neologism = Neologism(
                surface=candidate.surface,
                total_score=total_score,
                frequency_score=scores["frequency_score"],
                context_diversity_score=scores["context_diversity_score"],
                morphological_score=scores["morphological_score"],
                social_spread_score=scores["social_spread_score"],
                temporal_trend_score=scores["temporal_trend_score"],
                frequency=candidate.frequency,
                context_count=len(candidate.contexts or []),
                source_count=len(candidate.sources or []),
                pos=self.config.exporter.default_pos,
                cost=cost,
                confidence_level=confidence_level,
                first_detected=candidate.first_seen,
                examples=examples,
            )
            self.session.add(neologism)

        return neologism

    def _calculate_mecab_cost(self, score: float) -> int:
        """Calculate MeCab dictionary cost based on score.

        Higher score = lower cost (more likely to be used).

        Args:
            score: Neologism score (0.0 to 1.0).

        Returns:
            MeCab cost value.
        """
        base_cost = self.config.exporter.cost_base
        adjustment_factor = self.config.exporter.cost_adjustment_factor

        # Invert score (high score = low cost)
        cost = int(base_cost - (score * adjustment_factor * 10))

        # Clamp to reasonable range
        return max(1000, min(10000, cost))

    def update_temporal_scores(self) -> int:
        """Update temporal scores for all neologisms.

        This should be run periodically to decay scores over time.

        Returns:
            Number of neologisms updated.
        """
        logger.info("Updating temporal scores for all neologisms")

        stmt = select(Neologism)
        neologisms = self.session.execute(stmt).scalars().all()

        updated_count = 0

        for neologism in neologisms:
            try:
                # Get candidate for temporal calculation
                stmt_candidate = select(CandidateWord).where(
                    CandidateWord.surface == neologism.surface
                )
                candidate = self.session.execute(stmt_candidate).scalar_one_or_none()

                if not candidate:
                    continue

                # Recalculate temporal score
                new_temporal_score = self._calculate_temporal_trend_score(candidate)
                neologism.temporal_trend_score = new_temporal_score

                # Recalculate total score
                weights = self.config.classifier.weights
                neologism.total_score = (
                    neologism.frequency_score * weights.get("frequency", 0.3)
                    + neologism.context_diversity_score * weights.get("context_diversity", 0.25)
                    + neologism.morphological_score * weights.get("morphological_validity", 0.2)
                    + neologism.social_spread_score * weights.get("social_spread", 0.15)
                    + new_temporal_score * weights.get("temporal_trend", 0.1)
                )

                # Update cost
                neologism.cost = self._calculate_mecab_cost(neologism.total_score)
                neologism.last_updated = datetime.now()

                updated_count += 1

            except Exception as e:
                logger.error(f"Failed to update temporal score for {neologism.surface}: {e}")
                continue

        self.session.commit()
        logger.info(f"Updated {updated_count} neologisms")

        return updated_count

    def get_statistics(self) -> dict[str, Any]:
        """Get scoring statistics.

        Returns:
            Dictionary containing statistics.
        """
        total = self.session.query(Neologism).count()

        high_confidence = self.session.query(Neologism).filter(
            Neologism.confidence_level == "high"
        ).count()

        medium_confidence = self.session.query(Neologism).filter(
            Neologism.confidence_level == "medium"
        ).count()

        low_confidence = self.session.query(Neologism).filter(
            Neologism.confidence_level == "low"
        ).count()

        avg_score = self.session.query(func.avg(Neologism.total_score)).scalar() or 0.0

        return {
            "total_neologisms": total,
            "high_confidence": high_confidence,
            "medium_confidence": medium_confidence,
            "low_confidence": low_confidence,
            "average_score": float(avg_score),
        }
