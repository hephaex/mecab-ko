"""Neologism Collector for MeCab-Ko.

A comprehensive pipeline for collecting, detecting, and exporting Korean neologisms.
"""

__version__ = "0.1.0"
__author__ = "MeCab-Ko Team"

from .crawler import NaverNewsCrawler, WikipediaCrawler
from .detector import NeologismDetector
from .classifier import NeologismClassifier
from .scorer import NeologismScorer
from .storage import NeologismStorage
from .exporter import MeCabExporter
from .scheduler import CollectionScheduler

__all__ = [
    "NaverNewsCrawler",
    "WikipediaCrawler",
    "NeologismDetector",
    "NeologismClassifier",
    "NeologismScorer",
    "NeologismStorage",
    "MeCabExporter",
    "CollectionScheduler",
]
