---
title: Configuration
description: Configure validation rules, severity overrides, and spec selection.
---

imferno validates with sensible defaults, but every rule can be tuned via a rules config.

## Rules config

Every validation code can be set to one of five severities: `off`, `info`, `warn`, `error`, or `critical`. Pass a rules config to suppress known deviations or promote checks that matter to your workflow.

### JSON (CLI)

Create a `rules.json`:

```json
{
  "ST2067-2:2016:8/DigitalSignature": "off",
  "ST2067-2:2020:8.3/FileNotFound": "critical"
}
```

```sh
imferno validate ./my-package --rules-config rules.json
```

### Rust

Every validation code has a typed enum variant — no raw strings needed:

```rust
use imferno_core::assetmap::codes::St2067_2_2020;
use imferno_core::diagnostics::rules::{RulesConfig, RuleSeverity};
use imferno_core::package::{Imferno, ValidationOptions, read_dir};

let mut rules = RulesConfig::default();
rules.set(St2067_2_2020::FileNotFound, RuleSeverity::Critical);
rules.set(St2067_2_2020::ChecksumMismatch, RuleSeverity::Off);

let options = ValidationOptions {
    rules,
    ..Default::default()
};

let files = read_dir("/path/to/your.imp")?;
let report = Imferno::parse_and_validate(files, &options);
```

### JavaScript / TypeScript

Import the `codes` object for autocomplete and typo protection:

```typescript
import { validate, codes } from '@imferno/wasm';

const result = await validate(files, {
  rules: {
    [codes.ST2067_2_2020.FileNotFound]: "critical",
    [codes.ST2067_2_2020.ChecksumMismatch]: "off",
  },
});
```

Raw strings still work if you prefer:

```typescript
const result = await validate(files, {
  rules: {
    "ST2067-2:2020:8.3/FileNotFound": "critical",
  },
});
```

## Spec selection

imferno auto-detects the CPL's core spec version and application profile from the namespace declared in the XML. You can override this:

### CLI

```sh
# Force a specific core spec version
imferno validate ./my-package --core-spec v2020

# Force a specific application profile
imferno validate ./my-package --app2e-spec v2023
```

### Rust

```rust
use imferno_core::package::{ValidationOptions, CoreSpecTarget, AppSpecTarget};

let options = ValidationOptions {
    core_spec: Some(CoreSpecTarget::V2020),
    app_specs: Some(vec![AppSpecTarget::App2e2023]),
    ..Default::default()
};
```

## Available severities

| Severity | Meaning |
|---|---|
| `off` | Rule is disabled entirely |
| `info` | Informational — logged but does not affect compliance |
| `warn` | Spec deviation that may still work in practice |
| `error` | Spec non-conformance |
| `critical` | Package cannot be used |

See the [Validation Codes](/reference/codes/st2067-2/) reference for the full catalogue of rule codes.
