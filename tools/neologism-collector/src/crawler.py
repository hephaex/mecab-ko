"""Web crawlers for collecting Korean text data."""

from __future__ import annotations

import re
import time
from abc import ABC, abstractmethod
from datetime import datetime
from typing import Any
from urllib.parse import urljoin, urlparse
from urllib.robotparser import RobotFileParser

import backoff
import requests
from bs4 import BeautifulSoup
from loguru import logger
from ratelimit import limits, sleep_and_retry

from .config import Config, get_config


class BaseCrawler(ABC):
    """Base class for all crawlers."""

    def __init__(self, config: Config | None = None) -> None:
        """Initialize crawler.

        Args:
            config: Configuration object. If None, uses global config.
        """
        self.config = config or get_config()
        self.session = self._create_session()
        self.robot_parser: RobotFileParser | None = None

    def _create_session(self) -> requests.Session:
        """Create and configure requests session.

        Returns:
            Configured requests session.
        """
        session = requests.Session()
        session.headers.update(
            {
                "User-Agent": self.config.crawler.user_agent,
                "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                "Accept-Language": "ko-KR,ko;q=0.9,en-US;q=0.8,en;q=0.7",
            }
        )
        return session

    def _check_robots_txt(self, url: str) -> bool:
        """Check if URL is allowed by robots.txt.

        Args:
            url: URL to check.

        Returns:
            True if allowed, False otherwise.
        """
        if not self.config.crawler.respect_robots_txt:
            return True

        if self.robot_parser is None:
            parsed_url = urlparse(url)
            robots_url = f"{parsed_url.scheme}://{parsed_url.netloc}/robots.txt"

            self.robot_parser = RobotFileParser()
            self.robot_parser.set_url(robots_url)
            try:
                self.robot_parser.read()
            except Exception as e:
                logger.warning(f"Failed to read robots.txt from {robots_url}: {e}")
                return True

        return self.robot_parser.can_fetch(self.config.crawler.user_agent, url)

    @sleep_and_retry
    @limits(calls=1, period=1)  # Will be configured per instance
    def _rate_limited_get(self, url: str, **kwargs: Any) -> requests.Response:
        """Rate-limited HTTP GET request.

        Args:
            url: URL to fetch.
            **kwargs: Additional arguments for requests.get.

        Returns:
            Response object.
        """
        return self.session.get(url, **kwargs)

    @backoff.on_exception(
        backoff.expo,
        (requests.exceptions.RequestException,),
        max_tries=3,
        max_time=300,
    )
    def fetch_url(self, url: str) -> str | None:
        """Fetch URL with retries and error handling.

        Args:
            url: URL to fetch.

        Returns:
            Page content as string, or None if failed.
        """
        if not self._check_robots_txt(url):
            logger.warning(f"URL blocked by robots.txt: {url}")
            return None

        try:
            response = self._rate_limited_get(
                url,
                timeout=(
                    self.config.crawler.timeout.connect,
                    self.config.crawler.timeout.read,
                ),
            )
            response.raise_for_status()
            response.encoding = response.apparent_encoding
            return response.text

        except requests.exceptions.RequestException as e:
            logger.error(f"Failed to fetch {url}: {e}")
            return None

    @abstractmethod
    def collect(self) -> list[dict[str, Any]]:
        """Collect data from source.

        Returns:
            List of collected items with metadata.
        """
        pass


class NaverNewsCrawler(BaseCrawler):
    """Crawler for Naver News."""

    SECTION_URLS = {
        "politics": "https://news.naver.com/main/main.naver?mode=LSD&mid=shm&sid1=100",
        "economy": "https://news.naver.com/main/main.naver?mode=LSD&mid=shm&sid1=101",
        "society": "https://news.naver.com/main/main.naver?mode=LSD&mid=shm&sid1=102",
        "culture": "https://news.naver.com/main/main.naver?mode=LSD&mid=shm&sid1=103",
        "it": "https://news.naver.com/main/main.naver?mode=LSD&mid=shm&sid1=105",
    }

    def __init__(self, config: Config | None = None) -> None:
        """Initialize Naver News crawler."""
        super().__init__(config)
        self.source_config = self.config.sources.get("naver_news", {})

    def collect(self) -> list[dict[str, Any]]:
        """Collect articles from Naver News.

        Returns:
            List of article dictionaries with title, content, url, etc.
        """
        if not self.source_config.get("enabled", True):
            logger.info("Naver News crawler is disabled")
            return []

        articles: list[dict[str, Any]] = []
        sections = self.source_config.get("sections", ["it", "society"])
        max_articles = self.source_config.get("max_articles_per_section", 50)

        for section in sections:
            logger.info(f"Collecting from Naver News section: {section}")
            section_articles = self._collect_section(section, max_articles)
            articles.extend(section_articles)
            logger.info(f"Collected {len(section_articles)} articles from {section}")

        return articles

    def _collect_section(self, section: str, max_articles: int) -> list[dict[str, Any]]:
        """Collect articles from a specific section.

        Args:
            section: Section name (politics, economy, etc.).
            max_articles: Maximum number of articles to collect.

        Returns:
            List of article dictionaries.
        """
        articles: list[dict[str, Any]] = []
        section_url = self.SECTION_URLS.get(section)

        if not section_url:
            logger.warning(f"Unknown section: {section}")
            return articles

        html = self.fetch_url(section_url)
        if not html:
            return articles

        soup = BeautifulSoup(html, "lxml")

        # Find article links (this is a simplified example)
        # In practice, you'd need to inspect Naver's current HTML structure
        article_links = soup.select("div.list_body a")[:max_articles]

        for link in article_links:
            article_url = link.get("href")
            if not article_url:
                continue

            if not article_url.startswith("http"):
                article_url = urljoin(section_url, article_url)

            article_data = self._extract_article(article_url, section)
            if article_data:
                articles.append(article_data)

            # Rate limiting
            time.sleep(1.0 / self.config.crawler.rate_limit.requests_per_second)

        return articles

    def _extract_article(self, url: str, section: str) -> dict[str, Any] | None:
        """Extract article content from URL.

        Args:
            url: Article URL.
            section: Section name.

        Returns:
            Article data dictionary, or None if extraction failed.
        """
        html = self.fetch_url(url)
        if not html:
            return None

        soup = BeautifulSoup(html, "lxml")

        # Extract title
        title_elem = soup.select_one("h2#title_area, h3#articleTitle")
        title = title_elem.get_text(strip=True) if title_elem else ""

        # Extract content
        content_elem = soup.select_one("article#dic_area, div#articleBodyContents")
        if content_elem:
            # Remove script and style elements
            for script in content_elem(["script", "style"]):
                script.decompose()
            content = content_elem.get_text(separator=" ", strip=True)
        else:
            content = ""

        # Extract date
        date_elem = soup.select_one("span.media_end_head_info_datestamp_time")
        published_at = date_elem.get("data-date-time") if date_elem else None

        if not content:
            return None

        return {
            "source": "naver_news",
            "url": url,
            "title": title,
            "content": content,
            "metadata": {
                "section": section,
                "published_at": published_at,
            },
            "collected_at": datetime.now(),
        }


class WikipediaCrawler(BaseCrawler):
    """Crawler for Korean Wikipedia."""

    BASE_URL = "https://ko.wikipedia.org"

    def __init__(self, config: Config | None = None) -> None:
        """Initialize Wikipedia crawler."""
        super().__init__(config)
        self.source_config = self.config.sources.get("wikipedia", {})

    def collect(self) -> list[dict[str, Any]]:
        """Collect pages from Korean Wikipedia.

        Returns:
            List of page dictionaries with title, content, url, etc.
        """
        if not self.source_config.get("enabled", True):
            logger.info("Wikipedia crawler is disabled")
            return []

        pages: list[dict[str, Any]] = []
        max_pages = self.source_config.get("max_pages", 100)

        # Collect from Recent Changes
        logger.info("Collecting from Wikipedia Recent Changes")
        recent_pages = self._collect_recent_changes(max_pages)
        pages.extend(recent_pages)

        return pages

    def _collect_recent_changes(self, max_pages: int) -> list[dict[str, Any]]:
        """Collect recently changed pages.

        Args:
            max_pages: Maximum number of pages to collect.

        Returns:
            List of page dictionaries.
        """
        pages: list[dict[str, Any]] = []

        # Use Wikipedia API
        api_url = f"{self.BASE_URL}/w/api.php"
        params = {
            "action": "query",
            "list": "recentchanges",
            "rcnamespace": "0",  # Article namespace
            "rclimit": min(max_pages, 500),
            "rctype": "edit|new",
            "format": "json",
        }

        try:
            response = self.session.get(api_url, params=params, timeout=30)
            response.raise_for_status()
            data = response.json()

            for change in data.get("query", {}).get("recentchanges", []):
                page_title = change.get("title")
                if not page_title:
                    continue

                page_data = self._extract_page(page_title)
                if page_data:
                    pages.append(page_data)

                if len(pages) >= max_pages:
                    break

                # Rate limiting
                time.sleep(1.0 / self.config.crawler.rate_limit.requests_per_second)

        except Exception as e:
            logger.error(f"Failed to fetch recent changes: {e}")

        return pages

    def _extract_page(self, title: str) -> dict[str, Any] | None:
        """Extract page content from Wikipedia.

        Args:
            title: Page title.

        Returns:
            Page data dictionary, or None if extraction failed.
        """
        api_url = f"{self.BASE_URL}/w/api.php"
        params = {
            "action": "query",
            "titles": title,
            "prop": "extracts|info",
            "exintro": False,
            "explaintext": True,
            "inprop": "url",
            "format": "json",
        }

        try:
            response = self.session.get(api_url, params=params, timeout=30)
            response.raise_for_status()
            data = response.json()

            pages_data = data.get("query", {}).get("pages", {})
            if not pages_data:
                return None

            # Get first (and only) page
            page_data = next(iter(pages_data.values()))

            content = page_data.get("extract", "")
            if not content or len(content) < 100:
                return None

            return {
                "source": "wikipedia",
                "url": page_data.get("fullurl", ""),
                "title": title,
                "content": content,
                "metadata": {
                    "page_id": page_data.get("pageid"),
                },
                "collected_at": datetime.now(),
            }

        except Exception as e:
            logger.error(f"Failed to extract page {title}: {e}")
            return None


def clean_text(text: str) -> str:
    """Clean and normalize text.

    Args:
        text: Raw text.

    Returns:
        Cleaned text.
    """
    # Remove extra whitespace
    text = re.sub(r"\s+", " ", text)

    # Remove special characters except Korean, English, numbers, and basic punctuation
    text = re.sub(r"[^\w\s가-힣a-zA-Z0-9.,!?;:\-\'\"()]", "", text)

    return text.strip()
