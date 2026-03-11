# Neologism Collector for MeCab-Ko

A comprehensive Python-based pipeline for collecting, detecting, classifying, and exporting Korean neologisms for MeCab-Ko dictionary.

## Features

- **Multi-Source Collection**: Crawl from Naver News, Wikipedia, Twitter/X, and Reddit
- **Smart Detection**: Identify potential neologisms using frequency and context analysis
- **ML Classification**: Distinguish real neologisms from typos and errors
- **Scoring System**: Rank neologisms by quality, spread, and temporal relevance
- **MeCab Export**: Generate MeCab-compatible dictionary CSV files
- **Automated Scheduling**: Run collection pipeline on configurable schedules
- **Politeness**: Respects robots.txt and implements rate limiting

## Installation

### Prerequisites

- Python 3.10 or higher
- pip or poetry

### Basic Installation

```bash
cd /home/mare/mecab-ko/tools/neologism-collector

# Create virtual environment
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt
```

### Optional Dependencies

For Twitter/X collection:
```bash
pip install tweepy
```

For Reddit collection:
```bash
pip install praw
```

For better Korean NLP:
```bash
# Install KoNLPy and dependencies
pip install konlpy
# Follow KoNLPy installation guide for Java/Mecab dependencies
```

## Configuration

1. Copy the example configuration:
```bash
cp .env.example .env
```

2. Edit `.env` to add your API credentials (optional):
```bash
# For Twitter/X
TWITTER_API_KEY=your_key
TWITTER_API_SECRET=your_secret
TWITTER_ACCESS_TOKEN=your_token
TWITTER_ACCESS_SECRET=your_secret

# For Reddit
REDDIT_CLIENT_ID=your_client_id
REDDIT_CLIENT_SECRET=your_secret
```

3. Customize `config.yaml` for your needs:
- Adjust collection sources and limits
- Configure detection thresholds
- Set scheduling intervals
- Modify scoring weights

## Usage

### Initialize Database

First time setup:
```bash
python main.py init
```

### Run Individual Jobs

Collect from specific source:
```bash
python main.py collect --source naver_news
python main.py collect --source wikipedia
python main.py collect --source all
```

Run pipeline steps:
```bash
python main.py detect      # Detect neologisms from texts
python main.py classify    # Classify candidates
python main.py score       # Score neologisms
python main.py export      # Export to MeCab CSV
```

### Run Complete Pipeline

Execute all steps in sequence:
```bash
python main.py pipeline
```

### Start Automated Scheduler

Run continuous collection on schedule:
```bash
python main.py scheduler
```

This will run jobs according to `config.yaml` schedules.

### View Statistics

```bash
python main.py stats
```

### Export Options

Export with specific confidence level:
```bash
python main.py export --min-confidence high
python main.py export --min-confidence medium --output my_neologisms.csv
```

### Cleanup Old Data

Remove old processed data:
```bash
python main.py cleanup
```

## CLI Reference

```
Usage: main.py [OPTIONS] COMMAND [ARGS]...

Commands:
  init       Initialize database and create tables
  scheduler  Start the collection scheduler
  run        Run a single job immediately
  collect    Collect data from sources
  detect     Detect neologisms from collected texts
  classify   Classify candidate words
  score      Score confirmed neologisms
  export     Export neologisms to MeCab CSV format
  stats      Display collection statistics
  cleanup    Clean up old data
  pipeline   Run complete pipeline
```

## Pipeline Architecture

```
┌─────────────────┐
│   Data Sources  │
│  (News, Wiki,   │
│  Twitter, etc.) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Crawlers      │
│  - Rate Limited │
│  - robots.txt   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Raw Storage   │
│   (SQLite DB)   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Detector     │
│  - Tokenization │
│  - Frequency    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   Classifier    │
│  - Feature Ext. │
│  - Neo vs Typo  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│     Scorer      │
│  - Multi-factor │
│  - Confidence   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Exporter     │
│  - MeCab CSV    │
│  - Statistics   │
└─────────────────┘
```

## Data Flow

1. **Collection**: Crawlers gather text from configured sources
2. **Storage**: Raw texts saved to database
3. **Detection**: Tokenize and extract candidate words
4. **Classification**: Distinguish neologisms from typos
5. **Scoring**: Calculate quality scores using multiple factors
6. **Export**: Generate MeCab-compatible CSV files

## Scoring Algorithm

Neologisms are scored based on:

- **Frequency (30%)**: How often the word appears
- **Context Diversity (25%)**: Number of different contexts
- **Morphological Validity (20%)**: Korean word structure
- **Social Spread (15%)**: Number of different sources
- **Temporal Trend (10%)**: Recency and trend analysis

## Output Format

MeCab-Ko dictionary CSV format:
```
표층형,좌문맥ID,우문맥ID,비용,품사,의미분류1,의미분류2,종성유무,읽기,타입,첫번째품사,마지막품사,원형,*
```

Example:
```csv
갓생,1780,3540,4800,NNG,*,*,F,갓생,*,NNG,NNG,갓생,*
```

## Docker Support

Build and run with Docker:

```bash
docker-compose up -d
```

View logs:
```bash
docker-compose logs -f
```

Stop:
```bash
docker-compose down
```

## Development

### Running Tests

```bash
# Install development dependencies
pip install -r requirements.txt

# Run tests
pytest

# Run with coverage
pytest --cov=src --cov-report=html

# Type checking
mypy src/

# Linting
ruff check src/

# Formatting
black src/
```

### Project Structure

```
neologism-collector/
├── src/
│   ├── __init__.py
│   ├── config.py           # Configuration management
│   ├── models.py           # SQLAlchemy models
│   ├── crawler.py          # Web crawlers
│   ├── twitter_collector.py
│   ├── reddit_collector.py
│   ├── detector.py         # Neologism detection
│   ├── classifier.py       # Classification logic
│   ├── scorer.py           # Scoring system
│   ├── storage.py          # Database operations
│   ├── exporter.py         # MeCab CSV export
│   └── scheduler.py        # Job scheduling
├── data/
│   ├── neologisms.db       # SQLite database
│   └── export/             # Exported CSV files
├── logs/                   # Log files
├── tests/                  # Unit tests
├── config.yaml             # Main configuration
├── requirements.txt        # Python dependencies
├── main.py                 # CLI entry point
├── docker-compose.yml      # Docker setup
└── README.md              # This file
```

## Configuration Options

### Key Settings

- `database.url`: Database connection string
- `crawler.rate_limit.requests_per_second`: Rate limiting
- `crawler.respect_robots_txt`: Honor robots.txt
- `detector.min_frequency`: Minimum word frequency
- `classifier.weights`: Feature weights for classification
- `scorer.thresholds`: Confidence level thresholds
- `storage.retention_days`: Data retention period
- `exporter.default_pos`: Default part-of-speech tag

### Scheduling

Edit `config.yaml` schedules section:

```yaml
schedules:
  - name: "hourly_news"
    job: "collect_naver_news"
    trigger: "interval"
    hours: 1

  - name: "daily_detection"
    job: "detect_neologisms"
    trigger: "cron"
    hour: 3
    minute: 0
```

## Best Practices

1. **Rate Limiting**: Always respect source website limits
2. **robots.txt**: Keep `respect_robots_txt: true` in config
3. **Incremental Collection**: Run regularly rather than large batches
4. **Review Exports**: Manually review high-confidence neologisms
5. **Backup Database**: Regularly backup `data/neologisms.db`

## Troubleshooting

### KoNLPy Installation Issues

If KoNLPy fails to install:
```bash
# Ubuntu/Debian
sudo apt-get install openjdk-11-jdk

# macOS
brew install openjdk@11
```

### Database Locked

If you get "database is locked" errors:
```bash
# Stop scheduler
pkill -f "python main.py scheduler"

# Run vacuum
python main.py cleanup
```

### Low Quality Results

Adjust thresholds in `config.yaml`:
- Increase `detector.min_frequency`
- Increase `scorer.thresholds.medium_confidence`
- Adjust `classifier.weights`

## Contributing

1. Follow PEP 8 style guide
2. Use type hints for all functions
3. Write docstrings (Google style)
4. Add tests for new features
5. Run `black`, `ruff`, and `mypy` before committing

## License

This project is part of MeCab-Ko and follows the same license.

## Credits

- MeCab-Ko Project
- Contributors to source data (Naver, Wikipedia, etc.)
- Python ecosystem (requests, BeautifulSoup, SQLAlchemy, etc.)

## Support

For issues and questions:
- GitHub Issues: [Project Issues Page]
- Documentation: See `docs/` directory in main repo

## Roadmap

- [ ] Machine learning classifier with trained model
- [ ] Support for more data sources
- [ ] Real-time detection API
- [ ] Web dashboard for monitoring
- [ ] Automatic dictionary integration
- [ ] Multi-language support (beyond Korean)
