# MeCab-Ko Documentation

This directory contains the source for the MeCab-Ko documentation website built with [mdBook](https://rust-lang.github.io/mdBook/).

## Building the Documentation

### Prerequisites

- Rust 1.70.0 or later
- mdBook 0.4.40

### Install mdBook

```bash
cargo install mdbook --version 0.4.40
```

### Build

```bash
# Build the book
mdbook build

# Serve locally with live reload
mdbook serve

# Open in browser
mdbook serve --open
```

The built documentation will be in `book/` directory.

## Structure

```
src/
├── SUMMARY.md              # Table of contents
├── introduction.md         # Introduction
├── installation.md         # Installation guide
├── quick-start.md          # Quick start guide
├── cli-usage.md            # CLI usage
├── user-dictionary.md      # User dictionary guide
├── output-formats.md       # Output formats
├── api-reference/          # API documentation
│   ├── rust.md            # Rust API
│   ├── python.md          # Python bindings
│   ├── nodejs.md          # Node.js bindings
│   └── wasm.md            # WASM bindings
├── advanced/              # Advanced topics
│   ├── dictionary-builder.md
│   ├── performance-tuning.md
│   ├── elasticsearch.md
│   └── custom-analyzer.md
├── reference/             # Reference documentation
│   ├── pos-tags.md
│   ├── dictionary-format.md
│   └── binary-format.md
├── developer/             # Developer guide
│   ├── project-structure.md
│   ├── build-process.md
│   └── contributing.md
├── appendix/              # Appendix
│   ├── migration.md
│   └── benchmarks.md
├── faq.md                 # FAQ
├── changelog.md           # Changelog
├── custom.css             # Custom styles
└── custom.js              # Custom JavaScript
```

## Configuration

See `book.toml` for mdBook configuration:

- Theme customization
- Search settings
- Output options
- Preprocessors

## Contributing

To contribute to the documentation:

1. Edit the Markdown files in `src/`
2. Test locally with `mdbook serve`
3. Submit a pull request

### Writing Guidelines

- Use clear, concise language
- Include code examples
- Add cross-references between pages
- Follow the existing structure
- Test all code examples

### Code Blocks

Use language-specific syntax highlighting:

````markdown
```rust
let tagger = Tagger::new(config)?;
```

```python
tagger = Tagger()
```

```javascript
const tagger = new Tagger();
```
````

### Links

- Internal links: `[Text](./page.md)`
- Sections: `[Text](./page.md#section)`
- External: `[Text](https://example.com)`

## Deployment

The documentation is automatically built and deployed to GitHub Pages via GitHub Actions (`.github/workflows/docs.yml`).

### Manual Deployment

```bash
# Build
mdbook build

# Deploy to gh-pages branch
git worktree add gh-pages gh-pages
cp -r book/* gh-pages/
cd gh-pages
git add .
git commit -m "Update documentation"
git push origin gh-pages
```

## Features

- **Search**: Full-text search with lunr.js
- **Syntax Highlighting**: Code highlighting for multiple languages
- **Responsive**: Mobile-friendly design
- **Print**: Optimized for printing
- **Dark Mode**: Built-in theme support
- **Copy Buttons**: One-click code copying
- **Back to Top**: Smooth scrolling navigation

## Custom Enhancements

### CSS (`custom.css`)

- Gradient headers
- Enhanced tables
- Code block styling
- Responsive design
- Dark theme support

### JavaScript (`custom.js`)

- Copy buttons for code blocks
- Language labels
- Anchor links
- Smooth scrolling
- Back to top button
- External link indicators
- Search enhancements

## Troubleshooting

### mdBook not found

```bash
cargo install mdbook
```

### Build errors

```bash
# Clean and rebuild
mdbook clean
mdbook build
```

### Live reload not working

```bash
# Try different port
mdbook serve --port 3001
```

## Resources

- [mdBook Documentation](https://rust-lang.github.io/mdBook/)
- [Markdown Guide](https://www.markdownguide.org/)
- [MeCab-Ko Repository](https://github.com/hephaex/mecab-ko)

## License

This documentation is licensed under Apache 2.0 or MIT license.
