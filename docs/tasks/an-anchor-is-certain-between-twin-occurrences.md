---
title: An anchor is certain between twin occurrences
part_of: '[Tasks](tasks.md)'
created: 2026-11-08
status: open
description: '`anchor` calls a quote certain when it occurs more than once and both recorded sides agree at more than one of those places, because a full score at the first is taken as choosing it; `apply` then replaces the first of two passages the context cannot tell apart, and says it is sure'
---

# An anchor is certain between twin occurrences

[`Anchor::certain`](../../src/annotation.rs) is whether the recorded prefix
and suffix agree with what surrounds the quote "and picks this occurrence
out"; `false` is "one of several occurrences … without both sides recorded
and agreeing to choose it". Before the last fix it read "one of several
occurrences, with nothing to choose between them". Either way, a quote whose
recorded context agrees at two places has not been chosen.

`anchor` keeps the first occurrence with the highest score and calls it
certain when that score is 4, both sides recorded and agreeing, without
asking whether another occurrence scored 4 too. So a passage that recurs
with the same few characters on each side (a refrain, a repeated list item,
a table row, boilerplate between sections) is anchored at its first
occurrence and called certain, and `apply` hands that certainty to the store,
which may take the edit without a hand. The reader may have meant the second.

Found reading `anchor` at 9d216b2, after
[An anchor is certain where its context disagrees](an-anchor-is-certain-where-its-context-disagrees.md).

## Repro

```rust
use historica_remark::{Annotation, Selector, anchor, apply};

let text = "Refrain: the lake froze.\nVerse.\nRefrain: the lake froze.\n";
let sel = Selector::in_context("the lake", "Refrain: ", " froze");

// Both occurrences have "Refrain: " before and " froze" after.
assert!(!anchor(&sel, text).unwrap().certain); // fails: certain is true

let edit = Annotation::edit("doc", "3f9c", sel, "the river");
let applied = apply(&edit, text).unwrap().unwrap();
// "Refrain: the river froze.\nVerse.\nRefrain: the lake froze.\n"
assert!(!applied.anchor.certain); // fails: called certain
```

Run as an integration test against `main` at 9d216b2, both assertions fail
the same way.

## Why

`certain` is `!disagrees && (score == 4 || occurrences == 1)`, where `score`
and `disagrees` are the first occurrence's to reach the highest score. The
loop replaces the best only on a strictly higher score, so a later occurrence
that ties it is visited and forgotten. `score == 4` stands in for "the context
chose this one", which holds only when no other occurrence also scores 4.

## Done when

`certain` is false whenever more than one occurrence reaches the winning
score, the first of them is still the one returned (the Web Annotation rule's
"failing that, the first"), the two assertions above are tests in
`src/annotation.rs`, and the existing anchor tests pass as they are,
`a_quote_is_found_where_its_context_agrees` and the `"ban"` case of
`a_quote_overlapping_itself_is_more_than_one_occurrence` among them.
