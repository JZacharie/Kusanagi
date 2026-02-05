/**
 * ANSI Color Code Parser for Log Display
 * Converts ANSI escape sequences to HTML with styling
 */

const AnsiParser = {
    // ANSI color map to CSS classes
    colorMap: {
        // Reset
        '0': 'ansi-reset',
        
        // Colors (30-37 foreground, 40-47 background)
        '30': 'ansi-black',
        '31': 'ansi-red',
        '32': 'ansi-green', 
        '33': 'ansi-yellow',
        '34': 'ansi-blue',
        '35': 'ansi-magenta',
        '36': 'ansi-cyan',
        '37': 'ansi-white',
        '90': 'ansi-bright-black',
        '91': 'ansi-bright-red',
        '92': 'ansi-bright-green',
        '93': 'ansi-bright-yellow',
        '94': 'ansi-bright-blue',
        '95': 'ansi-bright-magenta',
        '96': 'ansi-bright-cyan',
        '97': 'ansi-bright-white',
        
        // Styles
        '1': 'ansi-bold',
        '2': 'ansi-dim',
        '3': 'ansi-italic',
        '4': 'ansi-underline',
    },

    /**
     * Parse ANSI codes in text and convert to HTML
     */
    parse(text) {
        if (!text) return '';
        
        // Replace HTML special chars first
        let html = this.escapeHtml(text);
        
        // Parse ANSI escape sequences
        // Pattern matches: \x1b[ or \x1b\x5b followed by numbers and letters
        const ansiPattern = /\x1b\[([0-9;]*)m/g;
        
        let result = '';
        let lastIndex = 0;
        let openSpans = 0;
        const activeStyles = new Set();
        
        // Simple approach: replace ANSI codes with spans
        html = html.replace(/\x1b\[([0-9;]+)m/g, (match, codes) => {
            const codeList = codes.split(';');
            const classes = [];
            
            for (const code of codeList) {
                if (code === '0' || code === '') {
                    // Reset - close all spans
                    if (openSpans > 0) {
                        const closeTags = '</span>'.repeat(openSpans);
                        openSpans = 0;
                        activeStyles.clear();
                        return closeTags;
                    }
                    return '';
                }
                
                const className = this.colorMap[code];
                if (className) {
                    classes.push(className);
                    activeStyles.add(className);
                }
            }
            
            if (classes.length > 0) {
                openSpans++;
                return `<span class="${classes.join(' ')}">`;
            }
            return '';
        });
        
        // Close any remaining open spans
        if (openSpans > 0) {
            html += '</span>'.repeat(openSpans);
        }
        
        return html;
    },

    /**
     * Parse and wrap in a styled container
     */
    parseToHtml(text) {
        const parsed = this.parse(text);
        return `<div class="ansi-log">${parsed}</div>`;
    },

    /**
     * Strip ANSI codes (for plain text display)
     */
    strip(text) {
        if (!text) return '';
        return text.replace(/\x1b\[[0-9;]*m/g, '');
    },

    /**
     * Escape HTML special characters
     */
    escapeHtml(text) {
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    },

    /**
     * Format log with timestamp highlighting
     */
    formatLogLine(line) {
        // Highlight timestamps (ISO format)
        line = line.replace(
            /(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+Z)/g,
            '<span class="log-timestamp">$1</span>'
        );
        
        // Highlight log levels
        line = line.replace(
            /\b(ERROR|FATAL|CRITICAL)\b/g,
            '<span class="log-level-error">$1</span>'
        );
        line = line.replace(
            /\b(WARN|WARNING)\b/g,
            '<span class="log-level-warn">$1</span>'
        );
        line = line.replace(
            /\b(INFO)\b/g,
            '<span class="log-level-info">$1</span>'
        );
        line = line.replace(
            /\b(DEBUG)\b/g,
            '<span class="log-level-debug">$1</span>'
        );
        
        // Highlight module names (e.g., `kusanagi::newsfeed`)
        line = line.replace(
            /\b([a-z_]+::[a-z_:]+)\b/g,
            '<span class="log-module">$1</span>'
        );
        
        return line;
    }
};

// Expose globally
window.AnsiParser = AnsiParser;
