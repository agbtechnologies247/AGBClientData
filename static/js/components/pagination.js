export function renderPaginationUI(prefix, totalCount, page, limit, onPageChange) {
    PaginationComponent.render(prefix, totalCount, page, limit, onPageChange);
}

export class PaginationComponent {
    static render(prefix, totalCount, page, limit, onPageChange) {
        const rangeEl = document.getElementById(`${prefix}Range`);
        const totalEl = document.getElementById(`${prefix}Total`);
        const indicatorEl = document.getElementById(`${prefix}PageIndicator`);
        const prevBtn = document.getElementById(`btnPrev${prefix.charAt(0).toUpperCase() + prefix.slice(1)}`);
        const nextBtn = document.getElementById(`btnNext${prefix.charAt(0).toUpperCase() + prefix.slice(1)}`);
        const limitSelect = document.getElementById(`${prefix}LimitSelect`);

        const totalPages = Math.max(1, Math.ceil(totalCount / limit));
        const startItem = totalCount === 0 ? 0 : (page - 1) * limit + 1;
        const endItem = Math.min(totalCount, page * limit);

        if (rangeEl) rangeEl.innerText = `${startItem}-${endItem}`;
        if (totalEl) totalEl.innerText = totalCount.toLocaleString();
        if (indicatorEl) indicatorEl.innerText = `Page ${page} of ${totalPages}`;

        if (prevBtn) {
            prevBtn.disabled = page <= 1;
            prevBtn.onclick = () => { if (page > 1) onPageChange(page - 1, limit); };
        }
        if (nextBtn) {
            nextBtn.disabled = page >= totalPages;
            nextBtn.onclick = () => { if (page < totalPages) onPageChange(page + 1, limit); };
        }
        if (limitSelect) {
            limitSelect.value = limit.toString();
            limitSelect.onchange = (e) => { onPageChange(1, parseInt(e.target.value, 10)); };
        }
    }
}
