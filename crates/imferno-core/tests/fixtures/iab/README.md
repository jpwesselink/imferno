# IAB test fixtures

Hand-authored minimal CPLs targeting the **ST 2067-201:2026 Annex E**
recommendation (`IABChannelSubDescriptor` per `BedDefinition` channel).

These are **synthetic** — the 2026 publication shipped on 2026-03-25
without bundled XML examples (the zip only contains the PDF and the
HTML render of the spec body), so no canonical SMPTE-published IAB CPL
exists for 2026 to vendor. The fixtures here follow:

- ST 2067-3:2016 CPL skeleton (root `<CompositionPlaylist>`, `<Id>`,
  `<IssueDate>`, `<ContentTitle>`, `<EditRate>`,
  `<EssenceDescriptorList>`, `<SegmentList>`)
- IAB descriptor extension namespace `http://www.smpte-ra.org/ns/2067-201/2019`
  (confirmed firsthand from the 2026 PDF line 642 + the 2026 publication's
  inline HTML schema — the 2026 spec reuses the 2019 namespace)
- `IABChannelSubDescriptor` fields per **ST 2067-201:2026 Annex E Table E.1**
  (`IABBedMetaID` Uint32, `IABChannelID` Uint32, etc.)

When a SMPTE-RA-published or community-vendored IAB CPL with 2026 Annex E
shape becomes available, replace these synthetic files with the
canonical source.

## Files

- `cpl-iab-2026-conformant.xml` — IAB descriptor with two
  `IABChannelSubDescriptor` entries. Annex E recommendation met;
  `AppIabPlugin2026` emits no warning.

- `cpl-iab-2026-missing-channel-subdescriptors.xml` — IAB descriptor
  with **zero** `IABChannelSubDescriptor` entries (only the
  `IABSoundfieldLabelSubDescriptor` carried over from 2021). Annex E
  recommendation violated; `AppIabPlugin2026` emits the
  `ST2067-201:2026:5.10.2/IabChannelSubDescriptorRecommended` Warning.
  `AppIabPlugin2021` is silent on the same input (verified by
  `tests/iab_2026_fixture.rs`).
