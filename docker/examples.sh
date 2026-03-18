#!/bin/bash
# MeCab-Ko Docker Examples
# Demonstrates common usage patterns for CLI and API containers

set -euo pipefail

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Functions
print_header() {
    echo -e "${BLUE}====================\n$1\n====================${NC}"
}

print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

print_error() {
    echo -e "${RED}✗ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}! $1${NC}"
}

# ============================================================================
# CLI Examples
# ============================================================================

cli_examples() {
    print_header "MeCab-Ko CLI Examples"

    # Example 1: Basic analysis
    print_header "1. Basic Text Analysis"
    echo "Input: 안녕하세요"
    echo "Output:"
    echo "안녕 NNG,*,F,안녕,*,*,*,*,*,*,*,*,*,*,*,*"
    echo "하 XSV,*,F,하,*,*,*,*,*,*,*,*,*,*,*,*"
    echo "세 NNG,*,F,세,*,*,*,*,*,*,*,*,*,*,*,*"
    echo "요 JX,*,F,요,*,*,*,*,*,*,*,*,*,*,*,*"
    echo "EOS"

    # Example 2: Wakati format
    print_header "2. Wakati Format (Space-separated tokens)"
    echo "Command: docker run --rm mecab-ko -O wakati '오늘 날씨가 좋습니다'"
    echo "Output: 오늘 날씨 가 좋 습니다"

    # Example 3: JSON format
    print_header "3. JSON Format Output"
    echo "Command: docker run --rm mecab-ko -O json '한국어 분석'"
    echo "Output:"
    echo "[{\"surface\":\"한국\",\"pos\":\"NNG\",\"reading\":\"한국\",\"lemma\":\"한국\"},"
    echo " {\"surface\":\"어\",\"pos\":\"NNG\",\"reading\":\"어\",\"lemma\":\"어\"},"
    echo " {\"surface\":\"분석\",\"pos\":\"NNG\",\"reading\":\"분석\",\"lemma\":\"분석\"}]"

    # Example 4: CSV format
    print_header "4. CSV Format Output"
    echo "Command: docker run --rm mecab-ko -O csv '형태소 분석'"
    echo "Output:"
    echo "surface,pos,start,end,reading,lemma"
    echo "형태,NNG,0,2,형태,형태"
    echo "소,NNG,2,3,소,소"
    echo "분석,NNG,3,5,분석,분석"

    # Example 5: File processing
    print_header "5. File Processing"
    echo "Command: echo '한국어 텍스트' > input.txt"
    echo "         docker run --rm -v \$(pwd)/input.txt:/input.txt -v \$(pwd)/output:/output mecab-ko -i /input.txt -o /output"

    # Example 6: Interactive REPL
    print_header "6. Interactive Mode"
    echo "Command: docker run --rm -it mecab-ko --repl"
    echo "Usage:"
    echo "  >> 형태소 분석"
    echo "  형태 NNG,*,F,형태,*,*,*,*,*,*,*,*,*,*,*,*"
    echo "  소 NNG,*,F,소,*,*,*,*,*,*,*,*,*,*,*,*"
    echo "  분석 NNG,*,F,분석,*,*,*,*,*,*,*,*,*,*,*,*"
    echo "  EOS"
    echo "  >> exit"

    # Example 7: Pipe input
    print_header "7. Pipe Input"
    echo "Command: echo '자연어 처리' | docker run --rm -i mecab-ko"
    echo "Output: Standard MeCab format"
}

# ============================================================================
# API Server Examples
# ============================================================================

api_examples() {
    print_header "MeCab-Ko API Server Examples"

    # Example 1: Health check
    print_header "1. Health Check"
    echo "Command: curl http://localhost:8000/health"
    echo "Response:"
    echo '{
  "status": "healthy",
  "version": "0.5.0",
  "dictionary_path": "/usr/share/mecab-ko-dic"
}'

    # Example 2: Server info
    print_header "2. Server Information"
    echo "Command: curl http://localhost:8000/info"
    echo "Response:"
    echo '{
  "name": "MeCab-Ko API Server",
  "version": "0.5.0",
  "dictionary_path": "/usr/share/mecab-ko-dic",
  "python_implementation": "CPython with Rust extension",
  "description": "Korean morphological analyzer"
}'

    # Example 3: Analyze text
    print_header "3. Analyze Korean Text"
    echo "Command: curl -X POST http://localhost:8000/analyze \\"
    echo "  -H 'Content-Type: application/json' \\"
    echo "  -d '{\"text\":\"한국어 분석\"}'"
    echo "Response:"
    echo '{
  "success": true,
  "data": {
    "result": "한국 NNG,*,F,...\n어 NNG,*,F,...\n분석 NNG,*,F,...\n"
  },
  "processing_time_ms": 2.34
}'

    # Example 4: Extract morphemes
    print_header "4. Extract Morphemes (Tokens)"
    echo "Command: curl -X POST http://localhost:8000/morphs \\"
    echo "  -H 'Content-Type: application/json' \\"
    echo "  -d '{\"text\":\"오늘 날씨가 좋습니다\"}'"
    echo "Response:"
    echo '{
  "success": true,
  "data": {
    "morphemes": ["오늘", "날씨", "가", "좋", "습니다"],
    "count": 5
  },
  "processing_time_ms": 1.23
}'

    # Example 5: Extract nouns
    print_header "5. Extract Nouns"
    echo "Command: curl -X POST http://localhost:8000/nouns \\"
    echo "  -H 'Content-Type: application/json' \\"
    echo "  -d '{\"text\":\"서울에서 한국 음식을 먹었습니다\"}'"
    echo "Response:"
    echo '{
  "success": true,
  "data": {
    "nouns": ["서울", "한국", "음식"],
    "count": 3
  },
  "processing_time_ms": 1.45
}'

    # Example 6: POS tagging
    print_header "6. Part-of-Speech Tagging"
    echo "Command: curl -X POST http://localhost:8000/pos \\"
    echo "  -H 'Content-Type: application/json' \\"
    echo "  -d '{\"text\":\"나는 학생입니다\"}'"
    echo "Response:"
    echo '{
  "success": true,
  "data": {
    "pos_tags": [
      {"surface": "나", "pos": "NP"},
      {"surface": "는", "pos": "JKB"},
      {"surface": "학생", "pos": "NNG"},
      {"surface": "입니다", "pos": "VCP+EF"}
    ],
    "count": 4
  },
  "processing_time_ms": 1.56
}'

    # Example 7: Batch analysis
    print_header "7. Batch Analysis"
    echo "Command: curl -X POST http://localhost:8000/batch \\"
    echo "  -H 'Content-Type: application/json' \\"
    echo "  -d '{\"texts\": [\"첫 번째\", \"두 번째\", \"세 번째\"]}'"
    echo "Response:"
    echo '{
  "success": true,
  "results": [
    {"success": true, "result": "첫 NNG,..."},
    {"success": true, "result": "두 NNG,..."},
    {"success": true, "result": "세 NNG,..."}
  ],
  "failed": 0,
  "total": 3,
  "processing_time_ms": 3.45
}'

    # Example 8: Python client
    print_header "8. Python Client Example"
    cat << 'EOF'
import httpx
import asyncio

async def main():
    async with httpx.AsyncClient() as client:
        # Single analysis
        response = await client.post(
            "http://localhost:8000/morphs",
            json={"text": "한국어 분석"}
        )
        print(response.json())

        # Batch analysis
        response = await client.post(
            "http://localhost:8000/batch",
            json={
                "texts": ["첫 문장", "둘째 문장", "셋째 문장"]
            }
        )
        print(response.json())

asyncio.run(main())
EOF

    # Example 9: JavaScript client
    print_header "9. JavaScript Client Example"
    cat << 'EOF'
// Node.js with fetch API or axios
const response = await fetch('http://localhost:8000/morphs', {
  method: 'POST',
  headers: {
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    text: '한국어 분석'
  })
});

const data = await response.json();
console.log(data);
EOF

    # Example 10: cURL with file
    print_header "10. File Input with cURL"
    echo "Command: curl -X POST http://localhost:8000/batch \\"
    echo "  -H 'Content-Type: application/json' \\"
    echo "  -d @request.json"
    echo ""
    echo "request.json content:"
    echo '{
  "texts": [
    "첫 번째 한국어 문장입니다",
    "두 번째 한국어 문장입니다",
    "세 번째 한국어 문장입니다"
  ]
}'
}

# ============================================================================
# Docker Compose Examples
# ============================================================================

compose_examples() {
    print_header "Docker Compose Examples"

    # Example 1: Start services
    print_header "1. Start All Services"
    echo "Commands:"
    echo "  cd docker"
    echo "  docker compose up -d"
    echo "  docker compose ps"

    # Example 2: View logs
    print_header "2. View Service Logs"
    echo "Commands:"
    echo "  # All services"
    echo "  docker compose logs -f"
    echo ""
    echo "  # API service only"
    echo "  docker compose logs -f mecab-api"
    echo ""
    echo "  # Last 100 lines"
    echo "  docker compose logs -f --tail=100 mecab-api"

    # Example 3: Execute commands
    print_header "3. Execute Commands in Running Container"
    echo "Commands:"
    echo "  # Check API health"
    echo "  docker compose exec mecab-api curl http://localhost:8000/health"
    echo ""
    echo "  # Interactive shell in CLI container"
    echo "  docker compose exec mecab-cli /bin/bash"

    # Example 4: Stop and cleanup
    print_header "4. Stop and Cleanup"
    echo "Commands:"
    echo "  # Stop services (keep volumes)"
    echo "  docker compose stop"
    echo ""
    echo "  # Remove containers (keep volumes)"
    echo "  docker compose down"
    echo ""
    echo "  # Remove everything including volumes"
    echo "  docker compose down -v"

    # Example 5: Rebuild images
    print_header "5. Rebuild Images"
    echo "Commands:"
    echo "  # Rebuild single image"
    echo "  docker compose up -d --build mecab-api"
    echo ""
    echo "  # Rebuild all images"
    echo "  docker compose up -d --build"
    echo ""
    echo "  # Rebuild without cache"
    echo "  docker compose build --no-cache"
}

# ============================================================================
# Performance Testing Examples
# ============================================================================

performance_examples() {
    print_header "Performance Testing Examples"

    # Example 1: Single request timing
    print_header "1. Single Request Timing"
    echo "Command: curl -w 'Processing time: %{time_total}s\\n' \\"
    echo "  -X POST http://localhost:8000/morphs \\"
    echo "  -H 'Content-Type: application/json' \\"
    echo "  -d '{\"text\":\"한국어 분석\"}'"

    # Example 2: Load testing with Apache Bench
    print_header "2. Load Testing with Apache Bench"
    echo "Installation: apt-get install apache2-utils"
    echo ""
    echo "Commands:"
    echo "  # 1000 requests, 10 concurrent"
    echo "  ab -n 1000 -c 10 -p payload.json -T application/json \\"
    echo "    http://localhost:8000/morphs"

    # Example 3: Load testing with wrk
    print_header "3. Load Testing with wrk"
    echo "Installation: https://github.com/wg/wrk"
    echo ""
    echo "Commands:"
    echo "  # 4 threads, 10 concurrent connections, 30 second test"
    echo "  wrk -t4 -c10 -d30s \\"
    echo "    -s script.lua \\"
    echo "    http://localhost:8000"

    # Example 4: Batch performance
    print_header "4. Batch Request Performance"
    echo "Command: curl -w 'Processing time: %{time_total}s\\n' \\"
    echo "  -X POST http://localhost:8000/batch \\"
    echo "  -H 'Content-Type: application/json' \\"
    echo "  -d '{\"texts\": [\"텍스트1\", \"텍스트2\", ..., \"텍스트1000\"]}'"

    # Example 5: Monitor container resources
    print_header "5. Monitor Container Resources"
    echo "Command: docker stats --format \\"
    echo "  'table {{.Container}}\\t{{.CPUPerc}}\\t{{.MemUsage}}\\t{{.NetIO}}' \\"
    echo "  mecab-api"
}

# ============================================================================
# Main Menu
# ============================================================================

main() {
    if [ $# -eq 0 ]; then
        cat << EOF
MeCab-Ko Docker Examples

Usage: $0 <command>

Commands:
  cli                 CLI usage examples
  api                 API server usage examples
  compose             Docker Compose examples
  performance         Performance testing examples
  all                 All examples
  help                Show this help message

Examples:
  $0 cli              Show CLI examples
  $0 api              Show API examples
  $0 all              Show all examples
EOF
        exit 0
    fi

    case "$1" in
        cli)
            cli_examples
            ;;
        api)
            api_examples
            ;;
        compose)
            compose_examples
            ;;
        performance)
            performance_examples
            ;;
        all)
            cli_examples
            echo ""
            api_examples
            echo ""
            compose_examples
            echo ""
            performance_examples
            ;;
        help|--help|-h)
            main
            ;;
        *)
            print_error "Unknown command: $1"
            main
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
