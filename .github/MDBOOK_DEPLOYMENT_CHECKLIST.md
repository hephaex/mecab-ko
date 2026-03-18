# mdBook Documentation Deployment Checklist

## Pre-Deployment Verification (5 minutes)

- [ ] **Verify Repository Access**
  - [ ] You have push access to https://github.com/hephaex/mecab-ko
  - [ ] You can access repository settings

- [ ] **Verify GitHub Pages Settings**
  - [ ] Go to: https://github.com/hephaex/mecab-ko/settings/pages
  - [ ] Check "Build and deployment" > "Source"
  - [ ] Confirm source is set to "GitHub Actions" (not "Deploy from a branch")
  - [ ] If not, change to "GitHub Actions" and save

- [ ] **Verify Workflow File Exists**
  - [ ] `.github/workflows/docs.yml` exists
  - [ ] File contains `name: Documentation`
  - [ ] Jobs include: build-rustdoc, build-mdbook, combine-docs, deploy-pages

- [ ] **Verify mdBook Configuration**
  - [ ] `docs/book/book.toml` exists
  - [ ] Contains `[book]` section with title and language
  - [ ] Contains `[output.html]` with custom CSS/JS
  - [ ] `docs/book/src/SUMMARY.md` exists (table of contents)

- [ ] **Verify Documentation Source Files**
  - [ ] `docs/book/src/introduction.md` exists
  - [ ] `docs/book/src/installation.md` exists
  - [ ] `docs/book/src/quick-start.md` exists
  - [ ] `docs/book/src/custom.css` exists
  - [ ] `docs/book/src/custom.js` exists

- [ ] **Verify Rust Project Builds**
  - [ ] Run: `cargo build --release`
  - [ ] No compilation errors
  - [ ] All dependencies resolved

---

## First Deployment Execution (Automatic)

### Step 1: Trigger Workflow

- [ ] Make a small test change:
  ```bash
  echo "" >> docs/book/README.md
  ```

- [ ] Stage the change:
  ```bash
  git add docs/book/README.md
  ```

- [ ] Commit with message:
  ```bash
  git commit -m "docs: Trigger documentation deployment"
  ```

- [ ] Push to main:
  ```bash
  git push origin main
  ```

### Step 2: Monitor Workflow Execution

- [ ] Go to: https://github.com/hephaex/mecab-ko/actions
- [ ] Find the "Documentation" workflow
- [ ] Monitor status as workflow progresses:
  - [ ] build-rustdoc job starts
  - [ ] build-mdbook job starts (parallel)
  - [ ] Both build jobs complete successfully
  - [ ] combine-docs job starts
  - [ ] combine-docs completes successfully
  - [ ] deploy-pages job starts
  - [ ] deploy-pages completes successfully
  - [ ] docs-summary job reports build status

- [ ] **Expected duration**: 5-10 minutes total
  - Rustdoc build: 3-5 minutes
  - mdBook build: 1-2 minutes
  - Combine & deploy: 1-2 minutes

- [ ] **Check for errors**:
  - [ ] All job badges are green (✓)
  - [ ] No red X marks (×)
  - [ ] No yellow warnings in job logs

### Step 3: Verify Workflow Logs

- [ ] Click on "Documentation" workflow run
- [ ] For each job, check the logs:

**build-rustdoc job**:
- [ ] Rust toolchain installed
- [ ] Cargo doc executed
- [ ] All crates documented
- [ ] Artifact uploaded

**build-mdbook job**:
- [ ] mdBook version 0.4.48 installed
- [ ] book.toml found
- [ ] All markdown files parsed
- [ ] HTML generated
- [ ] Sitemap.xml created
- [ ] robots.txt created
- [ ] Artifact uploaded

**combine-docs job**:
- [ ] Artifacts downloaded
- [ ] Landing page (index.html) generated
- [ ] Directory structure created:
  - [ ] docs-combined/index.html
  - [ ] docs-combined/api/ (rustdoc)
  - [ ] docs-combined/book/ (mdbook)
- [ ] Artifact uploaded

**deploy-pages job**:
- [ ] Pages environment setup
- [ ] Artifact uploaded to pages
- [ ] Deployment successful
- [ ] Deployment URL output (should be https://hephaex.github.io/mecab-ko/)

---

## Post-Deployment Verification (5 minutes)

### Step 1: Verify Site Loads

- [ ] **Main landing page**:
  - [ ] Visit: https://hephaex.github.io/mecab-ko/
  - [ ] Page loads without errors
  - [ ] See "MeCab-Ko Documentation" title
  - [ ] See gradient background
  - [ ] See two documentation cards:
    - [ ] "API Reference" card
    - [ ] "User Guide" card

- [ ] **Page styling**:
  - [ ] Custom CSS applied (gradient colors visible)
  - [ ] Responsive layout (not broken)
  - [ ] Fonts render correctly
  - [ ] Images (if any) display properly

### Step 2: Test API Reference

- [ ] Click "API Reference" card
- [ ] **Rustdoc page loads**:
  - [ ] See `mecab_ko` crate documentation
  - [ ] Navigation sidebar visible
  - [ ] Code examples render properly
  - [ ] Search works
  - [ ] No 404 errors in console

- [ ] **Test Rustdoc functionality**:
  - [ ] Click on module names
  - [ ] View function/struct documentation
  - [ ] Check code highlighting
  - [ ] Test search (search for "Tagger")

### Step 3: Test User Guide

- [ ] Click "User Guide" card
- [ ] **mdBook page loads**:
  - [ ] See table of contents on left
  - [ ] Main content area shows introduction
  - [ ] Navigation buttons visible
  - [ ] Search button visible
  - [ ] No 404 errors

- [ ] **Test mdBook functionality**:
  - [ ] Click through chapters in sidebar
  - [ ] Navigation buttons (Previous/Next) work
  - [ ] Search function works (search for "설치")
  - [ ] Code blocks display with syntax highlighting
  - [ ] Tables render properly
  - [ ] Links work (both internal and external)

### Step 4: Test Navigation

- [ ] **Inter-page navigation**:
  - [ ] Can navigate from landing page to API
  - [ ] Can navigate from landing page to User Guide
  - [ ] Can navigate back to landing page
  - [ ] All links work without 404 errors

- [ ] **Search functionality**:
  - [ ] mdBook search works (search "quick")
  - [ ] Results appear correctly
  - [ ] Click on result navigates to page

### Step 5: Responsive Design Testing

- [ ] **Desktop view** (1920x1080 or similar):
  - [ ] Layout is clean
  - [ ] Two-column layout for mdBook
  - [ ] No horizontal scrolling needed

- [ ] **Tablet view** (768x1024):
  - [ ] Layout adapts
  - [ ] Sidebar collapses to menu
  - [ ] Content is readable
  - [ ] Touch-friendly buttons

- [ ] **Mobile view** (375x667):
  - [ ] Layout is single column
  - [ ] Sidebar hidden behind menu
  - [ ] Content is readable (no tiny text)
  - [ ] Cards stack vertically
  - [ ] All functionality works

**Tools to use**:
- Chrome DevTools: F12 > Toggle device toolbar
- Firefox DevTools: F12 > Responsive Design Mode
- Safari: Develop > Enter Responsive Design Mode

### Step 6: Browser Compatibility

- [ ] **Google Chrome** (latest):
  - [ ] Site loads
  - [ ] All features work
  - [ ] No console errors

- [ ] **Mozilla Firefox** (latest):
  - [ ] Site loads
  - [ ] All features work
  - [ ] No console errors

- [ ] **Safari** (if on macOS):
  - [ ] Site loads
  - [ ] All features work
  - [ ] No console errors

- [ ] **Edge** (if on Windows):
  - [ ] Site loads
  - [ ] All features work
  - [ ] No console errors

### Step 7: Check Browser Console

- [ ] Open browser developer tools (F12)
- [ ] Go to Console tab
- [ ] Verify no errors (red messages)
- [ ] Verify no warnings (yellow messages)
- [ ] If errors exist:
  - [ ] Note the error message
  - [ ] Check if it's critical (breaks functionality)
  - [ ] File issue if critical

### Step 8: Verify Metadata

- [ ] Open page source (Ctrl+U or Cmd+U)
- [ ] Check meta tags:
  - [ ] `<meta charset="utf-8">`
  - [ ] `<meta name="viewport" content="width=device-width">`
  - [ ] `<meta name="description">`
  - [ ] `<meta name="keywords">`
  - [ ] `<link rel="canonical">`

- [ ] Check Open Graph tags:
  - [ ] `og:type`
  - [ ] `og:url`
  - [ ] `og:title`
  - [ ] `og:description`

---

## Optional: Custom Domain Setup (24-48 hours)

### Step 1: DNS Configuration

**Choose your registrar and follow the appropriate steps**:

#### GoDaddy
- [ ] Log in to GoDaddy account
- [ ] Go to "Domains" > "My Domains"
- [ ] Find "mecab-ko.dev"
- [ ] Click "Manage DNS"
- [ ] Find CNAME records section
- [ ] Add or edit CNAME record:
  - [ ] Name: `mecab-ko`
  - [ ] Value: `hephaex.github.io.` (note: trailing dot)
  - [ ] TTL: 3600
- [ ] Save changes
- [ ] Note: Changes take effect in 24-48 hours

#### Namecheap
- [ ] Log in to Namecheap
- [ ] Go to "My Account" > "Domain List"
- [ ] Find "mecab-ko.dev" and click "Manage"
- [ ] Go to "Advanced DNS" tab
- [ ] Add new CNAME record:
  - [ ] Type: CNAME
  - [ ] Host: `mecab-ko`
  - [ ] Value: `hephaex.github.io.`
  - [ ] TTL: 3600
- [ ] Save changes

#### Cloudflare
- [ ] Log in to Cloudflare
- [ ] Select "mecab-ko.dev" domain
- [ ] Go to "DNS" > "Records"
- [ ] Add new record:
  - [ ] Type: CNAME
  - [ ] Name: `mecab-ko`
  - [ ] Content: `hephaex.github.io.`
  - [ ] TTL: Auto
  - [ ] Proxied: As preferred
- [ ] Save

#### Other Registrars
- [ ] Log in to your registrar
- [ ] Find DNS management for mecab-ko.dev
- [ ] Add CNAME record:
  - [ ] Hostname: `mecab-ko`
  - [ ] Target: `hephaex.github.io.`
  - [ ] TTL: 3600 or default
- [ ] Save

### Step 2: Verify DNS Configuration

- [ ] **Wait for DNS propagation** (up to 48 hours)
- [ ] **Check DNS records**:
  ```bash
  # Check CNAME record
  dig mecab-ko.dev CNAME
  # Should show: mecab-ko.dev. IN CNAME hephaex.github.io.

  # Or use shorter command
  dig mecab-ko.dev +short
  # Should show: hephaex.github.io.
  ```

- [ ] **Use online DNS checker**:
  - [ ] Visit: https://dnschecker.org/
  - [ ] Enter: `mecab-ko.dev`
  - [ ] Check "CNAME" record type
  - [ ] Verify all nameservers return same result

- [ ] **Test domain accessibility**:
  ```bash
  # Try to resolve domain
  nslookup mecab-ko.dev
  # Should resolve to GitHub Pages IP
  ```

### Step 3: GitHub Pages Configuration

- [ ] Go to: https://github.com/hephaex/mecab-ko/settings/pages
- [ ] Under "Custom domain" section:
  - [ ] Enter: `mecab-ko.dev`
  - [ ] Click "Save"
- [ ] GitHub will verify the domain
  - [ ] **Success**: Green checkmark, "DNS is working"
  - [ ] **Waiting**: Yellow status, "Checking DNS configuration"
  - [ ] **Failed**: Red error, check DNS records

### Step 4: Enable HTTPS

- [ ] After DNS verification:
  - [ ] Check box: "Enforce HTTPS"
  - [ ] GitHub will provision SSL certificate
  - [ ] **Note**: Takes 5-10 minutes for certificate issuance

- [ ] If HTTPS not appearing:
  - [ ] Wait another 5-10 minutes
  - [ ] Remove and re-add custom domain
  - [ ] Check GitHub's status page

### Step 5: Verify Custom Domain Works

- [ ] Visit: https://mecab-ko.dev
- [ ] Should redirect to: https://hephaex.github.io/mecab-ko/
- [ ] OR directly serve content at: https://mecab-ko.dev

- [ ] Test subsections:
  - [ ] https://mecab-ko.dev/book/ (User Guide)
  - [ ] https://mecab-ko.dev/api/ (API Reference)

- [ ] Verify HTTPS:
  - [ ] Green lock icon in browser
  - [ ] URL starts with `https://`
  - [ ] Certificate is valid (check browser security info)

---

## Ongoing Monitoring

### Daily (After each deployment)
- [ ] Workflow completes without errors
- [ ] Site loads at target URL
- [ ] No obvious visual issues

### Weekly
- [ ] Check workflow success rate
- [ ] Verify site is accessible
- [ ] Test search functionality
- [ ] Check for any broken links (manual spot check)

### Monthly
- [ ] Check for mdBook updates:
  ```bash
  cargo search mdbook
  ```
- [ ] Run link checker (if implemented)
- [ ] Review documentation for outdated content
- [ ] Check page performance with Lighthouse

### Quarterly
- [ ] Comprehensive documentation review
- [ ] Compare documentation with codebase
- [ ] Update any outdated sections
- [ ] Gather user feedback on documentation
- [ ] Plan documentation improvements

---

## Troubleshooting Checklist

### If Workflow Fails

**Scenario: build-mdbook job fails**
- [ ] Check job log for error message
- [ ] Common causes:
  - [ ] `book.toml` syntax error
  - [ ] Missing markdown file referenced in SUMMARY.md
  - [ ] Invalid YAML in workflow
- [ ] Fix in local branch, test, commit, push again

**Scenario: deploy-pages job fails**
- [ ] Check GitHub Pages settings
- [ ] Verify source is "GitHub Actions"
- [ ] Check that deploy-pages action has proper permissions
- [ ] Try removing and re-enabling GitHub Pages

**Scenario: Workflow times out**
- [ ] Check if Rustdoc build is slow
- [ ] Consider using build cache
- [ ] Check cargo dependencies for large builds

### If Site Doesn't Load

**Scenario: 404 error at custom domain**
- [ ] Verify DNS records are correct: `dig mecab-ko.dev`
- [ ] Wait for DNS propagation (up to 48 hours)
- [ ] Re-enter custom domain in GitHub Pages settings
- [ ] Check GitHub Pages status: https://www.githubstatus.com/

**Scenario: Site shows old content**
- [ ] Hard refresh browser: Ctrl+Shift+R (Windows/Linux) or Cmd+Shift+R (Mac)
- [ ] Clear browser cache
- [ ] Try incognito/private mode
- [ ] Wait 5 minutes for CDN cache to update

**Scenario: CSS/JS not loading**
- [ ] Check file paths in book.toml
- [ ] Verify files exist in docs/book/src/
- [ ] Clear browser cache
- [ ] Check browser console for 404 errors

### If Search Doesn't Work

- [ ] Verify search enabled in book.toml
- [ ] Check that searchindex.json exists in built output
- [ ] Clear browser cache
- [ ] Try different search terms
- [ ] Check browser console for JavaScript errors

### If Custom Domain Not Working

- [ ] Verify DNS record: `dig mecab-ko.dev`
- [ ] Check DNS propagation: https://dnschecker.org/
- [ ] Verify domain entered correctly in GitHub Pages settings
- [ ] Wait for HTTPS certificate (5-10 minutes after DNS verification)
- [ ] Remove and re-add custom domain if stuck

---

## Success Criteria

### Minimum (Basic Functionality)
- [x] GitHub Pages enabled with "GitHub Actions" source
- [x] Workflow file exists and is valid
- [x] Site loads at https://hephaex.github.io/mecab-ko/
- [x] Landing page displays with two sections
- [x] Can navigate to mdBook section
- [x] Can navigate to Rustdoc section

### Expected (Full Functionality)
- [x] All above criteria met
- [x] mdBook search works
- [x] All documentation pages load
- [x] Navigation between pages works
- [x] Mobile responsive design works
- [x] No console errors

### Enhanced (Optional)
- [x] Custom domain https://mecab-ko.dev/ configured
- [x] HTTPS enforced on custom domain
- [x] Sitemap.xml generated
- [x] robots.txt configured
- [x] Analytics tracking enabled
- [x] Link checker integrated

---

## Final Verification Sign-Off

Once all items are complete, you can confirm:

- [ ] **Pre-deployment checks**: All verified
- [ ] **Workflow execution**: Successful
- [ ] **Post-deployment verification**: All tests passed
- [ ] **Documentation content**: Reviewed and accurate
- [ ] **Custom domain** (if applicable): Configured and working
- [ ] **Performance**: Acceptable load times
- [ ] **Functionality**: All features working
- [ ] **Accessibility**: Site is usable

**Status**: ✅ READY FOR PRODUCTION

---

## Quick Reference

| Task | Time | Status |
|------|------|--------|
| Verify GitHub Pages settings | 5 min | ○ |
| Trigger first deployment | Auto | ○ |
| Monitor workflow | 5-10 min | ○ |
| Verify site loads | 5 min | ○ |
| Test all functionality | 10 min | ○ |
| (Optional) Setup custom domain | 24-48 hr | ○ |
| **Total active time** | **25-30 min** | |
| **Total wall-clock time** | **24-48 hr** (with custom domain) | |

---

**Last Updated**: 2026-03-18
**Document Version**: 1.0
