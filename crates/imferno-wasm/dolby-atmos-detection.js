#!/usr/bin/env node
/**
 * Dolby Atmos Detection in IMF CPL Files
 *
 * Dolby Atmos tracks in IMF are identified by:
 * 1. IAB (Immersive Audio Bitstream) essence encoding
 * 2. Track naming conventions
 * 3. Channel configurations beyond standard 5.1/7.1
 */

import { parseCplTyped, getVersion } from './index.js';

console.log('🎬 Dolby Atmos Detection in IMF CPL');
console.log('====================================\n');

/**
 * Detection patterns for Dolby Atmos
 */
const ATMOS_PATTERNS = {
    // SMPTE UL patterns for IAB/Immersive Audio
    iabULPatterns: [
        /060e2b34\.04010101\.0e210201\.0301/,  // IAB essence
        /060e2b34\.04010101\.0e210201\.03/,     // Object-based audio prefix
        /060e2b34\.0401010d\.04020201\.0e/,     // Dolby specific patterns
    ],

    // Track ID naming patterns
    trackNamePatterns: [
        /iab/i,              // Immersive Audio Bitstream
        /atmos/i,            // Direct Atmos reference
        /immersive/i,        // Immersive audio
        /object[_-]?based/i, // Object-based audio
        /dolby/i,            // Dolby references
    ],

    // Channel configuration patterns (e.g., 7.1.4, 5.1.2)
    channelPatterns: [
        /7\.1\.[24]/,        // 7.1.2 or 7.1.4 (with height speakers)
        /5\.1\.[24]/,        // 5.1.2 or 5.1.4 (with height speakers)
        /9\.1/,              // Extended surround configurations
    ]
};

/**
 * Analyze an audio sequence for Dolby Atmos indicators
 */
function analyzeAudioTrack(audioSequence) {
    const indicators = [];
    let confidence = 0;

    // Check Track ID
    if (audioSequence.trackId) {
        for (const pattern of ATMOS_PATTERNS.trackNamePatterns) {
            if (pattern.test(audioSequence.trackId)) {
                indicators.push(`Track ID contains Atmos indicator: ${audioSequence.trackId}`);
                confidence += 30;
                break;
            }
        }
    }

    // Check Sequence ID
    if (audioSequence.id) {
        for (const pattern of ATMOS_PATTERNS.trackNamePatterns) {
            if (pattern.test(audioSequence.id)) {
                indicators.push(`Sequence ID suggests Atmos: ${audioSequence.id}`);
                confidence += 20;
                break;
            }
        }
    }

    // Check Resources for SourceEncoding (SMPTE UL)
    if (audioSequence.resourceList?.resource) {
        for (const resource of audioSequence.resourceList.resource) {
            if (resource.sourceEncoding) {
                // Remove urn:smpte:ul: prefix for pattern matching
                const cleanUL = resource.sourceEncoding.replace('urn:smpte:ul:', '');

                for (const pattern of ATMOS_PATTERNS.iabULPatterns) {
                    if (pattern.test(cleanUL)) {
                        indicators.push(`IAB/Immersive Audio UL detected: ${resource.sourceEncoding}`);
                        confidence += 50; // Strong indicator
                        break;
                    }
                }
            }

            // Check resource ID
            if (resource.id) {
                for (const pattern of ATMOS_PATTERNS.trackNamePatterns) {
                    if (pattern.test(resource.id)) {
                        indicators.push(`Resource ID contains Atmos reference: ${resource.id}`);
                        confidence += 15;
                        break;
                    }
                }
            }
        }
    }

    return {
        isAtmos: confidence >= 30,
        confidence: Math.min(confidence, 100),
        indicators
    };
}

/**
 * Find all Dolby Atmos tracks in a CPL
 */
export async function findDolbyAtmosTracks(cplXml) {
    const cpl = await parseCplTyped(cplXml);
    const results = {
        hasAtmos: false,
        atmosTracks: [],
        standardTracks: [],
        summary: {}
    };

    // Process all segments
    if (cpl.segmentList?.segment) {
        for (const segment of cpl.segmentList.segment) {
            // Check mainAudioSequences (camelCase from WASM)
            if (segment.sequenceList?.mainAudioSequences) {
                for (const audioSeq of segment.sequenceList.mainAudioSequences) {
                    const analysis = analyzeAudioTrack(audioSeq);

                    const trackInfo = {
                        id: audioSeq.id,
                        trackId: audioSeq.trackId,
                        isAtmos: analysis.isAtmos,
                        confidence: analysis.confidence,
                        indicators: analysis.indicators
                    };

                    if (analysis.isAtmos) {
                        results.atmosTracks.push(trackInfo);
                        results.hasAtmos = true;
                    } else {
                        results.standardTracks.push(trackInfo);
                    }
                }
            }
        }
    }

    // Create summary
    results.summary = {
        totalAudioTracks: results.atmosTracks.length + results.standardTracks.length,
        atmosTracks: results.atmosTracks.length,
        standardTracks: results.standardTracks.length,
        hasImmersiveAudio: results.hasAtmos
    };

    return results;
}

// Demo with example CPL
const demoCPL = `<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2016">
    <Id>urn:uuid:demo-cpl-with-audio</Id>
    <ContentTitle>Movie with Multiple Audio Tracks</ContentTitle>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <SegmentList>
        <Segment>
            <Id>urn:uuid:segment-1</Id>
            <SequenceList>
                <!-- Standard 5.1 Surround -->
                <MainAudioSequence>
                    <Id>urn:uuid:audio-51-main</Id>
                    <TrackId>urn:uuid:track-51-surround</TrackId>
                    <ResourceList>
                        <Resource>
                            <Id>urn:uuid:resource-51</Id>
                            <EditRate>24000 1001</EditRate>
                            <SourceEncoding>urn:smpte:ul:060e2b34.04010101.0e210201.02010000</SourceEncoding>
                        </Resource>
                    </ResourceList>
                </MainAudioSequence>

                <!-- Dolby Atmos IAB Track -->
                <MainAudioSequence>
                    <Id>urn:uuid:audio-dolby-atmos-iab</Id>
                    <TrackId>urn:uuid:track-iab-immersive</TrackId>
                    <ResourceList>
                        <Resource>
                            <Id>urn:uuid:resource-atmos</Id>
                            <EditRate>24000 1001</EditRate>
                            <!-- IAB Essence UL -->
                            <SourceEncoding>urn:smpte:ul:060e2b34.04010101.0e210201.03010000</SourceEncoding>
                        </Resource>
                    </ResourceList>
                </MainAudioSequence>

                <!-- Stereo Track -->
                <MainAudioSequence>
                    <Id>urn:uuid:audio-stereo</Id>
                    <TrackId>urn:uuid:track-stereo-20</TrackId>
                    <ResourceList>
                        <Resource>
                            <Id>urn:uuid:resource-stereo</Id>
                            <EditRate>48000 1</EditRate>
                        </Resource>
                    </ResourceList>
                </MainAudioSequence>
            </SequenceList>
        </Segment>
    </SegmentList>
</CompositionPlaylist>`;

// Run the demo
(async () => {
    console.log(`📦 IMF Parser Version: ${await getVersion()}\n`);

    try {
        const analysis = await findDolbyAtmosTracks(demoCPL);

        console.log('📊 Audio Track Analysis:');
        console.log(`   Total Audio Tracks: ${analysis.summary.totalAudioTracks}`);
        console.log(`   Dolby Atmos Tracks: ${analysis.summary.atmosTracks}`);
        console.log(`   Standard Tracks: ${analysis.summary.standardTracks}`);
        console.log(`   Has Immersive Audio: ${analysis.summary.hasImmersiveAudio ? '✅ YES' : '❌ NO'}\n`);

        if (analysis.atmosTracks.length > 0) {
            console.log('🎯 Dolby Atmos Tracks Detected:');
            analysis.atmosTracks.forEach((track, i) => {
                console.log(`\n   Track ${i + 1}:`);
                console.log(`   - ID: ${track.id}`);
                console.log(`   - Track ID: ${track.trackId}`);
                console.log(`   - Confidence: ${track.confidence}%`);
                console.log(`   - Detection indicators:`);
                track.indicators.forEach(indicator => {
                    console.log(`     • ${indicator}`);
                });
            });
        }

        if (analysis.standardTracks.length > 0) {
            console.log('\n📻 Standard Audio Tracks:');
            analysis.standardTracks.forEach((track, i) => {
                console.log(`   ${i + 1}. ${track.id} (${track.trackId})`);
            });
        }

        console.log('\n💡 Detection Methods Used:');
        console.log('   • SMPTE UL SourceEncoding for IAB essence');
        console.log('   • Track ID naming patterns');
        console.log('   • Resource metadata analysis');
        console.log('   • Sequence ID patterns');

        console.log('\n📝 Notes for Production:');
        console.log('   • Check MXF headers for MCA labels');
        console.log('   • Verify channel count (>8 typically indicates Atmos)');
        console.log('   • Look for ADM (Audio Definition Model) metadata');
        console.log('   • Validate against Dolby specifications');

    } catch (error) {
        console.error('❌ Error:', error.message);
    }
})();