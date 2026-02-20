#!/usr/bin/env node

import { parseCplTyped } from './index.js';

// Use a real CPL structure with proper namespaces
const cplXml = `<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2016">
    <Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
    <ContentTitle>Test Movie with Atmos</ContentTitle>
    <SegmentList>
        <Segment>
            <Id>urn:uuid:segment-001</Id>
            <SequenceList>
                <cc:MainAudioSequence xmlns:cc="http://www.smpte-ra.org/schemas/2067-3/2016">
                    <cc:Id>urn:uuid:audio-51-track</cc:Id>
                    <cc:TrackId>urn:uuid:51-surround</cc:TrackId>
                    <cc:ResourceList>
                        <cc:Resource>
                            <cc:Id>urn:uuid:resource-51</cc:Id>
                            <cc:EditRate>24000 1001</cc:EditRate>
                        </cc:Resource>
                    </cc:ResourceList>
                </cc:MainAudioSequence>
                <cc:MainAudioSequence xmlns:cc="http://www.smpte-ra.org/schemas/2067-3/2016">
                    <cc:Id>urn:uuid:audio-atmos-iab</cc:Id>
                    <cc:TrackId>urn:uuid:dolby-atmos-iab-track</cc:TrackId>
                    <cc:ResourceList>
                        <cc:Resource>
                            <cc:Id>urn:uuid:resource-atmos</cc:Id>
                            <cc:EditRate>24000 1001</cc:EditRate>
                            <cc:SourceEncoding>urn:smpte:ul:060e2b34.04010101.0e210201.03010000</cc:SourceEncoding>
                        </cc:Resource>
                    </cc:ResourceList>
                </cc:MainAudioSequence>
            </SequenceList>
        </Segment>
    </SegmentList>
</CompositionPlaylist>`;

(async () => {
    try {
        const cpl = await parseCplTyped(cplXml);
        console.log('Parsed CPL structure:');
        console.log(JSON.stringify(cpl, null, 2));
    } catch (error) {
        console.error('Error:', error);
    }
})();