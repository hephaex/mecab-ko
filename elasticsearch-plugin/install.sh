#!/bin/bash
#
# MeCab-Ko Elasticsearch Plugin Installation Script
#
# Usage:
#   ./install.sh [elasticsearch-home]
#
# Example:
#   ./install.sh /usr/share/elasticsearch
#   ./install.sh $ES_HOME

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Plugin info
PLUGIN_NAME="mecab-ko-analyzer"
PLUGIN_VERSION="0.1.0"

# Parse arguments
ES_HOME="${1:-$ES_HOME}"
if [ -z "$ES_HOME" ]; then
    echo -e "${RED}Error: Elasticsearch home directory not specified${NC}"
    echo "Usage: $0 [elasticsearch-home]"
    echo "Example: $0 /usr/share/elasticsearch"
    exit 1
fi

if [ ! -d "$ES_HOME" ]; then
    echo -e "${RED}Error: Elasticsearch home directory not found: $ES_HOME${NC}"
    exit 1
fi

# Verify Elasticsearch installation
if [ ! -x "$ES_HOME/bin/elasticsearch" ]; then
    echo -e "${RED}Error: elasticsearch binary not found in $ES_HOME/bin/${NC}"
    exit 1
fi

echo -e "${GREEN}=== MeCab-Ko Elasticsearch Plugin Installer ===${NC}"
echo "Elasticsearch home: $ES_HOME"
echo "Plugin: $PLUGIN_NAME v$PLUGIN_VERSION"
echo

# Check if plugin is already installed
PLUGIN_DIR="$ES_HOME/plugins/$PLUGIN_NAME"
if [ -d "$PLUGIN_DIR" ]; then
    echo -e "${YELLOW}Plugin already installed. Removing old version...${NC}"
    rm -rf "$PLUGIN_DIR"
fi

# Build plugin if necessary
PLUGIN_ZIP="$SCRIPT_DIR/build/distributions/${PLUGIN_NAME}-${PLUGIN_VERSION}.zip"
if [ ! -f "$PLUGIN_ZIP" ]; then
    echo -e "${YELLOW}Plugin package not found. Building...${NC}"
    cd "$SCRIPT_DIR"
    ./gradlew bundlePlugin
    if [ $? -ne 0 ]; then
        echo -e "${RED}Error: Failed to build plugin${NC}"
        exit 1
    fi
fi

# Install plugin using elasticsearch-plugin
echo -e "${GREEN}Installing plugin...${NC}"
if [ -x "$ES_HOME/bin/elasticsearch-plugin" ]; then
    # Use elasticsearch-plugin tool
    "$ES_HOME/bin/elasticsearch-plugin" install "file://$PLUGIN_ZIP"
else
    # Manual installation
    echo -e "${YELLOW}elasticsearch-plugin not found. Installing manually...${NC}"

    mkdir -p "$PLUGIN_DIR"

    # Extract plugin
    unzip -q "$PLUGIN_ZIP" -d "$PLUGIN_DIR"

    # Set permissions
    chmod -R 755 "$PLUGIN_DIR"
fi

# Verify installation
if [ -d "$PLUGIN_DIR" ]; then
    echo -e "${GREEN}✓ Plugin installed successfully${NC}"
    echo
    echo "Installation details:"
    echo "  Plugin directory: $PLUGIN_DIR"
    echo "  Version: $PLUGIN_VERSION"
    echo

    # Check native library
    NATIVE_DIR="$PLUGIN_DIR/native"
    if [ -d "$NATIVE_DIR" ]; then
        echo -e "${GREEN}✓ Native libraries found${NC}"
        ls -lh "$NATIVE_DIR"
    else
        echo -e "${YELLOW}⚠ Native libraries not found in $NATIVE_DIR${NC}"
        echo "  Make sure to build native libraries first:"
        echo "    cd ../rust"
        echo "    cargo build --release --features jni-bindings"
    fi

    echo
    echo -e "${GREEN}Installation complete!${NC}"
    echo
    echo "Next steps:"
    echo "  1. Restart Elasticsearch:"
    echo "     sudo systemctl restart elasticsearch"
    echo "     OR"
    echo "     $ES_HOME/bin/elasticsearch"
    echo
    echo "  2. Verify plugin is loaded:"
    echo "     curl -X GET 'localhost:9200/_cat/plugins?v'"
    echo
    echo "  3. Test the analyzer:"
    echo "     curl -X POST 'localhost:9200/_analyze' -H 'Content-Type: application/json' -d'"
    echo "     {"
    echo "       \"analyzer\": \"mecab_ko\","
    echo "       \"text\": \"한국어 형태소 분석기\""
    echo "     }'"
    echo
else
    echo -e "${RED}✗ Plugin installation failed${NC}"
    exit 1
fi
