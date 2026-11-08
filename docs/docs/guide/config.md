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
(`ST2067-3:2016:7.2/SegmentDuration`). A rules config maps keys to
severities; when several keys match a finding, the most specific wins:

| Key | Selects |
|---|---|
| `ST2067-3:2016/SegmentDuration` | the rule in one edition (a full code such as `ST2067-3:2016:7.2/SegmentDuration`, copied from a report, works too) |
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

# Force the Composition Playlist (ST 2067-3) edition
imferno validate ./my-package --cpl-spec v2016

# Force the IAB plug-in (ST 2067-201) edition
imferno validate ./my-package --iab-spec v2021

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

**`CplSpecTarget`** (`cpl_spec`): `St2067_3_2013`, `St2067_3_2016`, `St2067_3_2020`

**`IabSpecTarget`** (`iab_spec`): `St2067_201_2019`, `St2067_201_2021`, `St2067_201_2026`

**`AppSpecTarget`**: `St2067_21_2014`, `St2067_21_2016`, `St2067_21_2020`, `St2067_21_2023`, `St2067_21_2026`

### WASM

```typescript
const result = await validate(files, {
    coreSpec: 'v2020',
    app2eSpec: 'v2023',
});
```

- `coreSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"`
- `cplSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"` (ST 2067-3)
- `iabSpec`: `"auto"` | `"v2019"` | `"v2021"` | `"v2026"` (ST 2067-201)
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
- `cplSpec`: `"auto"` | `"v2013"` | `"v2016"` | `"v2020"` (ST 2067-3)
- `iabSpec`: `"auto"` | `"v2019"` | `"v2021"` | `"v2026"` (ST 2067-201)
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

Every edition choice follows one rule: an edition pinned by an option wins;
otherwise the document's namespace gives the candidate editions, and evidence
in the document that only some editions' text defines narrows them. When one
edition is left it is applied; otherwise the newest is applied and an
`IMFERNO:Package/EditionAssumed` (Info) notice names the namespace seen and the
edition used, one per document (package, CPL, MXF file). When the namespace
allows only one edition but other evidence names another, the namespace's
edition is applied and `IMFERNO:Package/EditionConflict` (Warning) names both
editions and the evidence.

- **ST 2067-2, package** (Packing List, Asset Map, ingest; `--core-spec` /
  `coreSpec`): an ST 429-8 Packing List means 2013; the `.../2067-2/2016/PKL`
  namespace is shared by 2016 and 2020, and the CPLs whose own Core
  Constraints edition is determined (below) decide.
- **ST 2067-2, CPL** (`--core-spec` / `coreSpec`): the 2013 CPL namespace means
  2013; the 2016 CPL namespace is shared by 2016 and 2020. Evidence: the
  namespace of the `ApplicationIdentification` element (else the Core
  Constraints namespaces the CPL declares), and the ST 2067-2 edition each
  listed App #2E edition references (2014 → 2013, 2016 → 2016, 2020/2023/2026
  → 2020).
- **ST 2067-3, CPL** (`--cpl-spec` / `cplSpec`): the 2013 namespace means 2013;
  2016 and 2020 share the 2016 namespace, and the only evidence is a Marker
  Label in the `2020#standard-markers` scope, which only 2020 defines. An
  ST 429-7 (D-Cinema) or unknown namespace is not an IMF Composition Playlist:
  it gets `ST2067-2/NotImfCompositionPlaylist` (Error) naming the namespace,
  and no ST 2067-3, Core Constraints or application rules.
- **ST 2067-201, CPL with IAB** (`--iab-spec` / `iabSpec`): every edition
  (2019, 2021, 2026) uses the 2019 namespace. ST 2067-201:2019 cites
  ST 2067-2:2016, 2021 and 2026 cite ST 2067-2:2020, so a determined or pinned
  Core Constraints edition names 2019, or leaves 2021 and 2026; an
  `IABMaxObjectCount` item or `IABChannelSubDescriptor` in the IAB essence
  descriptor (2026 only) names 2026. Otherwise the newest is applied and
  announced. The track files its IAB
  Virtual Tracks reference get the same edition.
- **ST 377-1 / ST 377-4, MXF track file**: MXF has no namespace; the dated
  normative references of the specs that apply to the file decide. Every
  ST 2067-2 edition references ST 377-1:2011; ST 2067-2:2020 references
  ST 377-4:2012 (2013 and 2016 cite "ST 377-4:2011", which never existed; it is read as
  ST 377-4:2012, the only edition then).
  For a track file a CPL IAB Virtual Track references, ST 2067-201 adds its
  references (2021: ST 377-1:2019; 2026: ST 377-1:2019 and ST 377-4:2021).
  When these do not leave one edition, the newest is applied, announced for
  the file when it has findings of that spec. ST 2067-201 rules on a track
  file (IAB labeling) use the edition the referencing CPL's IAB plug-in
  applies; a file no CPL IAB Virtual Track references gets the newest
  (2026), announced.

ST 2067-2:2013 defines no delivery requirements, so for a 2013 package the
delivery rules of later editions are not checked and
`IMFERNO:Package/DeliveryNotChecked` is reported instead. Packing List Size
and Hash mismatches of a 2013 package are reported under ST 429-8:2007 (its
Packing List spec). ST 429-9:2007 and :2014 share the Asset Map namespace;
ST 429-9 rules are checked against 2014, and an `EditionAssumed` notice says
so when the report has ST 429-9 findings.
