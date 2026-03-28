pub const HINT_JS: &str = r#"
(function() {
    document.querySelectorAll('.vimr-hint').forEach(e => e.remove());

    const elements = Array.from(document.querySelectorAll(
        'a[href], button, input:not([type=hidden]), select, textarea, [role=button], [onclick], [tabindex]'
    )).filter(el => {
        const rect = el.getBoundingClientRect();
        return rect.width > 0 && rect.height > 0 &&
               rect.top >= 0 && rect.top <= window.innerHeight;
    });

    const chars = 'asdfghjklqwertyuiopzxcvbnm';
    const hints = [];

    elements.forEach((el, i) => {
        const label = [];
        let n = i;
        do {
            label.unshift(chars[n % chars.length]);
            n = Math.floor(n / chars.length) - 1;
        } while (n >= 0);

        const hint = label.join('');
        const rect = el.getBoundingClientRect();

        const span = document.createElement('div');
        span.className = 'vimr-hint';
        span.textContent = hint;
        span.dataset.hint = hint;
        span.dataset.elementIndex = i;
        span.style.cssText = `
            position: fixed;
            left: ${rect.left}px;
            top: ${rect.top}px;
            background: #ffff00;
            color: #000;
            font: bold 11px monospace;
            padding: 1px 3px;
            border: 1px solid #ccc;
            z-index: 999999;
            pointer-events: none;
        `;
        document.body.appendChild(span);
        hints.push({ hint, index: i, href: el.href || null });
    });

    window._vimrHints = elements;
    return JSON.stringify(hints);
})()
"#;

pub const HINT_ACTIVATE_JS: &str = r#"
(function(hintText) {
    const hints = document.querySelectorAll('.vimr-hint');
    let found = null;
    hints.forEach(h => {
        if (h.dataset.hint === hintText) {
            found = h;
        }
    });
    if (!found) return null;

    const idx = parseInt(found.dataset.elementIndex);
    const el = window._vimrHints[idx];
    document.querySelectorAll('.vimr-hint').forEach(e => e.remove());

    if (el) {
        el.focus();
        el.click();
        return el.href || el.tagName;
    }
    return null;
})
"#;

pub const HINT_CLEAR_JS: &str = r#"
document.querySelectorAll('.vimr-hint').forEach(e => e.remove());
window._vimrHints = [];
"#;

pub const SCROLL_JS: &str = r#"
(function(action) {
    const amount = 60;
    const halfPage = window.innerHeight / 2;
    const fullPage = window.innerHeight * 0.9;
    switch(action) {
        case 'down':  window.scrollBy(0, amount); break;
        case 'up':    window.scrollBy(0, -amount); break;
        case 'left':  window.scrollBy(-amount, 0); break;
        case 'right': window.scrollBy(amount, 0); break;
        case 'half-down':  window.scrollBy(0, halfPage); break;
        case 'half-up':    window.scrollBy(0, -halfPage); break;
        case 'page-down':  window.scrollBy(0, fullPage); break;
        case 'page-up':    window.scrollBy(0, -fullPage); break;
        case 'top':   window.scrollTo(0, 0); break;
        case 'bottom': window.scrollTo(0, document.body.scrollHeight); break;
    }
})
"#;
