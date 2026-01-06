# Build and Test Guide

Complete guide for building, testing, and packaging the MeCab-Ko Elasticsearch plugin.

## Prerequisites

### System Requirements
- **OS**: Linux, macOS, or Windows
- **Java**: JDK 17 or higher
- **Rust**: 1.70 or higher (with cargo)
- **Elasticsearch**: 8.11.3 or higher (for testing)

### Install Prerequisites

#### Ubuntu/Debian
```bash
# Java 17
sudo apt update
sudo apt install openjdk-17-jdk

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Elasticsearch (optional, for testing)
wget -qO - https://artifacts.elastic.co/GPG-KEY-elasticsearch | sudo apt-key add -
sudo apt install apt-transport-https
echo "deb https://artifacts.elastic.co/packages/8.x/apt stable main" | sudo tee /etc/apt/sources.list.d/elastic-8.x.list
sudo apt update && sudo apt install elasticsearch
```

#### macOS
```bash
# Java 17
brew install openjdk@17

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Elasticsearch (optional)
brew tap elastic/tap
brew install elastic/tap/elasticsearch-full
```

#### Windows
```powershell
# Install via chocolatey
choco install openjdk17
choco install rust

# Or download installers:
# Java: https://adoptium.net/
# Rust: https://rustup.rs/
# Elasticsearch: https://www.elastic.co/downloads/elasticsearch
```

## Build Process

### Step 1: Clone Repository

```bash
git clone https://github.com/mecab-ko/mecab-ko.git
cd mecab-ko
```

### Step 2: Build Native Library

```bash
cd rust

# Build in release mode with JNI bindings
cargo build --release --features jni-bindings

# Verify build
ls -la target/release/libmecab_ko_elasticsearch.*

# Expected output (platform-dependent):
# Linux:   libmecab_ko_elasticsearch.so
# macOS:   libmecab_ko_elasticsearch.dylib
# Windows: mecab_ko_elasticsearch.dll
```

### Step 3: Build Plugin

```bash
cd ../elasticsearch-plugin

# Build plugin package
./gradlew clean build bundlePlugin

# Verify build
ls -la build/distributions/

# Expected output:
# mecab-ko-analyzer-0.1.0.zip
```

### Step 4: Verify Package Contents

```bash
# Extract and inspect
unzip -l build/distributions/mecab-ko-analyzer-0.1.0.zip

# Should contain:
# - plugin-descriptor.properties
# - plugin-security.policy
# - JAR files
# - native/ directory with library
# - lib/ directory with dependencies
```

## Build Variants

### Debug Build (for development)

```bash
# Rust library with debug symbols
cd rust
cargo build --features jni-bindings

# Plugin with debug info
cd ../elasticsearch-plugin
./gradlew build -Pdebug=true
```

### Release Build (for production)

```bash
# Optimized Rust library
cd rust
cargo build --release --features jni-bindings

# Optimized plugin
cd ../elasticsearch-plugin
./gradlew clean build bundlePlugin
```

### Platform-Specific Builds

#### Cross-Compile for Linux (from macOS)

```bash
# Install cross-compilation tools
rustup target add x86_64-unknown-linux-gnu
brew install SergioBenitez/osxct/x86_64-unknown-linux-gnu

# Build
cd rust
cargo build --release --target x86_64-unknown-linux-gnu --features jni-bindings
```

#### Build for Multiple Platforms

```bash
# Build script for all platforms
#!/bin/bash
for target in x86_64-unknown-linux-gnu x86_64-apple-darwin x86_64-pc-windows-gnu; do
    cargo build --release --target $target --features jni-bindings
done
```

## Testing

### Unit Tests

```bash
cd elasticsearch-plugin

# Run unit tests
./gradlew test

# Run with coverage
./gradlew test jacocoTestReport

# View results
open build/reports/tests/test/index.html
```

### Integration Tests

```bash
# Start Elasticsearch (required)
elasticsearch &

# Wait for cluster ready
curl -X GET "localhost:9200/_cluster/health?wait_for_status=yellow&timeout=30s"

# Run integration tests
./gradlew integrationTest

# View results
open build/reports/tests/integrationTest/index.html
```

### Manual Testing

```bash
# Install plugin
./install.sh /path/to/elasticsearch

# Restart Elasticsearch
pkill -f elasticsearch
elasticsearch &

# Wait for startup
until curl -s "http://localhost:9200" > /dev/null; do
    sleep 1
done

# Test analyzer
curl -X POST "localhost:9200/_analyze?pretty" \
  -H 'Content-Type: application/json' -d'
{
  "analyzer": "mecab_ko",
  "text": "한국어 형태소 분석기 테스트"
}'

# Verify output
# Should show tokenized Korean text
```

### Performance Testing

```bash
# Create test index
curl -X PUT "localhost:9200/perf_test" -H 'Content-Type: application/json' -d'
{
  "settings": {
    "number_of_shards": 1,
    "analysis": {
      "analyzer": {
        "test": {
          "type": "mecab_ko",
          "decompound_mode": "mixed"
        }
      }
    }
  }
}'

# Benchmark indexing
for i in {1..10000}; do
  curl -X POST "localhost:9200/perf_test/_doc" \
    -H 'Content-Type: application/json' -d"{\"text\": \"한국어 문서 $i\"}" &
done
wait

# Check indexing stats
curl -X GET "localhost:9200/perf_test/_stats?pretty"
```

## Gradle Tasks

### Essential Tasks

```bash
# Clean build artifacts
./gradlew clean

# Compile Java sources
./gradlew compileJava

# Run tests
./gradlew test

# Build JAR
./gradlew jar

# Create plugin package
./gradlew bundlePlugin

# Run all checks (test + lint)
./gradlew check

# Build everything
./gradlew build
```

### Advanced Tasks

```bash
# List all tasks
./gradlew tasks

# Build with verbose output
./gradlew build --info

# Build offline (no network)
./gradlew build --offline

# Continuous build (auto-rebuild on changes)
./gradlew build --continuous

# Build with specific Java version
./gradlew build -Dorg.gradle.java.home=/path/to/jdk17
```

## Troubleshooting Builds

### Issue: Native library not found during build

**Solution:**
```bash
# Ensure native library exists
ls -la ../rust/target/release/libmecab_ko_elasticsearch.*

# If missing, rebuild
cd ../rust
cargo clean
cargo build --release --features jni-bindings
```

### Issue: Gradle build fails with "Could not find elasticsearch"

**Solution:**
```bash
# Clear Gradle cache
rm -rf ~/.gradle/caches/

# Rebuild
./gradlew clean build --refresh-dependencies
```

### Issue: Java version mismatch

**Solution:**
```bash
# Check Java version
java -version  # Should be 17+

# Set JAVA_HOME
export JAVA_HOME=/path/to/jdk-17
export PATH=$JAVA_HOME/bin:$PATH

# Verify
java -version
./gradlew --version
```

### Issue: Out of memory during build

**Solution:**
```bash
# Increase Gradle heap
export GRADLE_OPTS="-Xmx4g -XX:MaxMetaspaceSize=512m"

# Or edit gradle.properties
echo "org.gradle.jvmargs=-Xmx4g -XX:MaxMetaspaceSize=512m" >> gradle.properties

# Rebuild
./gradlew clean build
```

### Issue: Tests fail with "Connection refused"

**Solution:**
```bash
# Ensure Elasticsearch is running
curl http://localhost:9200

# Start if not running
elasticsearch &

# Wait for yellow status
curl -X GET "localhost:9200/_cluster/health?wait_for_status=yellow"

# Re-run tests
./gradlew test
```

## Packaging for Distribution

### Create Release Package

```bash
#!/bin/bash
# build-release.sh

set -e

echo "Building MeCab-Ko Elasticsearch Plugin Release..."

# Version
VERSION="0.1.0"

# Build native libraries for all platforms
echo "Building native libraries..."
cd rust

for target in x86_64-unknown-linux-gnu x86_64-apple-darwin; do
    echo "Building for $target..."
    cargo build --release --target $target --features jni-bindings
done

cd ..

# Build plugin
echo "Building plugin..."
cd elasticsearch-plugin
./gradlew clean build bundlePlugin

# Create release directory
RELEASE_DIR="release-$VERSION"
mkdir -p $RELEASE_DIR

# Copy artifacts
cp build/distributions/*.zip $RELEASE_DIR/
cp README.md QUICKSTART.md LICENSE NOTICE $RELEASE_DIR/
cp -r examples $RELEASE_DIR/

# Create checksums
cd $RELEASE_DIR
sha256sum *.zip > SHA256SUMS

echo "Release package created in $RELEASE_DIR"
ls -lh
```

### Verify Release Package

```bash
# Extract
unzip mecab-ko-analyzer-0.1.0.zip -d test-install

# Verify structure
tree test-install/

# Test installation
./install.sh /path/to/elasticsearch

# Verify
curl -X GET "localhost:9200/_cat/plugins?v"
```

## Continuous Integration

### GitHub Actions Example

```yaml
# .github/workflows/build.yml
name: Build and Test

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v3

    - name: Set up JDK 17
      uses: actions/setup-java@v3
      with:
        java-version: '17'
        distribution: 'temurin'

    - name: Set up Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true

    - name: Build native library
      working-directory: rust
      run: cargo build --release --features jni-bindings

    - name: Build plugin
      working-directory: elasticsearch-plugin
      run: ./gradlew build bundlePlugin

    - name: Run tests
      working-directory: elasticsearch-plugin
      run: ./gradlew test

    - name: Upload artifact
      uses: actions/upload-artifact@v3
      with:
        name: mecab-ko-analyzer
        path: elasticsearch-plugin/build/distributions/*.zip
```

## Build Optimization Tips

### Faster Rust Builds

```bash
# Use sccache for caching
cargo install sccache
export RUSTC_WRAPPER=sccache

# Parallel compilation
export CARGO_BUILD_JOBS=8

# Incremental compilation
export CARGO_INCREMENTAL=1
```

### Faster Gradle Builds

```bash
# Enable daemon and parallel builds
echo "org.gradle.daemon=true" >> gradle.properties
echo "org.gradle.parallel=true" >> gradle.properties
echo "org.gradle.caching=true" >> gradle.properties

# Configure build cache
mkdir -p ~/.gradle/caches
```

### Build Time Comparison

| Configuration | Time |
|---------------|------|
| Cold build (no cache) | ~5 min |
| Warm build (with cache) | ~30 sec |
| Incremental (Java only) | ~10 sec |
| Incremental (Rust only) | ~20 sec |

## Version Management

### Updating Version

```bash
# Update in multiple files
VERSION="0.2.0"

# plugin-descriptor.properties
sed -i "s/version=.*/version=$VERSION/" \
  src/main/resources/plugin-descriptor.properties

# build.gradle.kts
sed -i "s/version = .*/version = \"$VERSION\"/" build.gradle.kts

# Rust Cargo.toml (workspace version)
sed -i "s/version = .*/version = \"$VERSION\"/" \
  ../rust/Cargo.toml
```

## Build Artifacts

### Expected Output Files

```
build/
├── classes/                     # Compiled Java classes
├── distributions/
│   └── mecab-ko-analyzer-0.1.0.zip  # Plugin package
├── libs/
│   └── mecab-ko-elasticsearch-plugin-0.1.0.jar  # Plugin JAR
├── reports/
│   ├── tests/                   # Test reports
│   └── jacoco/                  # Coverage reports
└── tmp/                         # Temporary build files
```

## Next Steps

After successful build:

1. **Test Installation**: Use `install.sh` to test on local Elasticsearch
2. **Run Integration Tests**: Verify all features work end-to-end
3. **Performance Testing**: Benchmark against production data
4. **Documentation**: Update README with any changes
5. **Release**: Tag version and create GitHub release

## Support

- Build issues: https://github.com/mecab-ko/mecab-ko/issues
- Discussions: https://github.com/mecab-ko/mecab-ko/discussions
- Email: mecab-ko@googlegroups.com

---

**Last Updated**: 2026-01-06
**Build Version**: 0.1.0
