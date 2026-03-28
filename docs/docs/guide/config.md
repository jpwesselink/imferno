---
title: Configuration
description: Configure validation rules, severity overrides, and spec selection.
---

imferno validates with sensible defaults, but every rule can be tuned via a rules config.

## Available severities

| Severity | Meaning |
|---|---|
| `off` | Rule is disabled entirely |
| `info` | Informational — logged but does not affect compliance |
| `warn` | Spec deviation that may still work in practice |
| `error` | Spec non-conformance |
| `critical` | Package cannot be used |

## Rules config

### CLI

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
use imferno_core::codes::St2067_2_2020;
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

### WASM

Import the `codes` object for autocomplete and typo protection:

```typescript
import { validate, codes } from '@imferno/wasm';

const result = await validate(files, {
    rules: {
        [codes.ST2067_2_2020.FileNotFound]: 'critical',
        [codes.ST2067_2_2020.ChecksumMismatch]: 'off',
    },
});
```

### Node.js

```typescript
import { buildReportFromPath, codes } from '@imferno/node';

const report = buildReportFromPath('./my-imp', {
    rules: {
        [codes.ST2067_21_2023.AppIdMismatch]: 'error',
        [codes.Imferno.UnreferencedAsset]: 'off',
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

# Disable application profile validation
imferno validate ./my-package --app2e-spec none
```

### Rust

```rust
use imferno_core::package::{ValidationOptions, CoreSpecTarget, AppSpecTarget};

let options = ValidationOptions {
    core_spec: Some(CoreSpecTarget::St2067_2_2020),
    app_specs: Some(vec![AppSpecTarget::St2067_21_2023]),
    ..Default::default()
};
```

**`CoreSpecTarget`**: `St2067_2_2013`, `St2067_2_2016`, `St2067_2_2020`

**`AppSpecTarget`**: `St2067_21_2020`, `St2067_21_2021`, `St2067_21_2023`

### WASM

```typescript
const result = await validate(files, {
    coreSpec: 'v2020',
    app2eSpec: 'v2023',
});
```

- `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"`
- `app2eSpec`: `"auto"` | `"none"` | `"v2020"` | `"v2021"` | `"v2023"`

### Node.js

```typescript
import { buildReportFromPath } from '@imferno/node';

const report = buildReportFromPath('./my-imp', {
    coreSpec: 'v2020',
    app2eSpec: 'v2023',
});
```

- `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"`
- `app2eSpec`: `"auto"` | `"none"` | `"v2020"` | `"v2021"` | `"v2023"`

See the [Validation Codes](/guide/codes/) reference for the full catalogue of rule codes.
