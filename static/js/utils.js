/**
 * SOLID Utility Module - Single Responsibility Helpers
 */

export class TimeUtils {
    static formatRelativeTime(dateString) {
        if (!dateString) return '-';
        const date = new Date(dateString);
        if (isNaN(date.getTime())) return dateString.slice(0, 19).replace('T', ' ');

        const now = new Date();
        const diffSeconds = Math.floor((now.getTime() - date.getTime()) / 1000);

        if (diffSeconds < 30) return 'Just now';
        if (diffSeconds < 60) return `${diffSeconds}s ago`;

        const diffMinutes = Math.floor(diffSeconds / 60);
        if (diffMinutes < 60) return `${diffMinutes} ${diffMinutes === 1 ? 'min' : 'mins'} ago`;

        const diffHours = Math.floor(diffMinutes / 60);
        if (diffHours < 24) return `${diffHours} ${diffHours === 1 ? 'hr' : 'hrs'} ago`;

        const diffDays = Math.floor(diffHours / 24);
        if (diffDays < 7) return `${diffDays} ${diffDays === 1 ? 'day' : 'days'} ago`;

        return dateString.slice(0, 10);
    }
}

export class UrlValidator {
    static isValidLinkedInUrl(url) {
        if (!url || typeof url !== 'string') return false;
        const clean = url.trim().replace(/\/+$/, '');
        if (clean.includes('/404') || clean.includes('share') || clean.includes('intent')) return false;
        if (!clean.includes('linkedin.com/in/') && !clean.includes('linkedin.com/company/')) return false;
        const parts = clean.split('/in/').concat(clean.split('/company/')).filter(p => p && !p.includes('linkedin.com'));
        if (parts.length === 0) return false;
        const slug = parts[0].trim().replace(/\/+$/, '');
        return slug.length >= 2 && slug !== '404' && /[a-zA-Z0-9]/.test(slug);
    }
}

export class ClipboardUtils {
    static copyToClipboard(text, label, toastCallback) {
        if (!text || text === '-') return;
        try {
            if (navigator.clipboard && window.isSecureContext) {
                navigator.clipboard.writeText(text).then(() => {
                    if (toastCallback) toastCallback(`Copied ${label || 'text'}: ${text}`);
                }).catch(() => {
                    this.fallbackCopy(text, label, toastCallback);
                });
            } else {
                this.fallbackCopy(text, label, toastCallback);
            }
        } catch (e) {
            this.fallbackCopy(text, label, toastCallback);
        }
    }

    static fallbackCopy(text, label, toastCallback) {
        const textArea = document.createElement("textarea");
        textArea.value = text;
        textArea.style.position = "fixed";
        textArea.style.left = "-999999px";
        textArea.style.top = "-999999px";
        document.body.appendChild(textArea);
        textArea.focus();
        textArea.select();
        try {
            document.execCommand('copy');
            if (toastCallback) toastCallback(`Copied ${label || 'text'}: ${text}`);
        } catch (err) {
            console.error('Fallback copy failed', err);
        }
        document.body.removeChild(textArea);
    }
}
