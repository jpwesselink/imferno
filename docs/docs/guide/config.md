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

Every finding names a rule and the edition of the spec that was applied
(`ST2067-3:2016:7.2.2/SegmentDuration`). A rules config maps keys to
severities; when several keys match a finding, the most specific wins:

| Key | Selects |
|---|---|
| `ST2067-3:2016/SegmentDuration` | the rule in one edition (a full code such as `ST2067-3:2016:7.2.2/SegmentDuration`, copied from a report, works too) |
| `ST2067-3/SegmentDuration` | the rule in every edition (the rule id) |
| `SegmentDuration` | every rule with that name, in any spec |
| `ST2067-3:2016:*` | every rule of one spec edition |
| `ST2067-3:*` | every rule of a spec |
| `source:XsdLayer` | every finding of a source (`XsdLayer`, `ProseRule`, `EngineInternal`) |

No other wildcards are supported. Keys that select nothing (typos, an edition
the rule does not have) are reported: as a warning on stderr by the CLI, and
in `rule_warnings` of the `validate*` result. A package is compliant when no finding has
an effective severity of `error` or above, so lowering a rule to `warn` makes
validation looser on purpose.

### CLI

Create a `rules.json`:

```json
{
  "ST2067-2/DigitalSignature": "off",
  "ST2067-2:2020/FileNotFound": "critical"
}
```

```sh
imferno validate ./my-package --rules-config rules.json

# Or inline (repeatable)
imferno validate ./my-package --rule SegmentDuration=off --rule FileNotFound=critical
```

### Rust

Every rule is a typed `static` in its spec's table — no raw strings needed:

```rust
use imferno_core::catalogue::st2067_2::{self, St2067_2Edition};
use imferno_core::diagnostics::rules::{RulesConfig, RuleSeverity};
use imferno_core::package::{Imferno, ValidationOptions, read_dir};

let mut rules = RulesConfig::default();
// One edition only:
rules.set_in_edition(&st2067_2::FileNotFound, St2067_2Edition::V2020, RuleSeverity::Critical);
// Every edition:
rules.set(&st2067_2::ChecksumMismatch, RuleSeverity::Off);

let options = ValidationOptions {
    rules,
    ..Default::default()
};

let files = read_dir("/path/to/your.imp")?;
let report = Imferno::parse_and_validate(files, &options);
```

### WASM

Import the `codes` object (rule ids per spec) for autocomplete and typo protection:

```typescript
import { validate, codes } from '@imferno/wasm';

const result = await validate(files, {
    rules: {
        [codes.ST2067_2.FileNotFound]: 'critical',   // "ST2067-2/FileNotFound"
        [codes.ST2067_2.ChecksumMismatch]: 'off',
    },
});
```

### Node.js

```typescript
import { validatePath, codes } from '@imferno/node';

const result = validatePath('./my-imp', {
    rules: {
        [codes.ST2067_21.AppIdMismatch]: 'error',
        [codes.IMFERNO.UnreferencedAsset]: 'off',
        'ST2067-21:2023/MaxCLLMaxFALL': 'off',  // one edition only
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
use imferno_core::validation::{CoreSpecTarget, AppSpecTarget};
use imferno_core::package::ValidationOptions;

let options = ValidationOptions {
    core_spec: Some(CoreSpecTarget::St2067_2_2020),
    app_specs: Some(vec![AppSpecTarget::St2067_21_2023]),
    ..Default::default()
};
```

**`CoreSpecTarget`**: `St2067_2_2013`, `St2067_2_2016`, `St2067_2_2020`

**`AppSpecTarget`**: `St2067_21_2014`, `St2067_21_2016`, `St2067_21_2020`, `St2067_21_2023`, `St2067_21_2026`

### WASM

```typescript
const result = await validate(files, {
    coreSpec: 'v2020',
    app2eSpec: 'v2023',
});
```

- `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"`
- `app2eSpec`: `"auto"` | `"none"` | `"v2014"` | `"v2016"` | `"v2020"` | `"v2023"` | `"v2026"`

### Node.js

```typescript
import { validatePath } from '@imferno/node';

const result = validatePath('./my-imp', {
    coreSpec: 'v2020',
    app2eSpec: 'v2023',
});
```

- `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"`
- `app2eSpec`: `"auto"` | `"none"` | `"v2014"` | `"v2016"` | `"v2020"` | `"v2023"` | `"v2026"`

`listRules()` (Node and WASM) returns the catalogue with each rule's edition table.
See the [Validation Codes](/guide/codes/) reference for the full catalogue.

## SARIF output

Findings can be exported as a [SARIF 2.1.0](https://docs.oasis-open.org/sarif/sarif/v2.1.0/) log for code-scanning tools:

```sh
imferno validate ./my-package --format sarif > imferno.sarif
```

```typescript
import { validatePath, toSarif } from '@imferno/node';
const log = toSarif(validatePath('./my-imp'));
```

Each rule is a SARIF rule (`ruleId` = `ST2067-3/SegmentDuration`); each spec
edition is a taxonomy (`ST2067-3:2016`) whose taxa are its clauses, and every
result points at the clause of the edition that was applied. Artifact URIs are
relative to the package root (`uriBaseId` `PKGROOT`, resolved in
`originalUriBaseIds`). Findings switched off by a rules config are included
with a suppression, at their original level.

## Which edition is applied

Package-level rules (Packing List, Asset Map, ingest) are checked against the
ST 2067-2 edition of the package: `--core-spec` / `coreSpec` when given;
else an ST 429-8 Packing List means ST 2067-2:2013, and the
`.../2067-2/2016/PKL` namespace (shared by 2016 and 2020) is resolved by the
Core Constraints namespace the CPLs declare. When that is not conclusive the
newest edition is used and an `IMFERNO:Package/EditionAssumed` notice says so.
ST 2067-2:2013 defines no delivery requirements, so for a 2013 package the
delivery rules of later editions are not checked and
`IMFERNO:Package/DeliveryNotChecked` is reported instead. Packing List Size
and Hash mismatches of a 2013 package are reported under ST 429-8:2007 (its
Packing List spec). ST 429-9:2007 and :2014 share the Asset Map namespace;
ST 429-9 rules are checked against 2014, and an `EditionAssumed` notice says
so when the report has ST 429-9 findings.
