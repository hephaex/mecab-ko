// MeCab-Ko Documentation Custom JavaScript

(function() {
    'use strict';

    // Add copy button to code blocks
    function addCopyButtons() {
        const codeBlocks = document.querySelectorAll('pre > code');

        codeBlocks.forEach((codeBlock) => {
            const pre = codeBlock.parentElement;
            const button = document.createElement('button');
            button.className = 'copy-button';
            button.textContent = 'Copy';
            button.setAttribute('aria-label', 'Copy code to clipboard');

            button.addEventListener('click', async () => {
                try {
                    await navigator.clipboard.writeText(codeBlock.textContent);
                    button.textContent = 'Copied!';
                    setTimeout(() => {
                        button.textContent = 'Copy';
                    }, 2000);
                } catch (err) {
                    console.error('Failed to copy:', err);
                    button.textContent = 'Error';
                }
            });

            pre.style.position = 'relative';
            pre.appendChild(button);
        });
    }

    // Add language labels to code blocks
    function addLanguageLabels() {
        const codeBlocks = document.querySelectorAll('pre > code[class*="language-"]');

        codeBlocks.forEach((codeBlock) => {
            const className = codeBlock.className;
            const match = className.match(/language-(\w+)/);

            if (match) {
                const lang = match[1];
                const pre = codeBlock.parentElement;
                pre.setAttribute('data-lang', lang);
            }
        });
    }

    // Add anchor links to headings
    function addAnchorLinks() {
        const headings = document.querySelectorAll('h2, h3, h4, h5, h6');

        headings.forEach((heading) => {
            if (heading.id) {
                const anchor = document.createElement('a');
                anchor.className = 'anchor-link';
                anchor.href = '#' + heading.id;
                anchor.innerHTML = '<span aria-hidden="true">#</span>';
                anchor.setAttribute('aria-label', 'Link to this section');
                heading.appendChild(anchor);
            }
        });
    }

    // Smooth scrolling for anchor links
    function smoothScroll() {
        document.querySelectorAll('a[href^="#"]').forEach(anchor => {
            anchor.addEventListener('click', function (e) {
                const href = this.getAttribute('href');
                if (href === '#') return;

                e.preventDefault();
                const target = document.querySelector(href);
                if (target) {
                    target.scrollIntoView({
                        behavior: 'smooth',
                        block: 'start'
                    });
                }
            });
        });
    }

    // Add "Back to top" button
    function addBackToTop() {
        const button = document.createElement('button');
        button.id = 'back-to-top';
        button.innerHTML = '↑';
        button.setAttribute('aria-label', 'Back to top');
        button.style.cssText = `
            position: fixed;
            bottom: 20px;
            right: 20px;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            border: none;
            border-radius: 50%;
            width: 50px;
            height: 50px;
            font-size: 20px;
            cursor: pointer;
            opacity: 0;
            transition: opacity 0.3s ease;
            z-index: 1000;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
        `;

        document.body.appendChild(button);

        window.addEventListener('scroll', () => {
            if (window.pageYOffset > 300) {
                button.style.opacity = '1';
            } else {
                button.style.opacity = '0';
            }
        });

        button.addEventListener('click', () => {
            window.scrollTo({
                top: 0,
                behavior: 'smooth'
            });
        });
    }

    // Enhance tables
    function enhanceTables() {
        const tables = document.querySelectorAll('table');

        tables.forEach((table) => {
            // Add responsive wrapper
            const wrapper = document.createElement('div');
            wrapper.style.overflowX = 'auto';
            table.parentNode.insertBefore(wrapper, table);
            wrapper.appendChild(table);

            // Add sorting capability (optional)
            const headers = table.querySelectorAll('th');
            headers.forEach((header, index) => {
                header.style.cursor = 'pointer';
                header.setAttribute('title', 'Click to sort');
            });
        });
    }

    // Add external link indicators
    function markExternalLinks() {
        const links = document.querySelectorAll('a[href^="http"]');

        links.forEach((link) => {
            if (!link.hostname.includes(window.location.hostname)) {
                link.setAttribute('target', '_blank');
                link.setAttribute('rel', 'noopener noreferrer');
                link.innerHTML += ' <span aria-hidden="true">↗</span>';
            }
        });
    }

    // Search enhancement
    function enhanceSearch() {
        const searchInput = document.getElementById('searchbar');
        if (!searchInput) return;

        // Add search keyboard shortcut (Ctrl+K or Cmd+K)
        document.addEventListener('keydown', (e) => {
            if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
                e.preventDefault();
                searchInput.focus();
            }
        });

        // Add placeholder hint
        searchInput.setAttribute('placeholder', 'Search (Ctrl+K)');
    }

    // Add version badge
    function addVersionBadge() {
        const version = 'v0.1.0'; // This should be dynamically set
        const sidebar = document.querySelector('.sidebar');
        if (!sidebar) return;

        const badge = document.createElement('div');
        badge.style.cssText = `
            text-align: center;
            padding: 10px;
            background: rgba(102, 126, 234, 0.1);
            color: #667eea;
            font-weight: 600;
            font-size: 0.9em;
        `;
        badge.textContent = version;

        const firstChild = sidebar.firstChild;
        sidebar.insertBefore(badge, firstChild);
    }

    // Code block line numbers
    function addLineNumbers() {
        const codeBlocks = document.querySelectorAll('pre > code');

        codeBlocks.forEach((codeBlock) => {
            const lines = codeBlock.textContent.split('\n');
            if (lines.length > 5) { // Only add for blocks with 5+ lines
                const lineNumbers = document.createElement('div');
                lineNumbers.className = 'line-numbers';
                lineNumbers.style.cssText = `
                    position: absolute;
                    left: 0;
                    padding: 10px 0;
                    text-align: right;
                    color: #999;
                    user-select: none;
                    width: 40px;
                `;

                for (let i = 1; i <= lines.length; i++) {
                    lineNumbers.innerHTML += i + '<br>';
                }

                const pre = codeBlock.parentElement;
                pre.style.paddingLeft = '50px';
                pre.insertBefore(lineNumbers, codeBlock);
            }
        });
    }

    // Print optimization
    function optimizePrint() {
        window.addEventListener('beforeprint', () => {
            // Expand all collapsed sections
            document.querySelectorAll('details').forEach((details) => {
                details.setAttribute('open', '');
            });
        });
    }

    // Analytics (placeholder)
    function initAnalytics() {
        // Add your analytics code here
        // Example: Google Analytics, Plausible, etc.
    }

    // Initialize all features
    function init() {
        // Run on DOM ready
        if (document.readyState === 'loading') {
            document.addEventListener('DOMContentLoaded', init);
            return;
        }

        addCopyButtons();
        addLanguageLabels();
        addAnchorLinks();
        smoothScroll();
        addBackToTop();
        enhanceTables();
        markExternalLinks();
        enhanceSearch();
        addVersionBadge();
        optimizePrint();
        initAnalytics();

        console.log('MeCab-Ko documentation enhancements loaded');
    }

    init();
})();
