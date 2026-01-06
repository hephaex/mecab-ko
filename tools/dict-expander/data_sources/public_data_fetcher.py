"""Public data API fetcher for Korean dictionary expansion.

Fetches data from Korean public data sources including:
- data.go.kr (공공데이터포털)
- Korean address data
- Government organization data
"""

from __future__ import annotations

import json
import urllib.request
import urllib.parse
import urllib.error
from dataclasses import dataclass
from typing import Iterator
from pathlib import Path
import time


@dataclass
class PublicDataRecord:
    """Public data record.

    Attributes:
        name: Entity name.
        category: Data category.
        metadata: Additional metadata.
    """

    name: str
    category: str
    metadata: dict[str, str] | None = None


class PublicDataFetcher:
    """Fetches data from Korean public data sources.

    Note: Requires API key from data.go.kr for some endpoints.
    """

    DATA_GO_KR_URL = "https://www.data.go.kr/data/15063424/openapi.do"

    def __init__(
        self,
        api_key: str | None = None,
        rate_limit: float = 1.0,
    ):
        """Initialize public data fetcher.

        Args:
            api_key: API key for data.go.kr (optional).
            rate_limit: Minimum seconds between requests.
        """
        self.api_key = api_key
        self.rate_limit = rate_limit
        self.last_request_time = 0.0

    def fetch_addresses(
        self,
        limit: int | None = None,
    ) -> Iterator[str]:
        """Fetch Korean address data.

        Note: This is a simplified implementation.
        Real implementation would use proper address API.

        Args:
            limit: Maximum addresses to fetch.

        Yields:
            Address strings.
        """
        # Sample data (in production, fetch from real API)
        sample_addresses = [
            "서울특별시", "부산광역시", "대구광역시", "인천광역시",
            "광주광역시", "대전광역시", "울산광역시", "세종특별자치시",
            "경기도", "강원도", "충청북도", "충청남도",
            "전라북도", "전라남도", "경상북도", "경상남도", "제주특별자치도",
        ]

        count = 0
        for address in sample_addresses:
            if limit and count >= limit:
                break
            yield address
            count += 1

    def fetch_organizations(
        self,
        category: str | None = None,
        limit: int | None = None,
    ) -> Iterator[PublicDataRecord]:
        """Fetch organization data.

        Args:
            category: Organization category filter.
            limit: Maximum records to fetch.

        Yields:
            Organization records.
        """
        # Sample organization data
        sample_orgs = [
            ("국회", "정부기관"),
            ("대법원", "정부기관"),
            ("헌법재판소", "정부기관"),
            ("중앙선거관리위원회", "정부기관"),
            ("감사원", "정부기관"),
            ("기획재정부", "정부기관"),
            ("교육부", "정부기관"),
            ("과학기술정보통신부", "정부기관"),
            ("외교부", "정부기관"),
            ("통일부", "정부기관"),
            ("법무부", "정부기관"),
            ("국방부", "정부기관"),
            ("행정안전부", "정부기관"),
            ("문화체육관광부", "정부기관"),
            ("농림축산식품부", "정부기관"),
            ("산업통상자원부", "정부기관"),
            ("보건복지부", "정부기관"),
            ("환경부", "정부기관"),
            ("고용노동부", "정부기관"),
            ("여성가족부", "정부기관"),
        ]

        count = 0
        for name, cat in sample_orgs:
            if limit and count >= limit:
                break
            if category and cat != category:
                continue

            yield PublicDataRecord(name=name, category=cat)
            count += 1

    def fetch_from_file(
        self,
        file_path: Path,
        name_field: str = "name",
        category_field: str = "category",
    ) -> Iterator[PublicDataRecord]:
        """Fetch data from local JSON file.

        Args:
            file_path: Path to JSON file.
            name_field: Field name for entity name.
            category_field: Field name for category.

        Yields:
            Data records.
        """
        if not file_path.exists():
            raise FileNotFoundError(f"File not found: {file_path}")

        with open(file_path, encoding="utf-8") as f:
            data = json.load(f)

        if isinstance(data, list):
            for item in data:
                if isinstance(item, dict):
                    name = item.get(name_field, "")
                    category = item.get(category_field, "")
                    if name:
                        yield PublicDataRecord(
                            name=name,
                            category=category,
                            metadata=item,
                        )


def fetch_public_data(
    source: str = "organizations",
    category: str | None = None,
    limit: int | None = None,
) -> list[PublicDataRecord]:
    """Fetch public data (convenience function).

    Args:
        source: Data source ('organizations', 'addresses').
        category: Category filter.
        limit: Maximum records.

    Returns:
        List of records.

    Examples:
        >>> records = fetch_public_data("organizations", limit=10)
        >>> len(records) <= 10
        True
    """
    fetcher = PublicDataFetcher()

    if source == "organizations":
        records = list(fetcher.fetch_organizations(category, limit))
    elif source == "addresses":
        # Convert addresses to records
        addresses = list(fetcher.fetch_addresses(limit))
        records = [
            PublicDataRecord(name=addr, category="지명")
            for addr in addresses
        ]
    else:
        raise ValueError(f"Unknown source: {source}")

    return records
