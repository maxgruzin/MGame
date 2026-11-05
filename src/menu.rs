pub const PAGE_SIZE: usize = 7;

/// Move both the page and its selection. Even a one-page catalog cycles its arenas.
pub fn page_selection(selected: usize, count: usize, forward: bool) -> usize {
    if count == 0 {
        return 0;
    }
    let pages = count.div_ceil(PAGE_SIZE);
    if pages == 1 {
        return if forward {
            (selected + 1) % count
        } else {
            (selected + count - 1) % count
        };
    }
    let page = selected / PAGE_SIZE;
    let next = if forward {
        (page + 1) % pages
    } else {
        (page + pages - 1) % pages
    };
    (next * PAGE_SIZE + selected % PAGE_SIZE).min(count - 1)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn paging_changes_selected_arena_and_wraps() {
        assert_eq!(page_selection(0, 18, true), 7);
        assert_eq!(page_selection(7, 18, true), 14);
        assert_eq!(page_selection(14, 18, true), 0);
        assert_eq!(page_selection(0, 18, false), 14);
        assert_eq!(page_selection(13, 18, true), 17);
    }
    #[test]
    fn three_arena_catalog_still_responds_to_next_and_previous() {
        assert_eq!(page_selection(0, 3, true), 1);
        assert_eq!(page_selection(0, 3, false), 2);
    }
}
