"""Twitter/X data collector for neologisms."""

from __future__ import annotations

import os
from datetime import datetime
from typing import Any

from loguru import logger

try:
    import tweepy
    TWEEPY_AVAILABLE = True
except ImportError:
    TWEEPY_AVAILABLE = False
    logger.warning("tweepy not installed. Twitter collector will be disabled.")

from .config import Config, get_config


class TwitterCollector:
    """Collector for Twitter/X data."""

    def __init__(self, config: Config | None = None) -> None:
        """Initialize Twitter collector.

        Args:
            config: Configuration object. If None, uses global config.

        Raises:
            ImportError: If tweepy is not installed.
            ValueError: If required credentials are missing.
        """
        if not TWEEPY_AVAILABLE:
            msg = "tweepy is required for Twitter collection. Install with: pip install tweepy"
            raise ImportError(msg)

        self.config = config or get_config()
        self.source_config = self.config.sources.get("twitter", {})

        if not self.source_config.get("enabled", False):
            logger.info("Twitter collector is disabled")
            return

        # Get credentials from config or environment
        api_key = self.source_config.get("api_key") or os.getenv("TWITTER_API_KEY")
        api_secret = self.source_config.get("api_secret") or os.getenv("TWITTER_API_SECRET")
        access_token = self.source_config.get("access_token") or os.getenv("TWITTER_ACCESS_TOKEN")
        access_secret = self.source_config.get("access_secret") or os.getenv("TWITTER_ACCESS_SECRET")

        if not all([api_key, api_secret, access_token, access_secret]):
            msg = "Twitter API credentials are required. Set in config.yaml or environment variables."
            raise ValueError(msg)

        # Authenticate
        auth = tweepy.OAuthHandler(api_key, api_secret)
        auth.set_access_token(access_token, access_secret)
        self.api = tweepy.API(auth, wait_on_rate_limit=True)

        # Verify credentials
        try:
            self.api.verify_credentials()
            logger.info("Twitter authentication successful")
        except Exception as e:
            logger.error(f"Twitter authentication failed: {e}")
            raise

    def collect(self) -> list[dict[str, Any]]:
        """Collect tweets based on configured keywords.

        Returns:
            List of tweet dictionaries with text, metadata, etc.
        """
        if not self.source_config.get("enabled", False):
            logger.info("Twitter collector is disabled")
            return []

        tweets_data: list[dict[str, Any]] = []
        keywords = self.source_config.get("keywords", ["한국어", "신조어"])
        max_tweets = self.source_config.get("max_tweets", 1000)

        for keyword in keywords:
            logger.info(f"Collecting tweets for keyword: {keyword}")
            keyword_tweets = self._collect_keyword(keyword, max_tweets // len(keywords))
            tweets_data.extend(keyword_tweets)
            logger.info(f"Collected {len(keyword_tweets)} tweets for {keyword}")

        return tweets_data

    def _collect_keyword(self, keyword: str, max_tweets: int) -> list[dict[str, Any]]:
        """Collect tweets for a specific keyword.

        Args:
            keyword: Search keyword.
            max_tweets: Maximum number of tweets to collect.

        Returns:
            List of tweet dictionaries.
        """
        tweets_data: list[dict[str, Any]] = []

        try:
            # Search for tweets
            tweets = tweepy.Cursor(
                self.api.search_tweets,
                q=keyword,
                lang="ko",
                tweet_mode="extended",
                result_type="recent",
            ).items(max_tweets)

            for tweet in tweets:
                # Extract full text
                text = tweet.full_text if hasattr(tweet, "full_text") else tweet.text

                # Skip retweets
                if text.startswith("RT @"):
                    continue

                tweets_data.append({
                    "source": "twitter",
                    "url": f"https://twitter.com/{tweet.user.screen_name}/status/{tweet.id}",
                    "title": f"Tweet by @{tweet.user.screen_name}",
                    "content": text,
                    "metadata": {
                        "tweet_id": str(tweet.id),
                        "user": tweet.user.screen_name,
                        "created_at": tweet.created_at.isoformat(),
                        "retweet_count": tweet.retweet_count,
                        "favorite_count": tweet.favorite_count,
                        "keyword": keyword,
                    },
                    "collected_at": datetime.now(),
                })

        except Exception as e:
            logger.error(f"Failed to collect tweets for {keyword}: {e}")

        return tweets_data

    def collect_user_timeline(self, username: str, max_tweets: int = 200) -> list[dict[str, Any]]:
        """Collect tweets from a specific user's timeline.

        Args:
            username: Twitter username (without @).
            max_tweets: Maximum number of tweets to collect.

        Returns:
            List of tweet dictionaries.
        """
        tweets_data: list[dict[str, Any]] = []

        try:
            tweets = tweepy.Cursor(
                self.api.user_timeline,
                screen_name=username,
                tweet_mode="extended",
                exclude_replies=True,
                include_rts=False,
            ).items(max_tweets)

            for tweet in tweets:
                text = tweet.full_text if hasattr(tweet, "full_text") else tweet.text

                tweets_data.append({
                    "source": "twitter",
                    "url": f"https://twitter.com/{username}/status/{tweet.id}",
                    "title": f"Tweet by @{username}",
                    "content": text,
                    "metadata": {
                        "tweet_id": str(tweet.id),
                        "user": username,
                        "created_at": tweet.created_at.isoformat(),
                        "retweet_count": tweet.retweet_count,
                        "favorite_count": tweet.favorite_count,
                    },
                    "collected_at": datetime.now(),
                })

        except Exception as e:
            logger.error(f"Failed to collect tweets from @{username}: {e}")

        return tweets_data
