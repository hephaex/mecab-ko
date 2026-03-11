#!/bin/bash
# Quick start script for neologism collector

set -e

echo "=== MeCab-Ko Neologism Collector - Quick Start ==="
echo ""

# Check Python version
python_version=$(python3 --version 2>&1 | awk '{print $2}')
echo "Python version: $python_version"

# Create virtual environment if it doesn't exist
if [ ! -d "venv" ]; then
    echo "Creating virtual environment..."
    python3 -m venv venv
fi

# Activate virtual environment
echo "Activating virtual environment..."
source venv/bin/activate

# Install dependencies
echo "Installing dependencies..."
pip install --upgrade pip
pip install -r requirements.txt

# Copy environment file if it doesn't exist
if [ ! -f ".env" ]; then
    echo "Creating .env file from example..."
    cp .env.example .env
    echo "Please edit .env to add your API credentials if needed"
fi

# Initialize database
echo "Initializing database..."
python main.py init

# Show statistics
echo ""
echo "Current statistics:"
python main.py stats

echo ""
echo "=== Setup Complete ==="
echo ""
echo "Available commands:"
echo "  python main.py collect --source all    # Collect from all sources"
echo "  python main.py pipeline                # Run full pipeline"
echo "  python main.py scheduler               # Start automated scheduler"
echo "  python main.py stats                   # Show statistics"
echo ""
echo "For more information, see README.md"
