---
title: An anchor is certain where its context disagrees
part_of: '[Tasks](tasks.md)'
created: 2026-11-07T01:43:52.646473Z
status: open
description: '`anchor` calls a quote certain when one recorded side of its context disagrees, because that scores the same as two sides never recorded, and when the quote overlaps a second occurrence of itself, because `str::matches` counts only one; `apply` reports the same certainty for a replacement that may have landed in the wrong place'
---

# An anchor is certain where its context disagrees

[`Anchor::certain`](../../src/annotation.rs) promises less than `anchor` gives.
Its doc says it is whether "the recorded prefix and suffix both agreed", and
that `false` is a quote found in changed surroundings or one of several
occurrences with nothing to choose between them. `apply` hands the same value
to a caller, and the module note leaves that caller to decide whether an anchor
is certain enough to take an edit without a hand. So a `true` that should be
`false` is a replacement taken on the wrong grounds.

Two shapes get it wrong, at 4a981d0.

## Repro

```rust
use historica_remark::{Annotation, Selector, anchor, apply};

let text = "We crossed the lake froze over.";

// The suffix was recorded and is gone; only the prefix agrees.
let sel = Selector::in_context("lake", "the ", " thawed");
assert!(!anchor(&sel, text).unwrap().certain); // fails: certain is true

// The same with the sides the other way round.
let sel = Selector::in_context("lake", "a ", " froze");
assert!(!anchor(&sel, text).unwrap().certain); // fails: certain is true

// "ana" is in "banana" twice, at 1 and at 3, and nothing chooses between them.
assert!(!anchor(&Selector::quoting("ana"), "banana").unwrap().certain); // fails

let edit = Annotation::edit("doc", "3f9c", Selector::quoting("ana"), "X");
let applied = apply(&edit, "banana").unwrap().unwrap();
assert!(!applied.anchor.certain); // fails: "bXna", called certain
```

With both sides changed the answer is right (`certain` is false), and so it
is for a quote that recurs without overlapping (`"ab"` in `"ab ab"`).

## Why

`certain` is `score == 4 || (occurrences == 1 && score >= 2)`, where each side
scores two if it agrees, none if it disagrees, and one if it was never recorded.

- **One side agreeing and one disagreeing scores 2**, the same as two sides
  never recorded. `score >= 2` is there for the second case, which
  `a_recurring_quote_with_no_context_to_choose_by_is_uncertain` pins as certain
  ("unique, and no context recorded to disagree"), and it lets the first through.
- **`occurrences` is `text.matches(&selector.exact).count()`**, which does not
  count overlapping matches, while the loop above it steps a character at a time
  and does visit them. The two disagree on how many candidates there were.

## Done when

`certain` is false whenever a recorded side disagrees, and whenever the quote
has more than one occurrence, overlapping ones included, unless both recorded
sides agree. The occurrences are counted by the loop that visits them, the four
assertions above are tests in `src/annotation.rs`, and the existing anchor
tests pass as they are.
