//! `DebugEntryMemory`: the memory in use, how fast it grows and what is allocated.

use super::super::layout::Line;

const MEBIBYTE: i64 = 1024 * 1024;

/// What the entry reads, in bytes (`Runtime.maxMemory`, `totalMemory` less `freeMemory`,
/// `totalMemory`) and the allocation rate it worked out, in bytes a second.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MemoryUsage {
    pub used: i64,
    pub total: i64,
    pub max: i64,
    pub rate: i64,
}

pub fn display(memory: &MemoryUsage) -> Vec<Line> {
    let mebibytes = |bytes: i64| bytes / MEBIBYTE;
    // `%2d` pads with blanks to two digits; `%03d` pads with zeros to three.
    vec![Line::Group(
        "minecraft:memory",
        vec![
            format!(
                "Mem: {:>2}% {:03}/{:03}MiB",
                memory.used * 100 / memory.max,
                mebibytes(memory.used),
                mebibytes(memory.max)
            ),
            format!("Allocation rate: {:03}MiB/s", mebibytes(memory.rate)),
            format!(
                "Allocated: {:>2}% {:03}MiB",
                memory.total * 100 / memory.max,
                mebibytes(memory.total)
            ),
        ],
    )]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(memory: MemoryUsage) -> Vec<String> {
        match display(&memory).remove(0) {
            Line::Group(_, lines) => lines,
            other => panic!("not a group: {other:?}"),
        }
    }

    #[test]
    fn the_figures_are_padded_as_the_format_says() {
        let gib = 1024 * MEBIBYTE;
        let shown = lines(MemoryUsage {
            used: 5 * MEBIBYTE,
            total: 600 * MEBIBYTE,
            max: 4 * gib,
            rate: 12 * MEBIBYTE + 1,
        });
        assert_eq!(shown[0], "Mem:  0% 005/4096MiB");
        assert_eq!(shown[1], "Allocation rate: 012MiB/s");
        assert_eq!(shown[2], "Allocated: 14% 600MiB");
    }

    #[test]
    fn a_shrinking_heap_has_a_negative_rate_that_rounds_towards_zero() {
        let shown = lines(MemoryUsage {
            used: 1,
            total: 1,
            max: 1024 * MEBIBYTE,
            rate: -(3 * MEBIBYTE + 5),
        });
        assert_eq!(shown[1], "Allocation rate: -03MiB/s");
    }
}
