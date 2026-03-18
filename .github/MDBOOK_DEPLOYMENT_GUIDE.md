# mdBook Documentation Deployment Guide

## Document Information
- **Project**: MeCab-Ko
- **Date**: 2026-03-18 (Updated: 2026-03-19)
- **Status**: ✅ DEPLOYED TO PRODUCTION
- **Target URL**: https://hephaex.github.io/mecab-ko/ (LIVE)

---

## Executive Summary

The mdBook documentation infrastructure is **fully configured and ready for deployment**. The GitHub Actions workflow (`.github/workflows/docs.yml`) is already in place and will automatically build and deploy documentation when code is pushed to the main branch.

### Current Status
- ✅ mdBook project configured (`docs/book/book.toml`)
- ✅ GitHub Actions workflow created (`.github/workflows/docs.yml`)
- ✅ Custom styling and scripts ready (`custom.css`, `custom.js`)
- ✅ Complete documentation structure in place
- ✅ Combined landing page with API reference + User Guide

### What You Need to Do
1. Verify GitHub Pages settings (5 minutes)
2. Push a test commit to main branch (automatic deployment)
3. Verify the site loads (5 minutes)
4. (Optional) Configure custom domain `mecab-ko.dev` (24-48 hours)

---

## Current Infrastructure Overview

### mdBook Configuration
```toml
[book]
title = "MeCab-Ko Documentation"
language = "ko"
src = "src"

[output.html]
additional-css = ["src/custom.css"]
additional-js = ["src/custom.js"]
cname = "mecab-ko.dev"
```

**File**: `docs/book/book.toml` (43 lines)

### GitHub Actions Workflow
**File**: `.github/workflows/docs.yml` (352 lines)

**Jobs**:
1. `build-rustdoc` - Compiles Rust API documentation
2. `build-mdbook` - Builds mdBook HTML
3. `combine-docs` - Merges both into single site with landing page
4. `deploy-pages` - Deploys to GitHub Pages
5. `docs-summary` - Reports build status

**Triggers**:
- Push to main, master, develop
- Pull requests on main, master, develop
- Manual workflow_dispatch
- File path changes in `rust/**`, `docs/**`, etc.

### Documentation Structure
```
docs/book/src/
├── SUMMARY.md                    # Table of Contents
├── introduction.md
├── installation.md
├── quick-start.md
├── cli-usage.md
├── user-dictionary.md
├── output-formats.md
├── custom.css                    # Custom styling
├── custom.js                     # Enhanced functionality
├── api-reference/                # API docs
│   ├── rust.md
│   ├── python.md
│   ├── nodejs.md
│   └── wasm.md
├── tutorials/                    # Tutorial section
├── advanced/                     # Advanced topics
├── reference/                    # Reference documentation
├── developer/                    # Developer guide
├── appendix/                     # Appendix
├── faq.md
└── changelog.md
```

---

## Step-by-Step Deployment Guide

### Step 1: Verify GitHub Pages Settings (5 minutes)

1. Go to your repository settings:
   https://github.com/hephaex/mecab-ko/settings/pages

2. Check the following:
   - **Build and deployment**
     - Source: **GitHub Actions** (not "Deploy from a branch")
   - **GitHub Pages** section should show:
     - "Your site is live at https://hephaex.github.io/mecab-ko/"

3. If source is not "GitHub Actions", change it:
   - Click the dropdown under "Build and deployment"
   - Select "GitHub Actions"
   - Save

### Step 2: Trigger First Deployment (Automatic)

Push a test commit to the main branch:

```bash
# Make a small change to trigger the workflow
echo "" >> docs/book/README.md

# Commit and push
git add docs/book/README.md
git commit -m "docs: Trigger documentation deployment"
git push origin main
```

The GitHub Actions workflow will automatically:
1. Check out the code
2. Install Rust and mdBook
3. Build Rustdoc
4. Build mdBook
5. Combine both into a landing page
6. Deploy to GitHub Pages

**Expected time**: 5-10 minutes

### Step 3: Verify Deployment (5 minutes)

1. Monitor the workflow execution:
   - Go to your repository: https://github.com/hephaex/mecab-ko/actions
   - Find the "Documentation" workflow
   - Wait for all jobs to complete (green checkmarks)

2. Check the deployed site:
   - Visit: https://hephaex.github.io/mecab-ko/
   - You should see the landing page with two cards:
     - "API Reference" (Rustdoc)
     - "User Guide" (mdBook)

3. Test functionality:
   - Click through navigation
   - Test search function (magnifying glass icon)
   - Check mobile responsiveness (resize browser)
   - Verify dark mode toggle (if implemented)

### Step 4 (Optional): Set Up Custom Domain

If you want to use `mecab-ko.dev` instead of `hephaex.github.io/mecab-ko/`:

#### DNS Configuration

**For most registrars (GoDaddy, Namecheap, Cloudflare, etc.)**:

1. Log in to your domain registrar
2. Find DNS settings for `mecab-ko.dev`
3. Add or modify the CNAME record:
   - **Name**: `mecab-ko` (or just the subdomain part)
   - **Type**: CNAME
   - **Value**: `hephaex.github.io.` (note the trailing dot)
   - **TTL**: 3600 (or default)
4. Save changes

**Verify DNS propagation**:
```bash
# Check if DNS is set correctly
dig mecab-ko.dev +short
# Should return: hephaex.github.io.

# Or use online tools:
# https://mxtoolbox.com/
# https://dnschecker.org/
```

#### GitHub Pages Configuration

1. Go to Settings > Pages
2. Under "Custom domain":
   - Enter: `mecab-ko.dev`
   - Click "Save"
3. GitHub will verify the domain
4. Check "Enforce HTTPS" (may take 5-10 minutes)

**Wait time**: DNS propagation can take 24-48 hours

---

## Build Workflow Details

### What Happens on Each Push

```
┌─ Push to main branch
│
├─ GitHub Actions triggered
│
├─ Parallel jobs:
│  ├─ build-rustdoc
│  │  └─ cargo doc --manifest-path rust/Cargo.toml
│  │
│  └─ build-mdbook
│     └─ mdbook build docs/book --dest-dir ../../site/book
│
├─ combine-docs
│  ├─ Merge rustdoc and mdbook artifacts
│  ├─ Generate sitemap.xml
│  ├─ Generate robots.txt
│  └─ Create landing page (index.html)
│
└─ deploy-pages
   └─ GitHub Pages deployment
      └─ Live at https://hephaex.github.io/mecab-ko/
```

### Permissions

The workflow uses these permissions:
```yaml
permissions:
  contents: read     # Read repository code
  pages: write       # Deploy to GitHub Pages
  id_token: write    # Create OIDC token for authentication
```

No secrets or API keys needed - GitHub provides automatic authentication.

---

## File Locations Reference

| Component | Location |
|-----------|----------|
| Workflow file | `.github/workflows/docs.yml` |
| mdBook config | `docs/book/book.toml` |
| Documentation source | `docs/book/src/` |
| Table of contents | `docs/book/src/SUMMARY.md` |
| Custom CSS | `docs/book/src/custom.css` |
| Custom JS | `docs/book/src/custom.js` |
| Build output (local) | `docs/book/book/` |
| Deployed URL | https://hephaex.github.io/mecab-ko/ |

---

## Local Testing

Before pushing to main, test documentation locally:

```bash
# Install mdBook (if not already installed)
cargo install mdbook --version 0.4.48

# Build documentation
mdbook build docs/book

# Serve with live reload
mdbook serve docs/book

# Open in browser (automatic with --open)
mdbook serve docs/book --open
```

The site will be available at: http://localhost:3000

Changes to Markdown files will trigger automatic rebuild and browser refresh.

---

## Customization Options

### Change Theme
Edit `docs/book/book.toml`:
```toml
[output.html]
default-theme = "light"  # or "ayu", "coal", "navy"
preferred-dark-theme = "navy"
```

### Add Custom CSS
Edit `docs/book/src/custom.css` (already has extensive styling)

### Add Custom JavaScript
Edit `docs/book/src/custom.js` (already has enhancements)

### Update Site Title
Edit `docs/book/book.toml`:
```toml
[book]
title = "Your New Title"
```

### Update Git Edit URL
Edit `docs/book/book.toml`:
```toml
[output.html]
edit-url-template = "https://github.com/hephaex/mecab-ko/edit/main/docs/book/{path}"
```

---

## Multilingual Support

### Current State
- Primary language: Korean
- English support: Can be added

### To Add English Documentation

**Option 1: Content Duplication (Recommended for now)**

1. Create separate mdBook project:
   ```bash
   mkdir -p docs/book-en
   cp -r docs/book/* docs/book-en/
   ```

2. Translate Markdown files in `docs/book-en/src/`

3. Update `.github/workflows/docs.yml`:
   ```yaml
   build-mdbook-en:
     name: Build mdBook (English)
     # ... similar to build-mdbook job
     # Output to: site/book-en/
   ```

4. Update landing page to include language selector

5. Deploy both versions:
   - Korean: `/book/`
   - English: `/book-en/`

**Option 2: Native mdBook i18n (Advanced)**

Requires upgrading mdBook to 0.5+ and using native multilingual support. More complex but cleaner architecture.

---

## Troubleshooting

### Workflow Fails: "mdbook command not found"
**Solution**: The workflow installs mdBook. Check that Rust toolchain is installed first.

```yaml
- name: Install Rust toolchain
  uses: dtolnay/rust-toolchain@stable

- name: Install mdBook
  run: cargo install mdbook --version 0.4.48
```

### Documentation Not Deployed
**Check**:
1. GitHub Pages enabled: Settings > Pages > Source = "GitHub Actions"
2. Workflow executed: Go to Actions tab, check for "Documentation" workflow
3. Workflow logs: Click workflow run to see detailed logs
4. Permissions: Ensure `pages: write` permission

### Custom Domain Not Working
**Check**:
1. DNS records are correct: `dig mecab-ko.dev +short`
2. DNS has propagated (can take 24-48 hours)
3. Custom domain entered in GitHub Pages settings
4. HTTPS enforced (after DNS propagation)

### CSS/JavaScript Not Loading
**Check**:
1. Files exist: `docs/book/src/custom.css`, `docs/book/src/custom.js`
2. Files referenced in `book.toml`:
   ```toml
   additional-css = ["src/custom.css"]
   additional-js = ["src/custom.js"]
   ```
3. Clear browser cache (Ctrl+Shift+Delete or Cmd+Shift+Delete)
4. Check browser console for 404 errors

### Search Not Working
**Check**:
1. Search enabled in `book.toml`:
   ```toml
   [output.html.search]
   enable = true
   ```
2. Rebuild documentation: `mdbook build docs/book`
3. Clear browser cache and try again
4. Check that `searchindex.json` exists in built files

---

## Monitoring & Maintenance

### Weekly Tasks
- Monitor workflow success rate
- Check for any deployment errors in Actions tab

### Monthly Tasks
- Check for mdBook updates
- Run link checker to verify broken links
- Review documentation structure
- Check page performance with Lighthouse

### Quarterly Tasks
- Review documentation coverage
- Compare with codebase for gaps
- Update user guides based on feedback
- Test across different browsers/devices

### Before Each Release
- Update `docs/book/src/changelog.md`
- Update version numbers in documentation
- Test all code examples
- Review for broken links

---

## Advanced Features

### Search Configuration
Already enabled with 30-word teasers and hierarchical index.

### Code Playground
Disabled but can be enabled:
```toml
[output.html.playground]
editable = true
copyable = true
line-numbers = true
```

### Content Folding
Already enabled with level 1 (top-level sections fold by default).

### Git Repository Links
Already configured:
```toml
git-repository-url = "https://github.com/hephaex/mecab-ko"
edit-url-template = "https://github.com/hephaex/mecab-ko/edit/main/docs/book/{path}"
```

---

## Performance Optimization

The current setup is optimized for:

✅ **Build speed**: ~1-2 minutes for mdBook, ~3-5 minutes for Rustdoc
✅ **Load time**: Static HTML files, <2 seconds typical
✅ **SEO**: Includes sitemap.xml and robots.txt
✅ **Accessibility**: Semantic HTML, keyboard navigation
✅ **Mobile**: Responsive design with custom CSS

### Future Optimizations
- Add link checker to CI/CD
- Implement PR preview deployments
- Add analytics tracking
- Optimize images/assets
- Implement service worker for offline access

---

## Security Considerations

✅ **HTTPS**: Enforced by GitHub Pages
✅ **Static content**: No dynamic code execution
✅ **No secrets**: No credentials in workflow
✅ **Access control**: Governed by repository permissions
✅ **Content immutability**: Can be enforced with branch protection

---

## Success Metrics

| Metric | Target | Method |
|--------|--------|--------|
| Workflow success rate | >95% | Monitor Actions tab |
| Uptime | 99.9% | GitHub Pages SLA |
| Page load time | <2 seconds | Lighthouse |
| Search performance | <500ms | Manual testing |
| Documentation freshness | Updated per release | Changelog tracking |

---

## Quick Reference Commands

```bash
# Build locally
mdbook build docs/book

# Preview locally
mdbook serve docs/book --open

# Check mdBook installation
mdbook --version

# Clean build
mdbook clean && mdbook build docs/book

# Deploy (automatic on push)
git push origin main

# Check workflow status
# https://github.com/hephaex/mecab-ko/actions

# View deployed site
# https://hephaex.github.io/mecab-ko/
```

---

## Additional Resources

- **mdBook Documentation**: https://rust-lang.github.io/mdBook/
- **GitHub Pages Help**: https://docs.github.com/en/pages
- **GitHub Actions Documentation**: https://docs.github.com/en/actions
- **Markdown Guide**: https://www.markdownguide.org/
- **MeCab-Ko Repository**: https://github.com/hephaex/mecab-ko

---

## Next Steps

1. ✅ Verify GitHub Pages settings (5 min)
2. ✅ Push test commit (automatic)
3. ✅ Verify deployment (5 min)
4. 🔄 (Optional) Set up custom domain (24-48 hours)
5. 📚 Start adding/updating documentation
6. 🚀 Monitor and maintain

---

**Status**: Ready for Production Deployment
**Last Updated**: 2026-03-18
