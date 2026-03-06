---
title: IMF Packages
description: Understanding the structure of an IMP directory.
---

An IMF Package (IMP) is a directory. Everything in it is described by the ASSETMAP.

```
my-title.imp/
├── ASSETMAP.xml
├── PKL_<uuid>.xml
├── CPL_<uuid>.xml        # one or more
└── <uuid>.mxf            # track files (video, audio, subs, etc.)
```

## ASSETMAP

The entry point. Lists every file in the package with its UUID and relative path. imferno reads this first to build the file index.

```xml
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
  <Id>urn:uuid:...</Id>
  <AssetList>
    <Asset>
      <Id>urn:uuid:...</Id>
      <ChunkList>
        <Chunk>
          <Path>CPL_abc123.xml</Path>
        </Chunk>
      </ChunkList>
    </Asset>
    ...
  </AssetList>
</AssetMap>
```

## PKL — Packing List

Lists every file with its size and SHA-1 hash. Used for integrity verification. A package can have multiple PKLs (one per volume in a multi-volume delivery).

## CPL — Composition Playlist

The timeline. A CPL describes one version of the title — a specific language, territory, or rating cut. It references track files by UUID and defines how they are assembled into segments and sequences.

### Segments and sequences

A CPL contains one or more **segments** (acts or reels). Each segment contains **sequences** — parallel tracks:

- `MainImageSequence` — video
- `MainAudioSequence` — audio channels
- `SubtitlesSequence` — subtitles
- `ForcedNarrativeSequence` — forced narrative (burned-in subtitles)
- `HearingImpairedCaptionsSequence` — captions
- `IABSequence` — immersive audio (Dolby Atmos)

Each sequence contains **resources** — references to ranges within MXF track files, with entry point, source duration, and edit rate.

## Supplemental packages

A title can have multiple CPLs sharing the same essence. A supplemental CPL adds a new version (say, a Dutch dub) without duplicating the video MXF — it just points to the same file UUID.

imferno tracks supplemental relationships via the `isSupplemental` flag in each CPL entry of the report.

## Application profiles

CPLs declare which Application profile they conform to:

| Profile | Description |
|---|---|
| App#2 | Standard HD/UHD |
| App#2E | UHD with HDR (Dolby Vision, HDR10) |
| App#2Extended | Extended color / dynamic range |
| App#5 | IMAX |

imferno detects the profile from the `ApplicationIdentification` element in the CPL.
