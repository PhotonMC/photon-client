//! Where the kit's repeated elements go: a column of rows and a wrapping grid of cards.

use crate::layout::Rect;

/// `count` rows of `row_height`, one under the other with `gap` between, from the top of `area`.
pub fn column(area: Rect, count: usize, (row_height, gap): (i32, i32)) -> Vec<Rect> {
    (0..count as i32)
        .map(|index| Rect {
            x: area.x,
            y: area.y + index * (row_height + gap),
            width: area.width,
            height: row_height,
        })
        .collect()
}

/// `count` cards of `size` that wrap into rows inside `area`, `gap` apart. A card that does not fit
/// in the width starts the next row; at least one card goes in every row.
pub fn cards(area: Rect, count: usize, (width, height): (i32, i32), gap: i32) -> Vec<Rect> {
    let per_row = ((area.width + gap) / (width + gap)).max(1) as usize;
    (0..count)
        .map(|index| Rect {
            x: area.x + (index % per_row) as i32 * (width + gap),
            y: area.y + (index / per_row) as i32 * (height + gap),
            width,
            height,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const AREA: Rect = Rect {
        x: 10,
        y: 20,
        width: 100,
        height: 200,
    };

    #[test]
    fn a_column_stacks_rows_with_the_gap_between() {
        let rows = column(AREA, 3, (12, 2));
        assert_eq!(rows.iter().map(|r| r.y).collect::<Vec<_>>(), [20, 34, 48]);
        assert!(
            rows.iter()
                .all(|r| r.x == 10 && r.width == 100 && r.height == 12)
        );
    }

    #[test]
    fn cards_wrap_when_the_width_is_full() {
        // Two of 40 fit in 100 with a gap of 8 (88), a third would need 136.
        let cards = cards(AREA, 3, (40, 30), 8);
        assert_eq!((cards[0].x, cards[0].y), (10, 20));
        assert_eq!((cards[1].x, cards[1].y), (58, 20));
        assert_eq!((cards[2].x, cards[2].y), (10, 58));
    }

    #[test]
    fn a_card_wider_than_the_area_still_gets_a_row_of_its_own() {
        let cards = cards(AREA, 2, (150, 30), 8);
        assert_eq!((cards[0].x, cards[0].y), (10, 20));
        assert_eq!((cards[1].x, cards[1].y), (10, 58));
    }
}
