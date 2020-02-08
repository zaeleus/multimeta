# multimeta

[![GitHub Actions status](https://github.com/zaeleus/multimeta/workflows/CI/badge.svg)](https://github.com/zaeleus/multimeta/actions)

**multimeta** is a music metadata scraper for multiple Asian music stores. It
currently supports [Melon] (KR) and [Mora] (JP). The output is a relational,
multi-file TOML format, which can be seen in [lp-seeds].

[Melon]: http://www.melon.com/
[Mora]: http://mora.jp/
[lp-seeds]: https://github.com/zaeleus/lp-seeds

## Prerequisites

  * [mozjpeg] ^3.0.0 (provides `cjpeg` and `jpegtran`)
  * [rust] ^1.39.0

[mozjpeg]: https://github.com/mozilla/mozjpeg
[rust]: https://www.rust-lang.org/
