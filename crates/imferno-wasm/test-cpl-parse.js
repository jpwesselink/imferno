#!/usr/bin/env node

import { parseCplTyped } from './index.js';

const simpleCPL = `<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2016">
    <Id>urn:uuid:test-123</Id>
    <ContentTitle>Test</ContentTitle>
    <IssueDate>2024-01-01T00:00:00Z</IssueDate>
</CompositionPlaylist>`;

(async () => {
    try {
        const cpl = await parseCplTyped(simpleCPL);
        console.log('CPL parsed successfully!');
        console.log('Structure:', JSON.stringify(cpl, null, 2));
    } catch (error) {
        console.error('Parse error:', error.message);
        // Try with a more minimal CPL
        console.log('\nTrying minimal CPL...');
        const minimalCPL = `<?xml version="1.0"?><CompositionPlaylist><Id>test</Id></CompositionPlaylist>`;
        try {
            const cpl2 = await parseCplTyped(minimalCPL);
            console.log('Minimal CPL parsed:', JSON.stringify(cpl2, null, 2));
        } catch (e2) {
            console.error('Minimal parse error:', e2.message);
        }
    }
})();