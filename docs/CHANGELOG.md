# Changelog

All notable changes to historica-remark are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the versions
follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

<!-- git-cliff:begin — generated; edits here are overwritten -->

_No commits since the last tag._

<!-- git-cliff:end -->

## v0.3.0 — 2026-09-15

### Breaking

- **annotation** — an `editing` motivation, a `replacement` field, and `apply` ([`69af858`](https://github.com/diaryx-org/historica-remark/commit/69af858aea65f65007ed60032110572836c6c26c))

### Behavioural changes

- `Reader::read` and `Reader::document` now return

- `Motivation::ALL` and `Motivation::terms()` include
`editing`, so a store that declares the motivation vocabulary from
`terms()` now declares six terms; a store that declared five earlier
and validates against them will refuse an edit until it re-declares.

- `Annotation::fields()` and `FIELDS` include
`replacement`, written only when set.


## v0.2.0 — 2026-09-14

### Breaking

- **deps** — move to fig 4 ([`ae774a3`](https://github.com/diaryx-org/historica-remark/commit/ae774a3bf7366fbdb315d913b180846a1a140754))

### Behavioural changes

- historica-remark now requires `fig = "4"`. A consumer
still pinned to fig 3.x resolves two copies of fig — refused outright,
since fig-sys links the one native library — and `document::split`'s


## v0.1.1 — 2026-09-03

### Fixed

- **layer** — count the revisions a fetch names, now that it names them ([`94e2a60`](https://github.com/diaryx-org/historica-remark/commit/94e2a6004d147aa64e52d9c63095524828983c22))

## v0.1.0 — 2026-09-02

### Added

- remarks on a historica chain, and the layer that carries them ([`a0f495b`](https://github.com/diaryx-org/historica-remark/commit/a0f495b5f008435f7303e15e11cde3f240f5e1ca))

