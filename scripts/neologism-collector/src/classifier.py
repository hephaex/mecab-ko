"""Neologism classifier - distinguish neologisms from typos."""

from __future__ import annotations

import math
from datetime import datetime
from typing import Any

from loguru import logger
from sqlalchemy import func, select
from sqlalchemy.orm import Session

from .config import Config, get_config
from .models import CandidateWord


class NeologismClassifier:
    """Classifier to distinguish neologisms from typos and errors."""

    # Korean keyboard layout for calculating keyboard distance
    KEYBOARD_LAYOUT = {
        "ㅂ": (0, 0), "ㅈ": (0, 1), "ㄷ": (0, 2), "ㄱ": (0, 3),
        "ㅅ": (0, 4), "ㅛ": (0, 5), "ㅕ": (0, 6), "ㅑ": (0, 7),
        "ㅐ": (0, 8), "ㅔ": (0, 9),
        "ㅁ": (1, 0), "ㄴ": (1, 1), "ㅇ": (1, 2), "ㄹ": (1, 3),
        "ㅎ": (1, 4), "ㅗ": (1, 5), "ㅓ": (1, 6), "ㅏ": (1, 7),
        "ㅣ": (1, 8),
        "ㅋ": (2, 0), "ㅌ": (2, 1), "ㅊ": (2, 2), "ㅍ": (2, 3),
        "ㅠ": (2, 4), "ㅜ": (2, 5), "ㅡ": (2, 6),
    }

    def __init__(self, session: Session, config: Config | None = None) -> None:
        """Initialize classifier.

        Args:
            session: SQLAlchemy database session.
            config: Configuration object. If None, uses global config.
        """
        self.session = session
        self.config = config or get_config()
        self.classifier_config = self.config.classifier

        # Load KoNLPy for morphological analysis (optional)
        try:
            from konlpy.tag import Okt
            self.tagger = Okt()
            self.use_konlpy = True
            logger.info("KoNLPy initialized for classification")
        except ImportError:
            self.tagger = None
            self.use_konlpy = False
            logger.warning("KoNLPy not available for classification")

    def classify_candidates(self, batch_size: int = 100) -> int:
        """Classify unclassified candidate words.

        Args:
            batch_size: Number of candidates to process in one batch.

        Returns:
            Number of candidates classified.
        """
        # Get unclassified candidates
        stmt = (
            select(CandidateWord)
            .where(CandidateWord.is_neologism.is_(None))
            .order_by(CandidateWord.frequency.desc())
            .limit(batch_size)
        )
        candidates = self.session.execute(stmt).scalars().all()

        if not candidates:
            logger.info("No unclassified candidates found")
            return 0

        logger.info(f"Classifying {len(candidates)} candidates")
        classified_count = 0

        for candidate in candidates:
            try:
                result = self.classify(candidate)
                candidate.is_neologism = result["is_neologism"]
                candidate.is_typo = result["is_typo"]
                candidate.confidence_score = result["confidence"]
                candidate.metadata = result.get("features", {})
                classified_count += 1

            except Exception as e:
                logger.error(f"Failed to classify {candidate.surface}: {e}")
                continue

        self.session.commit()
        logger.info(f"Classified {classified_count} candidates")

        return classified_count

    def classify(self, candidate: CandidateWord) -> dict[str, Any]:
        """Classify a single candidate word.

        Args:
            candidate: Candidate word to classify.

        Returns:
            Dictionary with classification results.
        """
        features = self._extract_features(candidate)
        score = self._calculate_score(features)

        # Determine classification
        is_neologism = score > 0.5
        is_typo = score < 0.3 and self._is_likely_typo(candidate, features)

        return {
            "is_neologism": is_neologism,
            "is_typo": is_typo,
            "confidence": score,
            "features": features,
        }

    def _extract_features(self, candidate: CandidateWord) -> dict[str, float]:
        """Extract classification features from candidate.

        Args:
            candidate: Candidate word.

        Returns:
            Dictionary of feature values (0.0 to 1.0).
        """
        features = {}

        # 1. Length feature
        features["length"] = self._normalize_length(candidate.length)

        # 2. Character type ratio
        features["char_type_ratio"] = self._calculate_char_type_ratio(candidate)

        # 3. Keyboard distance (for typo detection)
        features["keyboard_distance"] = self._calculate_keyboard_distance(
            candidate.surface
        )

        # 4. Phonetic similarity (simplified)
        features["phonetic_similarity"] = 0.5  # Placeholder

        # 5. Context coherence
        features["context_coherence"] = self._calculate_context_coherence(candidate)

        # 6. Frequency feature
        features["frequency"] = self._normalize_frequency(candidate.frequency)

        # 7. Context diversity
        features["context_diversity"] = self._normalize_context_diversity(
            len(candidate.contexts or [])
        )

        # 8. Morphological validity
        features["morphological_validity"] = self._check_morphological_validity(
            candidate.surface
        )

        return features

    def _calculate_score(self, features: dict[str, float]) -> float:
        """Calculate overall neologism score from features.

        Args:
            features: Feature dictionary.

        Returns:
            Score between 0.0 and 1.0.
        """
        weights = self.classifier_config.weights

        score = 0.0
        total_weight = 0.0

        # Map features to weights
        feature_weight_map = {
            "frequency": weights.get("frequency", 0.3),
            "context_diversity": weights.get("context_diversity", 0.25),
            "morphological_validity": weights.get("morphological_validity", 0.2),
            "context_coherence": weights.get("social_spread", 0.15),
        }

        for feature, weight in feature_weight_map.items():
            if feature in features:
                score += features[feature] * weight
                total_weight += weight

        # Normalize
        if total_weight > 0:
            score /= total_weight

        return max(0.0, min(1.0, score))

    def _normalize_length(self, length: int) -> float:
        """Normalize word length to 0-1 scale.

        Args:
            length: Word length.

        Returns:
            Normalized value.
        """
        # Optimal length for Korean words is 2-4 characters
        if 2 <= length <= 4:
            return 1.0
        elif length < 2 or length > 10:
            return 0.0
        else:
            return 1.0 - abs(length - 3) / 7.0

    def _calculate_char_type_ratio(self, candidate: CandidateWord) -> float:
        """Calculate character type diversity ratio.

        Args:
            candidate: Candidate word.

        Returns:
            Ratio value (0.0 to 1.0).
        """
        types_present = sum([
            candidate.has_hangul,
            candidate.has_english,
            candidate.has_number,
        ])

        # Pure Korean or Korean+English is good
        if types_present == 1 or (candidate.has_hangul and candidate.has_english):
            return 1.0
        elif types_present == 0:
            return 0.0
        else:
            return 0.5

    def _calculate_keyboard_distance(self, word: str) -> float:
        """Calculate average keyboard distance between adjacent characters.

        High distance suggests intentional typing (neologism).
        Low distance suggests typo (fat finger error).

        Args:
            word: Word to analyze.

        Returns:
            Normalized distance (0.0 = likely typo, 1.0 = intentional).
        """
        if len(word) < 2:
            return 0.5

        # Decompose Korean characters to jamo (simplified)
        # This is a placeholder - full implementation would use proper decomposition
        distances = []

        for i in range(len(word) - 1):
            char1, char2 = word[i], word[i + 1]
            # Simplified distance calculation
            distances.append(abs(ord(char1) - ord(char2)) / 100.0)

        avg_distance = sum(distances) / len(distances) if distances else 0.5
        return min(1.0, avg_distance)

    def _calculate_context_coherence(self, candidate: CandidateWord) -> float:
        """Calculate how coherent the word is in its contexts.

        Args:
            candidate: Candidate word.

        Returns:
            Coherence score (0.0 to 1.0).
        """
        contexts = candidate.contexts or []
        if len(contexts) < 2:
            return 0.5

        # Simple heuristic: words appearing in similar contexts are more coherent
        # This is a simplified version - could use word embeddings in practice
        avg_context_length = sum(len(ctx.split()) for ctx in contexts) / len(contexts)

        # Longer contexts suggest more coherent usage
        coherence = min(1.0, avg_context_length / 20.0)
        return coherence

    def _normalize_frequency(self, frequency: int) -> float:
        """Normalize frequency to 0-1 scale.

        Args:
            frequency: Raw frequency count.

        Returns:
            Normalized value.
        """
        # Log scale normalization
        if frequency <= 0:
            return 0.0

        normalized = math.log(frequency + 1) / math.log(1000)
        return min(1.0, normalized)

    def _normalize_context_diversity(self, context_count: int) -> float:
        """Normalize context diversity to 0-1 scale.

        Args:
            context_count: Number of unique contexts.

        Returns:
            Normalized value.
        """
        if context_count <= 0:
            return 0.0

        normalized = math.log(context_count + 1) / math.log(100)
        return min(1.0, normalized)

    def _check_morphological_validity(self, word: str) -> float:
        """Check if word follows Korean morphological patterns.

        Args:
            word: Word to check.

        Returns:
            Validity score (0.0 to 1.0).
        """
        if not self.use_konlpy or not self.tagger:
            return 0.5  # Unknown

        try:
            # Try to analyze with KoNLPy
            morphs = self.tagger.morphs(word)

            # If KoNLPy can split it into known morphemes, less likely to be neologism
            if len(morphs) > 1:
                return 0.3
            else:
                # Single unknown morpheme might be neologism
                return 0.7

        except Exception:
            return 0.5

    def _is_likely_typo(
        self,
        candidate: CandidateWord,
        features: dict[str, float],
    ) -> bool:
        """Determine if candidate is likely a typo.

        Args:
            candidate: Candidate word.
            features: Extracted features.

        Returns:
            True if likely typo, False otherwise.
        """
        # Low frequency + low keyboard distance suggests typo
        if features["frequency"] < 0.2 and features["keyboard_distance"] < 0.3:
            return True

        # Very low context diversity suggests typo
        if features["context_diversity"] < 0.1:
            return True

        return False

    def get_statistics(self) -> dict[str, Any]:
        """Get classification statistics.

        Returns:
            Dictionary containing statistics.
        """
        total = self.session.query(CandidateWord).count()
        classified = self.session.query(CandidateWord).filter(
            CandidateWord.is_neologism.isnot(None)
        ).count()
        neologisms = self.session.query(CandidateWord).filter(
            CandidateWord.is_neologism == True  # noqa: E712
        ).count()
        typos = self.session.query(CandidateWord).filter(
            CandidateWord.is_typo == True  # noqa: E712
        ).count()

        avg_confidence = self.session.query(
            func.avg(CandidateWord.confidence_score)
        ).scalar() or 0.0

        return {
            "total_candidates": total,
            "classified": classified,
            "neologisms": neologisms,
            "typos": typos,
            "avg_confidence": float(avg_confidence),
        }
