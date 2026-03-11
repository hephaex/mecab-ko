"""Wikipedia data fetcher for Korean dictionary expansion.

Fetches article titles and content from Korean Wikipedia
for proper noun extraction (people, places, organizations).
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
class WikipediaArticle:
    """Wikipedia article information.

    Attributes:
        title: Article title.
        page_id: Wikipedia page ID.
        namespace: Article namespace (0 for main).
        categories: Article categories.
    """

    title: str
    page_id: int
    namespace: int = 0
    categories: list[str] | None = None


class WikipediaFetcher:
    """Fetches data from Korean Wikipedia API.

    Uses the MediaWiki API to retrieve article titles and metadata
    for dictionary expansion.
    """

    API_URL = "https://ko.wikipedia.org/w/api.php"
    USER_AGENT = "MeCab-Ko-DictExpander/1.0 (Dictionary Expansion Tool)"

    def __init__(
        self,
        cache_dir: Path | None = None,
        rate_limit: float = 1.0,
    ):
        """Initialize Wikipedia fetcher.

        Args:
            cache_dir: Directory for caching responses (optional).
            rate_limit: Minimum seconds between API requests.
        """
        self.cache_dir = cache_dir
        self.rate_limit = rate_limit
        self.last_request_time = 0.0

        if cache_dir:
            cache_dir.mkdir(parents=True, exist_ok=True)

    def fetch_all_titles(
        self,
        namespace: int = 0,
        limit: int | None = None,
    ) -> Iterator[str]:
        """Fetch all article titles from Wikipedia.

        Args:
            namespace: Wikipedia namespace (0=main articles).
            limit: Maximum number of titles to fetch.

        Yields:
            Article titles.
        """
        params = {
            "action": "query",
            "list": "allpages",
            "aplimit": "500",
            "apnamespace": str(namespace),
            "format": "json",
        }

        count = 0
        continue_token = None

        while True:
            if limit and count >= limit:
                break

            # Add continuation token
            if continue_token:
                params["apcontinue"] = continue_token

            try:
                data = self._api_request(params)

                # Extract titles
                if "query" in data and "allpages" in data["query"]:
                    for page in data["query"]["allpages"]:
                        yield page["title"]
                        count += 1
                        if limit and count >= limit:
                            break

                # Check for continuation
                if "continue" in data:
                    continue_token = data["continue"].get("apcontinue")
                    if not continue_token:
                        break
                else:
                    break

            except Exception as e:
                print(f"Error fetching titles: {e}")
                break

    def fetch_titles_by_category(
        self,
        category: str,
        limit: int | None = None,
    ) -> Iterator[str]:
        """Fetch titles in specific category.

        Args:
            category: Category name (without 'Category:' prefix).
            limit: Maximum number of titles.

        Yields:
            Article titles in category.
        """
        params = {
            "action": "query",
            "list": "categorymembers",
            "cmtitle": f"Category:{category}",
            "cmlimit": "500",
            "format": "json",
        }

        count = 0
        continue_token = None

        while True:
            if limit and count >= limit:
                break

            if continue_token:
                params["cmcontinue"] = continue_token

            try:
                data = self._api_request(params)

                if "query" in data and "categorymembers" in data["query"]:
                    for page in data["query"]["categorymembers"]:
                        yield page["title"]
                        count += 1
                        if limit and count >= limit:
                            break

                if "continue" in data:
                    continue_token = data["continue"].get("cmcontinue")
                    if not continue_token:
                        break
                else:
                    break

            except Exception as e:
                print(f"Error fetching category {category}: {e}")
                break

    def search_titles(
        self,
        query: str,
        limit: int = 100,
    ) -> Iterator[str]:
        """Search for titles matching query.

        Args:
            query: Search query.
            limit: Maximum results.

        Yields:
            Matching titles.
        """
        params = {
            "action": "query",
            "list": "search",
            "srsearch": query,
            "srlimit": str(min(limit, 500)),
            "format": "json",
        }

        try:
            data = self._api_request(params)

            if "query" in data and "search" in data["query"]:
                for result in data["query"]["search"]:
                    yield result["title"]

        except Exception as e:
            print(f"Error searching for '{query}': {e}")

    def _api_request(self, params: dict[str, str]) -> dict:
        """Make API request with rate limiting.

        Args:
            params: API parameters.

        Returns:
            JSON response data.

        Raises:
            urllib.error.URLError: If request fails.
        """
        # Rate limiting
        elapsed = time.time() - self.last_request_time
        if elapsed < self.rate_limit:
            time.sleep(self.rate_limit - elapsed)

        # Build URL
        url = f"{self.API_URL}?{urllib.parse.urlencode(params)}"

        # Make request
        req = urllib.request.Request(url)
        req.add_header("User-Agent", self.USER_AGENT)

        with urllib.request.urlopen(req, timeout=30) as response:
            self.last_request_time = time.time()
            return json.loads(response.read().decode("utf-8"))


def fetch_wikipedia_titles(
    category: str | None = None,
    limit: int | None = None,
    output_file: Path | None = None,
) -> list[str]:
    """Fetch Wikipedia titles (convenience function).

    Args:
        category: Category to fetch from (None = all).
        limit: Maximum titles to fetch.
        output_file: Optional file to save titles.

    Returns:
        List of titles.

    Examples:
        >>> titles = fetch_wikipedia_titles(category="대한민국의_배우", limit=100)
        >>> len(titles) <= 100
        True
    """
    fetcher = WikipediaFetcher()

    if category:
        titles = list(fetcher.fetch_titles_by_category(category, limit))
    else:
        titles = list(fetcher.fetch_all_titles(limit=limit))

    if output_file:
        output_file.write_text("\n".join(titles), encoding="utf-8")

    return titles
