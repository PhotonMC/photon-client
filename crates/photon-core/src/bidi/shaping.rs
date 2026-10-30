//! The Arabic joining pass of the reordering: letters take their isolated, final, initial or
//! medial form, a lam followed by an alef becomes one ligature, and a vowel mark between two
//! joined letters takes the medial form of its own. Only the letters of the basic Arabic block
//! and the presentation forms take part; every other character passes through and joins nothing.

/// The letters that have presentation forms, each with the code point of its isolated form. The
/// final form follows it, and a letter that joins on both sides has an initial and a medial one
/// after that (`true`). Hamza has the forms of a letter that never joins and stays as it is.
const LETTERS: &[(u32, u32, bool)] = &[
    (0x0622, 0xFE81, false),
    (0x0623, 0xFE83, false),
    (0x0624, 0xFE85, false),
    (0x0625, 0xFE87, false),
    (0x0626, 0xFE89, true),
    (0x0627, 0xFE8D, false),
    (0x0628, 0xFE8F, true),
    (0x0629, 0xFE93, false),
    (0x062A, 0xFE95, true),
    (0x062B, 0xFE99, true),
    (0x062C, 0xFE9D, true),
    (0x062D, 0xFEA1, true),
    (0x062E, 0xFEA5, true),
    (0x062F, 0xFEA9, false),
    (0x0630, 0xFEAB, false),
    (0x0631, 0xFEAD, false),
    (0x0632, 0xFEAF, false),
    (0x0633, 0xFEB1, true),
    (0x0634, 0xFEB5, true),
    (0x0635, 0xFEB9, true),
    (0x0636, 0xFEBD, true),
    (0x0637, 0xFEC1, true),
    (0x0638, 0xFEC5, true),
    (0x0639, 0xFEC9, true),
    (0x063A, 0xFECD, true),
    (0x0641, 0xFED1, true),
    (0x0642, 0xFED5, true),
    (0x0643, 0xFED9, true),
    (0x0644, 0xFEDD, true),
    (0x0645, 0xFEE1, true),
    (0x0646, 0xFEE5, true),
    (0x0647, 0xFEE9, true),
    (0x0648, 0xFEED, false),
    (0x0649, 0xFEEF, false),
    (0x064A, 0xFEF1, true),
];

const HAMZA: u32 = 0x0621;
const TATWEEL: u32 = 0x0640;
const LAM: u32 = 0x0644;

/// The vowel marks (fathatan to sukun) and the isolated form of each; the medial form is the next
/// code point for the marks that have one (`true`), and the isolated form otherwise.
const MARKS: &[(u32, u32, bool)] = &[
    (0x064B, 0xFE70, true),
    (0x064C, 0xFE72, false),
    (0x064D, 0xFE74, false),
    (0x064E, 0xFE76, true),
    (0x064F, 0xFE78, true),
    (0x0650, 0xFE7A, true),
    (0x0651, 0xFE7C, true),
    (0x0652, 0xFE7E, true),
];

/// The lam-alef ligatures, each as the code point standing for it while shaping, the alef it
/// swallows and its isolated form; the final form follows.
const LIGATURES: &[(u32, u32, u32)] = &[
    (0x065C, 0x0622, 0xFEF5),
    (0x065D, 0x0623, 0xFEF7),
    (0x065E, 0x0625, 0xFEF9),
    (0x065F, 0x0627, 0xFEFB),
];

/// How a character takes part in joining.
#[derive(Clone, Copy, PartialEq)]
enum Link {
    /// Joins nothing and breaks the joining around it.
    None,
    /// Joins the letter before it only.
    Right,
    /// Joins the letters on both sides.
    Dual,
    /// Does not take part: the letters around it join through it.
    Transparent,
}

/// The text with its Arabic letters in their joined forms.
pub fn shape(text: &str) -> String {
    if !text
        .chars()
        .any(|c| matches!(u32::from(c), 0x0621..=0x06D3 | 0xFE70..=0xFEFC))
    {
        return text.to_owned();
    }
    let items = ligated(text.chars().map(|c| base_of(u32::from(c))).collect());
    let links: Vec<Link> = items.iter().map(|&item| link_of(item)).collect();
    let mut shaped = String::with_capacity(text.len());
    for (index, &item) in items.iter().enumerate() {
        let before = joins_forward(&items, &links, index);
        let after = joins_backward(&links, index);
        shaped.push(form_of(item, links[index], before, after));
    }
    shaped
}

/// The character under a presentation form, and every other character as itself.
fn base_of(code: u32) -> u32 {
    match code {
        HAMZA | 0xFE80 => HAMZA,
        0xFE70..=0xFE7F => MARKS
            .iter()
            .find(|&&(_, isolated, _)| code == isolated || code == isolated + 1)
            .map_or(code, |&(mark, _, _)| mark),
        0xFEF5..=0xFEFC => LIGATURES
            .iter()
            .find(|&&(_, _, isolated)| code == isolated || code == isolated + 1)
            .map_or(code, |&(ligature, _, _)| ligature),
        0xFE81..=0xFEF4 => LETTERS
            .iter()
            .find(|&&(_, isolated, dual)| code >= isolated && code < isolated + forms(dual))
            .map_or(code, |&(letter, _, _)| letter),
        _ => code,
    }
}

/// How many presentation forms a letter has.
fn forms(dual: bool) -> u32 {
    if dual { 4 } else { 2 }
}

/// A lam followed by an alef swallows it into the ligature.
fn ligated(items: Vec<u32>) -> Vec<u32> {
    let mut result = Vec::with_capacity(items.len());
    let mut rest = items.into_iter().peekable();
    while let Some(item) = rest.next() {
        let ligature = (item == LAM)
            .then(|| rest.peek().copied())
            .flatten()
            .and_then(|next| LIGATURES.iter().find(|&&(_, alef, _)| alef == next));
        if let Some(&(ligature, _, _)) = ligature {
            rest.next();
            result.push(ligature);
        } else {
            result.push(item);
        }
    }
    result
}

fn link_of(item: u32) -> Link {
    if let Some(&(_, _, dual)) = LETTERS.iter().find(|&&(letter, _, _)| letter == item) {
        return if dual { Link::Dual } else { Link::Right };
    }
    match item {
        TATWEEL => Link::Dual,
        0x064B..=0x0655 | 0x0670 => Link::Transparent,
        0x065C..=0x065F => Link::Right,
        // The letters of the other languages the pass links: Persian, Urdu and the like.
        0x0672..=0x0673
        | 0x0675..=0x0677
        | 0x0688..=0x0699
        | 0x06C0
        | 0x06C2..=0x06CB
        | 0x06CD
        | 0x06D2..=0x06D3 => Link::Right,
        0x0678..=0x0687 | 0x069A..=0x06BF | 0x06C1 | 0x06CC | 0x06CE..=0x06D1 => Link::Dual,
        _ => Link::None,
    }
}

/// Whether the letter before `index` joins forward to it: the nearest one that is not transparent.
fn joins_forward(items: &[u32], links: &[Link], index: usize) -> bool {
    let _ = items;
    links[..index]
        .iter()
        .rev()
        .find(|&&link| link != Link::Transparent)
        .is_some_and(|&link| link == Link::Dual)
}

/// Whether the letter after `index` joins back to it.
fn joins_backward(links: &[Link], index: usize) -> bool {
    links[index + 1..]
        .iter()
        .find(|&&link| link != Link::Transparent)
        .is_some_and(|&link| link == Link::Dual || link == Link::Right)
}

/// The character drawn for `item` between the neighbours that join it or not.
fn form_of(item: u32, link: Link, before: bool, after: bool) -> char {
    let code = if let Some(&(_, isolated, dual)) =
        LETTERS.iter().find(|&&(letter, _, _)| letter == item)
    {
        match (before, after && dual) {
            (false, false) => isolated,
            (true, false) => isolated + 1,
            (false, true) => isolated + 2,
            (true, true) => isolated + 3,
        }
    } else if let Some(&(_, isolated, medial)) = MARKS.iter().find(|&&(mark, _, _)| mark == item) {
        isolated + u32::from(medial && before && after)
    } else if let Some(&(_, _, isolated)) =
        LIGATURES.iter().find(|&&(ligature, _, _)| ligature == item)
    {
        isolated + u32::from(before)
    } else {
        item
    };
    let _ = link;
    char::from_u32(code).unwrap_or('\u{FFFD}')
}
