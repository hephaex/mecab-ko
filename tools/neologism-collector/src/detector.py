"""Neologism detection algorithms."""

from __future__ import annotations

import re
from collections import Counter, defaultdict
from datetime import datetime
from typing import Any

from loguru import logger
from sqlalchemy import select
from sqlalchemy.orm import Session

from .config import Config, get_config
from .models import CandidateWord, RawText


class NeologismDetector:
    """Detector for identifying potential neologisms in text."""

    # Korean character ranges
    HANGUL_PATTERN = re.compile(r"[가-힣]+")
    ENGLISH_PATTERN = re.compile(r"[a-zA-Z]+")
    NUMBER_PATTERN = re.compile(r"[0-9]+")
    SPECIAL_PATTERN = re.compile(r"[^\w\s가-힣a-zA-Z0-9]")

    # Common word patterns to filter
    FILTER_PATTERNS = [
        re.compile(r"^[ㄱ-ㅎㅏ-ㅣ]+$"),  # Only jamo
        re.compile(r"^(.)\1{2,}$"),  # Repeated characters (ㅋㅋㅋ)
        re.compile(r"^\d+$"),  # Only numbers
    ]

    def __init__(
        self,
        session: Session,
        config: Config | None = None,
        mecab_dict: set[str] | None = None,
    ) -> None:
        """Initialize detector.

        Args:
            session: SQLAlchemy database session.
            config: Configuration object. If None, uses global config.
            mecab_dict: Set of known words from MeCab dictionary.
                       If None, all words are considered candidates.
        """
        self.session = session
        self.config = config or get_config()
        self.detector_config = self.config.detector
        self.mecab_dict = mecab_dict or set()

        # Initialize KoNLPy (optional, for better tokenization)
        try:
            from konlpy.tag import Okt
            self.tokenizer = Okt()
            self.use_konlpy = True
            logger.info("KoNLPy initialized for tokenization")
        except ImportError:
            self.tokenizer = None
            self.use_konlpy = False
            logger.warning("KoNLPy not available. Using simple tokenization.")

    def detect_from_raw_texts(self, batch_size: int = 100) -> int:
        """Detect neologisms from unprocessed raw texts.

        Args:
            batch_size: Number of texts to process in one batch.

        Returns:
            Number of candidate words created/updated.
        """
        total_candidates = 0

        # Get unprocessed texts
        stmt = (
            select(RawText)
            .where(RawText.processed == False)  # noqa: E712
            .limit(batch_size)
        )
        raw_texts = self.session.execute(stmt).scalars().all()

        if not raw_texts:
            logger.info("No unprocessed texts found")
            return 0

        logger.info(f"Processing {len(raw_texts)} raw texts")

        for raw_text in raw_texts:
            try:
                candidates = self.extract_candidates(
                    raw_text.content,
                    source=raw_text.source,
                    url=raw_text.url,
                )

                # Update or create candidate words
                for surface, data in candidates.items():
                    self._update_or_create_candidate(surface, data)
                    total_candidates += 1

                # Mark as processed
                raw_text.processed = True
                raw_text.processed_at = datetime.now()

            except Exception as e:
                logger.error(f"Failed to process text {raw_text.id}: {e}")
                continue

        self.session.commit()
        logger.info(f"Created/updated {total_candidates} candidate words")

        return total_candidates

    def extract_candidates(
        self,
        text: str,
        source: str | None = None,
        url: str | None = None,
    ) -> dict[str, dict[str, Any]]:
        """Extract candidate neologisms from text.

        Args:
            text: Text to analyze.
            source: Source of the text (for metadata).
            url: URL of the text (for metadata).

        Returns:
            Dictionary mapping surface forms to candidate data.
        """
        candidates: dict[str, dict[str, Any]] = {}

        # Tokenize text
        words = self._tokenize(text)

        # Extract contexts (sliding window)
        contexts = self._extract_contexts(text)

        for word in words:
            # Filter invalid words
            if not self._is_valid_candidate(word):
                continue

            # Check if word is in dictionary
            if word in self.mecab_dict:
                continue

            # Extract features
            features = self._extract_features(word)

            # Initialize or update candidate
            if word not in candidates:
                candidates[word] = {
                    "frequency": 0,
                    "contexts": [],
                    "sources": set(),
                    "features": features,
                }

            candidates[word]["frequency"] += 1
            if source:
                candidates[word]["sources"].add(source)

        # Add contexts
        for word, context in contexts.items():
            if word in candidates:
                candidates[word]["contexts"].append(context)

        # Filter by minimum frequency
        min_freq = self.detector_config.min_frequency
        candidates = {
            word: data
            for word, data in candidates.items()
            if data["frequency"] >= min_freq
        }

        return candidates

    def _tokenize(self, text: str) -> list[str]:
        """Tokenize text into words.

        Args:
            text: Text to tokenize.

        Returns:
            List of tokens.
        """
        if self.use_konlpy and self.tokenizer:
            # Use KoNLPy for better tokenization
            try:
                morphs = self.tokenizer.morphs(text)
                return morphs
            except Exception as e:
                logger.warning(f"KoNLPy tokenization failed: {e}. Using fallback.")

        # Fallback: simple regex tokenization
        words = []

        # Extract Korean words
        korean_words = self.HANGUL_PATTERN.findall(text)
        words.extend(korean_words)

        # Extract mixed Korean-English words
        mixed_pattern = re.compile(r"[가-힣a-zA-Z0-9]+")
        mixed_words = mixed_pattern.findall(text)
        words.extend(mixed_words)

        return words

    def _extract_contexts(self, text: str, window_size: int = 5) -> dict[str, list[str]]:
        """Extract context windows for each word.

        Args:
            text: Text to analyze.
            window_size: Number of words before/after target word.

        Returns:
            Dictionary mapping words to their contexts.
        """
        contexts: dict[str, list[str]] = defaultdict(list)
        words = text.split()

        for i, word in enumerate(words):
            # Extract Korean content from word
            korean_matches = self.HANGUL_PATTERN.findall(word)
            if not korean_matches:
                continue

            target = korean_matches[0]

            # Extract context window
            start = max(0, i - window_size)
            end = min(len(words), i + window_size + 1)
            context = " ".join(words[start:end])

            contexts[target].append(context)

        return contexts

    def _is_valid_candidate(self, word: str) -> bool:
        """Check if word is a valid neologism candidate.

        Args:
            word: Word to validate.

        Returns:
            True if valid, False otherwise.
        """
        # Check length
        if len(word) < self.detector_config.min_length:
            return False
        if len(word) > self.detector_config.max_length:
            return False

        # Check if contains Korean
        if not self.HANGUL_PATTERN.search(word):
            return False

        # Check filter patterns
        for pattern in self.FILTER_PATTERNS:
            if pattern.match(word):
                return False

        # Check special characters
        if not self.detector_config.allow_special_chars:
            if self.SPECIAL_PATTERN.search(word):
                return False

        return True

    def _extract_features(self, word: str) -> dict[str, Any]:
        """Extract features from word.

        Args:
            word: Word to analyze.

        Returns:
            Dictionary of features.
        """
        return {
            "length": len(word),
            "has_hangul": bool(self.HANGUL_PATTERN.search(word)),
            "has_english": bool(self.ENGLISH_PATTERN.search(word)),
            "has_number": bool(self.NUMBER_PATTERN.search(word)),
            "has_special": bool(self.SPECIAL_PATTERN.search(word)),
        }

    def _update_or_create_candidate(
        self,
        surface: str,
        data: dict[str, Any],
    ) -> CandidateWord:
        """Update existing or create new candidate word.

        Args:
            surface: Surface form of the word.
            data: Candidate data including frequency, contexts, etc.

        Returns:
            Updated or created CandidateWord instance.
        """
        # Get existing candidate
        stmt = select(CandidateWord).where(CandidateWord.surface == surface)
        candidate = self.session.execute(stmt).scalar_one_or_none()

        if candidate:
            # Update existing
            candidate.frequency += data["frequency"]
            candidate.last_seen = datetime.now()

            # Update contexts (keep unique)
            existing_contexts = set(candidate.contexts or [])
            new_contexts = existing_contexts.union(data.get("contexts", []))
            candidate.contexts = list(new_contexts)[:100]  # Limit to 100 contexts

            # Update sources
            existing_sources = set(candidate.sources or [])
            new_sources = existing_sources.union(data.get("sources", set()))
            candidate.sources = list(new_sources)

        else:
            # Create new
            features = data.get("features", {})
            candidate = CandidateWord(
                surface=surface,
                frequency=data["frequency"],
                first_seen=datetime.now(),
                last_seen=datetime.now(),
                contexts=data.get("contexts", [])[:100],
                sources=list(data.get("sources", set())),
                length=features.get("length", len(surface)),
                has_hangul=features.get("has_hangul", True),
                has_english=features.get("has_english", False),
                has_number=features.get("has_number", False),
                has_special=features.get("has_special", False),
            )
            self.session.add(candidate)

        return candidate

    def get_statistics(self) -> dict[str, Any]:
        """Get detection statistics.

        Returns:
            Dictionary containing statistics.
        """
        total_candidates = self.session.query(CandidateWord).count()
        classified = self.session.query(CandidateWord).filter(
            CandidateWord.is_neologism.isnot(None)
        ).count()
        neologisms = self.session.query(CandidateWord).filter(
            CandidateWord.is_neologism == True  # noqa: E712
        ).count()
        typos = self.session.query(CandidateWord).filter(
            CandidateWord.is_typo == True  # noqa: E712
        ).count()

        return {
            "total_candidates": total_candidates,
            "classified": classified,
            "neologisms": neologisms,
            "typos": typos,
            "unclassified": total_candidates - classified,
        }
