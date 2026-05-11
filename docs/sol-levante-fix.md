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

### imferno output (origineel)

```
$ imferno validate --skip-hashes fixture/sol/

  ok  ASSETMAP.xml - 17 assets
  ok  2 CPL(s), 1 PKL(s)

CPL [b993cba5] Sol Levante UHD SDR FR (en)
  track  MainImage - 1920x1080 24fps JPEG 2000 (2K) ? SDR - f7879831
  track  MainAudio (ja) - 5.1 Surround 48.0kHz 24-bit - 02b1ad47
  track  MainAudio (ja) - Stereo 48.0kHz 24-bit - 84557505
  track  MainAudio (en) - Stereo 48.0kHz 24-bit - 2d05dacf
  track  MainAudio (en) - 5.1 Surround 48.0kHz 24-bit - 20506d44
  track  MainAudio (ja) - Stereo 48.0kHz 24-bit - 6edb0d2e
  track  MainAudio (en) - Stereo 48.0kHz 24-bit - 2aeeb0cd
  track  Subtitles (en) - b0e3b943
  track  Subtitles (fr) - 4dd597ad
  track  HearingImpairedCaptions (en) - c8ce2c82
  track  HearingImpairedCaptions (fr) - 62418f3c
  track  ForcedNarrative (en) - de85a18b
  track  ForcedNarrative (fr) - 75bb55d7

CPL [6005b7eb] Sol Levante UHD HDR FR (en)
  track  MainImage - 3840x2160 24fps JPEG 2000 (4K) ? Dolby Vision - d15f7a99
  ...

Validation findings:
  error   ST2067-3:2016:7.2.2/SegmentDuration
          [CPL:b993cba5 CPL_b993cba5-d112-4b02-b504-28166bf30024.xml Sol Levante UHD SDR FR (en)]
          Segment 1 Subtitles track 86500a29: duration 262.000s differs from MainImage track
          b2e067c3: duration 263.083s - all virtual tracks in a segment shall have equal duration
  error   ST2067-3:2016:7.2.2/SegmentDuration
          [CPL:b993cba5 CPL_b993cba5-d112-4b02-b504-28166bf30024.xml Sol Levante UHD SDR FR (en)]
          Segment 1 Subtitles track e5ac3808: duration 262.000s differs from MainImage track
          b2e067c3: duration 263.083s - all virtual tracks in a segment shall have equal duration
  error   ST2067-3:2016:7.2.2/SegmentDuration
          [CPL:b993cba5 CPL_b993cba5-d112-4b02-b504-28166bf30024.xml Sol Levante UHD SDR FR (en)]
          Segment 1 HICaptions track 052008cd: duration 262.000s differs from MainImage track
          b2e067c3: duration 263.083s - all virtual tracks in a segment shall have equal duration
  error   ST2067-3:2016:7.2.2/SegmentDuration
          [CPL:b993cba5 CPL_b993cba5-d112-4b02-b504-28166bf30024.xml Sol Levante UHD SDR FR (en)]
          Segment 1 HICaptions track f4c6541b: duration 262.000s differs from MainImage track
          b2e067c3: duration 263.083s - all virtual tracks in a segment shall have equal duration
  error   ST2067-3:2016:7.2.2/SegmentDuration
          [CPL:b993cba5 CPL_b993cba5-d112-4b02-b504-28166bf30024.xml Sol Levante UHD SDR FR (en)]
          Segment 1 ForcedNarrative track ffb7be52: duration 262.000s differs from MainImage track
          b2e067c3: duration 263.083s - all virtual tracks in a segment shall have equal duration
  error   ST2067-3:2016:7.2.2/SegmentDuration
          [CPL:b993cba5 CPL_b993cba5-d112-4b02-b504-28166bf30024.xml Sol Levante UHD SDR FR (en)]
          Segment 1 ForcedNarrative track 13244713: duration 262.000s differs from MainImage track
          b2e067c3: duration 263.083s - all virtual tracks in a segment shall have equal duration
  error   ST2067-3:2016:7.2.2/SegmentDuration
          [CPL:6005b7eb CPL_6005b7eb-ad96-4937-872a-62629fad4bf1.xml Sol Levante UHD HDR FR (en)]
          Segment 1 Subtitles track 05c4ccd5: duration 262.000s differs from MainImage track
          9e470d1a: duration 263.083s - all virtual tracks in a segment shall have equal duration
  error   ST2067-3:2016:7.2.2/SegmentDuration
          [CPL:6005b7eb CPL_6005b7eb-ad96-4937-872a-62629fad4bf1.xml Sol Levante UHD HDR FR (en)]
          Segment 1 Subtitles track 5096dbe8: duration 262.000s differs from MainImage track
          9e470d1a: duration 263.083s - all virtual tracks in a segment shall have equal duration
  error   ST2067-3:2016:7.2.2/SegmentDuration
          [CPL:6005b7eb CPL_6005b7eb-ad96-4937-872a-62629fad4bf1.xml Sol Levante UHD HDR FR (en)]
          Segment 1 HICaptions track 339c3b97: duration 262.000s differs from MainImage track
          9e470d1a: duration 263.083s - all virtual tracks in a segment shall have equal duration
  error   ST2067-3:2016:7.2.2/SegmentDuration
          [CPL:6005b7eb CPL_6005b7eb-ad96-4937-872a-62629fad4bf1.xml Sol Levante UHD HDR FR (en)]
          Segment 1 HICaptions track 464b7969: duration 262.000s differs from MainImage track
          9e470d1a: duration 263.083s - all virtual tracks in a segment shall have equal duration
  error   ST2067-3:2016:7.2.2/SegmentDuration
          [CPL:6005b7eb CPL_6005b7eb-ad96-4937-872a-62629fad4bf1.xml Sol Levante UHD HDR FR (en)]
          Segment 1 ForcedNarrative track 03bf1260: duration 262.000s differs from MainImage track
          9e470d1a: duration 263.083s - all virtual tracks in a segment shall have equal duration
  error   ST2067-3:2016:7.2.2/SegmentDuration
          [CPL:6005b7eb CPL_6005b7eb-ad96-4937-872a-62629fad4bf1.xml Sol Levante UHD HDR FR (en)]
          Segment 1 ForcedNarrative track 155a2768: duration 262.000s differs from MainImage track
          9e470d1a: duration 263.083s - all virtual tracks in a segment shall have equal duration
  warning IMFERNO:Package/UnlistedEssence File '.DS_Store' is present in the package directory
          but not listed in the AssetMap
  warning IMFERNO:Package/UnlistedEssence File 'CPL_b993cba5-d112-4b02-b504-28166bf30024_fix.xml'
          is present in the package directory but not listed in the AssetMap
  warning IMFERNO:Package/UnlistedEssence File 'CPL_6005b7eb-ad96-4937-872a-62629fad4bf1_fix.xml'
          is present in the package directory but not listed in the AssetMap
  info    ST2067-2:2016:8/DigitalSignature
          [CPL:b993cba5 CPL_b993cba5-d112-4b02-b504-28166bf30024.xml Sol Levante UHD SDR FR (en)]
          Digital signature validation (ST 2067-2 par. 8) is not currently performed
  info    ST2067-2:2016:8/DigitalSignature
          [CPL:6005b7eb CPL_6005b7eb-ad96-4937-872a-62629fad4bf1.xml Sol Levante UHD HDR FR (en)]
          Digital signature validation (ST 2067-2 par. 8) is not currently performed

failed 12 error(s), 3 warning(s)
```

Beide validators vinden hetzelfde probleem: 6288 vs 6314 frames.

## De fix

### Stap 1: TTML extraheren uit MXF

Alle 6 timed text MXF-bestanden bevatten IMSC1 timed text
(`http://www.w3.org/ns/ttml/profile/imsc1/text`), gewrapt in een MXF-container.

Met `as-02-unwrap` (asdcplib, gebouwd met xerces-c XML-ondersteuning) is de TTML
geextraheerd:

```
as-02-unwrap TimedText_4dd597ad.mxf /tmp/sol-ttml/sub1.xml
```

### Stap 2: MXF opnieuw wrappen met correcte duur

De TTML-bestanden zijn opnieuw gewrapt met `as-02-wrap`, waarbij de duur op 6314
frames is gezet (gelijk aan de video). De `-a` flag zorgt ervoor dat de AssetMap
UUID als MXF PackageUID wordt ingesteld:

```
as-02-wrap -d 6314 -a 4dd597ad-c38f-4526-8c91-8ed13bf0bdf2 \
  -r 24/1 -P "http://www.w3.org/ns/ttml/profile/imsc1/text" \
  sub1.xml TimedText_4dd597ad.mxf
```

Dit is herhaald voor alle 6 timed text bestanden.

### Stap 3: CPL's bijwerken

In beide CPL's zijn de volgende velden aangepast om overeen te komen met de nieuwe
MXF-bestanden:

- `IntrinsicDuration`: 6288 naar 6314
- `SourceDuration`: 6288 naar 6314
- `EssenceLength` in `DCTimedTextDescriptor`: 6288 naar 6314
- `ResourceID` in `DCTimedTextDescriptor`: aangepast naar de nieuwe MXF-waarden
- `InstanceID` in `DCTimedTextDescriptor`: aangepast naar de nieuwe MXF InstanceUID
- `RFC5646LanguageTagList`: verwijderd (as-02-wrap behoudt deze niet)

### Stap 4: PKL hashes herberekenen

Nieuwe SHA-1 hashes berekend voor alle 6 MXF-bestanden en beide CPL's, plus
bijgewerkte bestandsgroottes, en in de PKL bijgewerkt.

## Resultaat

### Photon output (na fix)

```
$ java -cp Photon-5.1.0-SNAPSHOT.jar com.netflix.imflibrary.app.IMPAnalyzer fixture/sol-fixed/

AUDIO_84557505-3805-43d7-a9fd-76fd76a6787c.mxf has no errors or warnings
VIDEO_SDR.mxf has no errors or warnings
AUDIO_02b1ad47-f8ec-49aa-8dcb-340a6c33f3e9.mxf has no errors or warnings
TimedText_c8ce2c82-1b5e-4b6d-a14d-b700f9f49730.mxf has no errors or warnings
TimedText_4dd597ad-c38f-4526-8c91-8ed13bf0bdf2.mxf has no errors or warnings
TimedText_75bb55d7-8d78-43fc-b447-e49c12445a4d.mxf has no errors or warnings
CPL_6005b7eb-ad96-4937-872a-62629fad4bf1.xml has no errors or warnings
TimedText_b0e3b943-dd4e-457e-b457-7d53165d6eaf.mxf has no errors or warnings
CPL_b993cba5-d112-4b02-b504-28166bf30024.xml has no errors or warnings
VIDEO_d15f7a99-666c-4be0-85ef-be11990e47c3.mxf has no errors or warnings
TimedText_de85a18b-accf-4585-b7c7-ed8b745e5480.mxf has no errors or warnings
PKL_d325c8e7-fe63-4345-9c11-9af4f667cd07.xml has no errors or warnings
TimedText_62418f3c-4f21-4b7c-8280-363cb2afdd65.mxf has no errors or warnings
```

0 fouten, 0 waarschuwingen.

### imferno output (na fix)

```
$ imferno validate --skip-hashes fixture/sol-fixed/

  ok  ASSETMAP.xml - 17 assets
  ok  2 CPL(s), 1 PKL(s)

CPL [b993cba5] Sol Levante UHD SDR FR (en)
  track  MainImage - 1920x1080 24fps JPEG 2000 (2K) ? SDR - f7879831
  track  MainAudio (ja) - 5.1 Surround 48.0kHz 24-bit - 02b1ad47
  track  MainAudio (ja) - Stereo 48.0kHz 24-bit - 84557505
  track  MainAudio (en) - Stereo 48.0kHz 24-bit - 2d05dacf
  track  MainAudio (en) - 5.1 Surround 48.0kHz 24-bit - 20506d44
  track  MainAudio (ja) - Stereo 48.0kHz 24-bit - 6edb0d2e
  track  MainAudio (en) - Stereo 48.0kHz 24-bit - 2aeeb0cd
  track  Subtitles - b0e3b943
  track  Subtitles - 4dd597ad
  track  HearingImpairedCaptions - c8ce2c82
  track  HearingImpairedCaptions - 62418f3c
  track  ForcedNarrative - de85a18b
  track  ForcedNarrative - 75bb55d7

CPL [6005b7eb] Sol Levante UHD HDR FR (en)
  track  MainImage - 3840x2160 24fps JPEG 2000 (4K) ? Dolby Vision - d15f7a99
  ...

Validation findings:
  info    ST2067-2:2016:8/DigitalSignature
          [CPL:b993cba5 CPL_b993cba5-d112-4b02-b504-28166bf30024.xml Sol Levante UHD SDR FR (en)]
          Digital signature validation (ST 2067-2 par. 8) is not currently performed
  info    ST2067-2:2016:8/DigitalSignature
          [CPL:6005b7eb CPL_6005b7eb-ad96-4937-872a-62629fad4bf1.xml Sol Levante UHD HDR FR (en)]
          Digital signature validation (ST 2067-2 par. 8) is not currently performed

valid
```

0 fouten, 0 waarschuwingen. Beide validators zijn volledig tevreden.

## Gebruikte tools

- **imferno** v2.0.0 (Rust)
- **Photon** v5.1.0-SNAPSHOT (Netflix, Java)
- **asdcplib** v2.13.3 (CineCert, C++) met xerces-c XML-ondersteuning
