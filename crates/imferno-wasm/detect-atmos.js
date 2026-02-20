#!/usr/bin/env node
/**
 * Demo: Detecting Dolby Atmos Audio Tracks in CPL
 *
 * Dolby Atmos in IMF is typically identified through:
 * 1. TrackId patterns (often contains "atmos", "IAB", or specific UL values)
 * 2. Resource metadata and essence descriptors
 * 3. Channel configuration (7.1.4, 5.1.4, etc.)
 */

import { parseCplTyped } from './index.js';
import { readFileSync } from 'fs';

// Sample CPL with Dolby Atmos track
const cplWithAtmos = `<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2016/cpl">
    <Id>urn:uuid:demo-cpl-with-atmos</Id>
    <IssueDate>2024-12-12T12:00:00Z</IssueDate>
    <ContentTitle>Demo with Atmos</ContentTitle>
    <SegmentList>
        <Segment>
            <Id>urn:uuid:segment-1</Id>
            <SequenceList>
                <!-- Standard 5.1 Audio Track -->
                <MainAudioSequence>
                    <Id>urn:uuid:audio-51</Id>
                    <TrackId>urn:uuid:track-audio-51</TrackId>
                    <ResourceList>
                        <Resource>
                            <Id>urn:uuid:resource-audio-51</Id>
                            <EditRate>24000 1001</EditRate>
                            <SourceEncoding>urn:smpte:ul:060e2b34.04010101.0e210201.02010000</SourceEncoding>
                        </Resource>
                    </ResourceList>
                </MainAudioSequence>

                <!-- Dolby Atmos IAB Track -->
                <MainAudioSequence>
                    <Id>urn:uuid:audio-atmos</Id>
                    <TrackId>urn:uuid:track-audio-iab</TrackId>
                    <ResourceList>
                        <Resource>
                            <Id>urn:uuid:resource-audio-atmos</Id>
                            <EditRate>24000 1001</EditRate>
                            <!-- IAB Essence Descriptor UL -->
                            <SourceEncoding>urn:smpte:ul:060e2b34.04010101.0e210201.03010000</SourceEncoding>
                            <Hash>abc123</Hash>
                        </Resource>
                    </ResourceList>
                </MainAudioSequence>

                <!-- Another potential Atmos track with naming convention -->
                <MainAudioSequence>
                    <Id>urn:uuid:audio-immersive</Id>
                    <TrackId>urn:smpte:ul:060e2b34.01020102.0e210201.03010000</TrackId>
                    <ResourceList>
                        <Resource>
                            <Id>urn:uuid:resource-immersive</Id>
                            <EditRate>24000 1001</EditRate>
                        </Resource>
                    </ResourceList>
                </MainAudioSequence>
            </SequenceList>
        </Segment>
    </SegmentList>
</CompositionPlaylist>`;

/**
 * Detect if an audio track is Dolby Atmos based on various indicators
 */
function isLikelyDolbyAtmos(audioSequence) {
    // Common Dolby Atmos indicators
    const atmosIndicators = {
        // Track ID patterns
        trackIdPatterns: [
            /iab/i,           // Immersive Audio Bitstream
            /atmos/i,         // Direct Atmos reference
            /immersive/i,     // Immersive audio
            /object[_-]?based/i, // Object-based audio
            /7\.1\.\d/,       // 7.1.x channel configurations
            /5\.1\.\d/,       // 5.1.x channel configurations with height
        ],

        // SMPTE UL patterns for IAB/Atmos
        sourceEncodingPatterns: [
            /0e210201\.0301/,     // IAB essence
            /0e210201\.03/,       // Object-based audio
            /0d010301\.0202/,     // Dolby specific
        ],

        // Resource ID patterns
        resourceIdPatterns: [
            /atmos/i,
            /iab/i,
            /dolby/i,
            /immersive/i
        ]
    };

    let score = 0;
    const reasons = [];

    // Check TrackId
    const trackId = audioSequence.trackId || '';
    for (const pattern of atmosIndicators.trackIdPatterns) {
        if (pattern.test(trackId)) {
            score += 3;
            reasons.push(`TrackId matches pattern: ${pattern}`);
        }
    }

    // Check resources
    if (audioSequence.resourceList?.resource) {
        for (const resource of audioSequence.resourceList.resource) {
            // Check SourceEncoding (UL)
            const sourceEncoding = resource.sourceEncoding || '';
            for (const pattern of atmosIndicators.sourceEncodingPatterns) {
                if (pattern.test(sourceEncoding)) {
                    score += 5; // Strong indicator
                    reasons.push(`SourceEncoding UL indicates IAB/Atmos: ${sourceEncoding}`);
                }
            }

            // Check Resource ID
            const resourceId = resource.id || '';
            for (const pattern of atmosIndicators.resourceIdPatterns) {
                if (pattern.test(resourceId)) {
                    score += 2;
                    reasons.push(`Resource ID suggests Atmos: ${resourceId}`);
                }
            }
        }
    }

    // Check sequence ID
    const sequenceId = audioSequence.id || '';
    for (const pattern of atmosIndicators.resourceIdPatterns) {
        if (pattern.test(sequenceId)) {
            score += 2;
            reasons.push(`Sequence ID suggests Atmos: ${sequenceId}`);
        }
    }

    return {
        isLikelyAtmos: score >= 3, // Threshold for detection
        confidence: score > 7 ? 'high' : score > 3 ? 'medium' : 'low',
        score,
        reasons
    };
}

/**
 * Find all Dolby Atmos tracks in a CPL
 */
async function findDolbyAtmosTracks(cplXml) {
    const cpl = await parseCplTyped(cplXml);
    const atmosTracks = [];

    console.log('🔍 Analyzing CPL for Dolby Atmos tracks...\n');

    // Check all segments
    if (cpl.segmentList?.segment) {
        for (const segment of cpl.segmentList.segment) {
            // Check MainAudioSequences
            if (segment.sequenceList?.mainAudioSequences) {
                segment.sequenceList.mainAudioSequences.forEach((audioSeq, index) => {
                    const analysis = isLikelyDolbyAtmos(audioSeq);

                    console.log(`Audio Track ${index + 1}:`);
                    console.log(`  ID: ${audioSeq.id}`);
                    console.log(`  TrackId: ${audioSeq.trackId}`);
                    console.log(`  Dolby Atmos: ${analysis.isLikelyAtmos ? '✅ YES' : '❌ NO'}`);
                    console.log(`  Confidence: ${analysis.confidence}`);

                    if (analysis.reasons.length > 0) {
                        console.log(`  Detection reasons:`);
                        analysis.reasons.forEach(reason => {
                            console.log(`    - ${reason}`);
                        });
                    }
                    console.log();

                    if (analysis.isLikelyAtmos) {
                        atmosTracks.push({
                            sequence: audioSeq,
                            analysis
                        });
                    }
                });
            }
        }
    }

    return atmosTracks;
}

// Run the demo
(async () => {
    console.log('🎬 Dolby Atmos Detection Demo');
    console.log('================================\n');

    try {
        const atmosTracks = await findDolbyAtmosTracks(cplWithAtmos);

        console.log('📊 Summary:');
        console.log(`   Total Atmos tracks found: ${atmosTracks.length}`);

        if (atmosTracks.length > 0) {
            console.log('\n🎯 Dolby Atmos Tracks:');
            atmosTracks.forEach((track, i) => {
                console.log(`   ${i + 1}. ${track.sequence.id}`);
                console.log(`      Confidence: ${track.analysis.confidence}`);
                console.log(`      Score: ${track.analysis.score}`);
            });
        }

        console.log('\n💡 Detection Methods:');
        console.log('   • Track ID patterns (IAB, Atmos, Immersive)');
        console.log('   • SMPTE UL SourceEncoding values');
        console.log('   • Resource and Sequence ID naming');
        console.log('   • Channel configuration patterns');

        console.log('\n📝 Note:');
        console.log('   For production use, you should also check:');
        console.log('   • MCA (Multi-Channel Audio) labels in MXF headers');
        console.log('   • Audio channel count (>8 channels often indicates Atmos)');
        console.log('   • Essence descriptor metadata from the actual MXF files');

    } catch (error) {
        console.error('❌ Error:', error.message);
    }
})();