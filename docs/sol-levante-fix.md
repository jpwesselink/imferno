# Sol Levante - Duur-fix

## Het probleem

De originele Sol Levante IMF-package heeft een SMPTE-overtreding: alle timed text
tracks (ondertitels, HI-ondertiteling, forced narrative) zijn 6288 frames lang,
terwijl de video 6314 frames lang is. ST 2067-3 sectie 7.2.2 vereist dat alle
virtual tracks in een segment dezelfde duur hebben.

### Photon output (origineel)

```
$ java -cp Photon-5.1.0-SNAPSHOT.jar com.netflix.imflibrary.app.IMPAnalyzer fixture/sol/

ERROR - Segment represented by the Id urn:uuid:468b550b-b215-494f-84f5-bb43bbda86f7
        seems to have sequences that are not of the same duration, following sequence
        durations were computed based on the information in the Sequence List for this
        Segment, 6288 6314 represented in Composition Edit Units

ERROR - Segment represented by the Id urn:uuid:fd169c19-adb2-4f92-9cc5-229995f73186
        seems to have sequences that are not of the same duration, following sequence
        durations were computed based on the information in the Sequence List for this
        Segment, 6288 6314 represented in Composition Edit Units
```

Photon vindt het probleem in beide CPL's maar verwijst naar segment-UUID's zonder
CPL-context.

### imferno output (origineel)

```
$ imferno validate --skip-hashes fixture/sol/

  ok  ASSETMAP.xml - 17 assets
  ok  2 CPL(s), 1 PKL(s)

CPL [b993cba5] Sol Levante UHD SDR FR (en)
  track  MainImage - 1920x1080 24fps JPEG 2000 (2K) ? SDR - f7879831
  track  MainAudio (ja) - 5.1 Surround 48.0kHz 24-bit - 02b1ad47
  track  Subtitles (en) - b0e3b943
  track  Subtitles (fr) - 4dd597ad
  track  HearingImpairedCaptions (en) - c8ce2c82
  track  HearingImpairedCaptions (fr) - 62418f3c
  track  ForcedNarrative (en) - de85a18b
  track  ForcedNarrative (fr) - 75bb55d7
  ... (+ 5 audio tracks)

CPL [6005b7eb] Sol Levante UHD HDR FR (en)
  ... (zelfde tracks, 4K Dolby Vision video)

Validation findings:
  error   ST2067-3:2016:7.2.2/SegmentDuration
          [CPL:b993cba5 CPL_b993cba5-...xml Sol Levante UHD SDR FR (en)]
          Segment 1 Subtitles track 86500a29: duration 262.000s differs from
          MainImage track b2e067c3: duration 263.083s -
          all virtual tracks in a segment shall have equal duration
  ... (12x hetzelfde: alle 6 timed text tracks x 2 CPL's)

  warning IMFERNO:Package/UnlistedEssence File '.DS_Store' ...
  warning IMFERNO:Package/UnlistedEssence File 'CPL_b993cba5-..._fix.xml' ...
  warning IMFERNO:Package/UnlistedEssence File 'CPL_6005b7eb-..._fix.xml' ...

failed 12 error(s), 3 warning(s)
```

imferno geeft per fout de exacte spec-clausule, CPL-bestandsnaam, contenttitel, en
welke track afwijkt van welke referentietrack.

## De fix

### Stap 1: TTML extraheren uit MXF

Alle 6 timed text MXF-bestanden bevatten IMSC1 timed text
(`http://www.w3.org/ns/ttml/profile/imsc1/text`), gewrapt in een MXF-container.
Met `as-02-unwrap` (asdcplib) is de TTML geextraheerd:

```
as-02-unwrap TimedText_4dd597ad.mxf /tmp/sol-ttml/sub1.xml
```

### Stap 2: MXF opnieuw wrappen met correcte duur

De TTML-bestanden zijn opnieuw gewrapt met `as-02-wrap`. De duur is op 6314 frames
gezet (gelijk aan de video). De `-a` flag stelt de AssetMap UUID in als MXF
PackageUID:

```
as-02-wrap -d 6314 -a 4dd597ad-c38f-4526-8c91-8ed13bf0bdf2 \
  -r 24/1 -P "http://www.w3.org/ns/ttml/profile/imsc1/text" \
  sub1.xml TimedText_4dd597ad.mxf
```

Herhaald voor alle 6 timed text bestanden.

### Stap 3: CPL's bijwerken

In beide CPL's zijn de volgende velden aangepast:

- `IntrinsicDuration` en `SourceDuration`: 6288 naar 6314
- `EssenceLength` in `DCTimedTextDescriptor`: 6288 naar 6314
- `ResourceID` en `InstanceID`: aangepast naar de nieuwe MXF-waarden
- `RFC5646LanguageTagList`: verwijderd (as-02-wrap behoudt deze niet)

### Stap 4: PKL hashes herberekenen

SHA-1 hashes herberekend voor alle 6 MXF-bestanden en beide CPL's, plus
bijgewerkte bestandsgroottes, in de PKL bijgewerkt.

## Resultaat

### Photon output (na fix)

```
$ java -cp Photon-5.1.0-SNAPSHOT.jar com.netflix.imflibrary.app.IMPAnalyzer fixture/sol-fixed/

TimedText_4dd597ad-c38f-4526-8c91-8ed13bf0bdf2.mxf has no errors or warnings
TimedText_c8ce2c82-1b5e-4b6d-a14d-b700f9f49730.mxf has no errors or warnings
CPL_6005b7eb-ad96-4937-872a-62629fad4bf1.xml has no errors or warnings
CPL_b993cba5-d112-4b02-b504-28166bf30024.xml has no errors or warnings
PKL_d325c8e7-fe63-4345-9c11-9af4f667cd07.xml has no errors or warnings
... (alle 17 bestanden: no errors or warnings)
```

### imferno output (na fix)

```
$ imferno validate --skip-hashes fixture/sol-fixed/

  ok  ASSETMAP.xml - 17 assets
  ok  2 CPL(s), 1 PKL(s)

CPL [6005b7eb] Sol Levante UHD HDR FR (en)
  track  MainImage - 3840x2160 24fps JPEG 2000 (4K) ? Dolby Vision - d15f7a99
  track  Subtitles - b0e3b943
  track  Subtitles - 4dd597ad
  track  HearingImpairedCaptions - c8ce2c82
  track  HearingImpairedCaptions - 62418f3c
  track  ForcedNarrative - de85a18b
  track  ForcedNarrative - 75bb55d7
  ... (+ 6 audio tracks)

CPL [b993cba5] Sol Levante UHD SDR FR (en)
  ... (zelfde tracks, 2K SDR video)

valid
```

0 fouten, 0 waarschuwingen. Beide validators zijn tevreden.

## Gebruikte tools

- [**imferno**](https://github.com/jpwesselink/imferno) v2.3.0 (Rust)
- [**Photon**](https://github.com/Netflix/photon) v5.1.0-SNAPSHOT (Netflix, Java)
- [**asdcplib**](https://github.com/cinecert/asdcplib) v2.13.3 (CineCert, C++) met [xerces-c](https://xerces.apache.org/xerces-c/) XML-ondersteuning
