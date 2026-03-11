"""Reddit data collector for neologisms."""

from __future__ import annotations

import os
from datetime import datetime
from typing import Any

from loguru import logger

try:
    import praw
    PRAW_AVAILABLE = True
except ImportError:
    PRAW_AVAILABLE = False
    logger.warning("praw not installed. Reddit collector will be disabled.")

from .config import Config, get_config


class RedditCollector:
    """Collector for Reddit data."""

    def __init__(self, config: Config | None = None) -> None:
        """Initialize Reddit collector.

        Args:
            config: Configuration object. If None, uses global config.

        Raises:
            ImportError: If praw is not installed.
            ValueError: If required credentials are missing.
        """
        if not PRAW_AVAILABLE:
            msg = "praw is required for Reddit collection. Install with: pip install praw"
            raise ImportError(msg)

        self.config = config or get_config()
        self.source_config = self.config.sources.get("reddit", {})

        if not self.source_config.get("enabled", False):
            logger.info("Reddit collector is disabled")
            return

        # Get credentials from config or environment
        client_id = self.source_config.get("client_id") or os.getenv("REDDIT_CLIENT_ID")
        client_secret = self.source_config.get("client_secret") or os.getenv("REDDIT_CLIENT_SECRET")
        user_agent = self.source_config.get("user_agent", "MeCab-Ko Neologism Collector")

        if not all([client_id, client_secret]):
            msg = "Reddit API credentials are required. Set in config.yaml or environment variables."
            raise ValueError(msg)

        # Initialize Reddit instance
        self.reddit = praw.Reddit(
            client_id=client_id,
            client_secret=client_secret,
            user_agent=user_agent,
        )

        logger.info("Reddit authentication successful")

    def collect(self) -> list[dict[str, Any]]:
        """Collect posts from configured subreddits.

        Returns:
            List of post dictionaries with title, content, metadata, etc.
        """
        if not self.source_config.get("enabled", False):
            logger.info("Reddit collector is disabled")
            return []

        posts_data: list[dict[str, Any]] = []
        subreddits = self.source_config.get("subreddits", ["hanguk", "korea"])
        max_posts = self.source_config.get("max_posts", 500)

        for subreddit_name in subreddits:
            logger.info(f"Collecting from subreddit: r/{subreddit_name}")
            subreddit_posts = self._collect_subreddit(
                subreddit_name, max_posts // len(subreddits)
            )
            posts_data.extend(subreddit_posts)
            logger.info(f"Collected {len(subreddit_posts)} posts from r/{subreddit_name}")

        return posts_data

    def _collect_subreddit(self, subreddit_name: str, max_posts: int) -> list[dict[str, Any]]:
        """Collect posts from a specific subreddit.

        Args:
            subreddit_name: Name of the subreddit (without r/).
            max_posts: Maximum number of posts to collect.

        Returns:
            List of post dictionaries.
        """
        posts_data: list[dict[str, Any]] = []

        try:
            subreddit = self.reddit.subreddit(subreddit_name)

            # Collect from hot, new, and top
            sources = [
                ("hot", subreddit.hot(limit=max_posts // 3)),
                ("new", subreddit.new(limit=max_posts // 3)),
                ("top", subreddit.top(time_filter="week", limit=max_posts // 3)),
            ]

            for source_type, posts in sources:
                for post in posts:
                    # Combine title and selftext
                    content = post.title
                    if post.selftext:
                        content += "\n\n" + post.selftext

                    # Only collect posts with Korean content
                    if not self._contains_korean(content):
                        continue

                    posts_data.append({
                        "source": "reddit",
                        "url": f"https://reddit.com{post.permalink}",
                        "title": post.title,
                        "content": content,
                        "metadata": {
                            "post_id": post.id,
                            "subreddit": subreddit_name,
                            "author": str(post.author) if post.author else "[deleted]",
                            "score": post.score,
                            "num_comments": post.num_comments,
                            "created_utc": datetime.fromtimestamp(post.created_utc).isoformat(),
                            "source_type": source_type,
                        },
                        "collected_at": datetime.now(),
                    })

                    # Collect top comments
                    comment_text = self._collect_comments(post, max_comments=10)
                    if comment_text:
                        posts_data.append({
                            "source": "reddit",
                            "url": f"https://reddit.com{post.permalink}",
                            "title": f"Comments on: {post.title}",
                            "content": comment_text,
                            "metadata": {
                                "post_id": post.id,
                                "subreddit": subreddit_name,
                                "type": "comments",
                            },
                            "collected_at": datetime.now(),
                        })

        except Exception as e:
            logger.error(f"Failed to collect from r/{subreddit_name}: {e}")

        return posts_data

    def _collect_comments(self, post: Any, max_comments: int = 10) -> str:
        """Collect top comments from a post.

        Args:
            post: Reddit post object.
            max_comments: Maximum number of comments to collect.

        Returns:
            Combined comment text.
        """
        comments_text: list[str] = []

        try:
            post.comments.replace_more(limit=0)
            for comment in post.comments.list()[:max_comments]:
                if hasattr(comment, "body") and self._contains_korean(comment.body):
                    comments_text.append(comment.body)

        except Exception as e:
            logger.error(f"Failed to collect comments: {e}")

        return "\n\n".join(comments_text)

    @staticmethod
    def _contains_korean(text: str) -> bool:
        """Check if text contains Korean characters.

        Args:
            text: Text to check.

        Returns:
            True if text contains Korean, False otherwise.
        """
        return any("\uac00" <= char <= "\ud7a3" for char in text)
