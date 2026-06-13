# SCM (Sidecar Composition Map) test fixtures

`st2067-9b-2018.xml` is the canonical SidecarCompositionMap example
distributed alongside SMPTE ST 2067-9:2018 in
[`st2067-9-20180522-pub.zip`](https://pub.smpte.org/doc/st2067-9/20180522-pub/).
Vendored verbatim — BSD-3-Clause per the zip's `readme.txt`:

> Copyright © (as appropriate), SMPTE. All rights reserved.
> Redistribution and use in source and binary forms, with or without
> modification, are permitted provided that the following conditions
> are met: …

Used by `tests/scm_fixture.rs` to ensure the SCM parser handles the
canonical-shape document, not just hand-rolled XML.
