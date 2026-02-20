#!/usr/bin/env node
/**
 * Demo: Auto-Initializing IMF Parser
 *
 * No more manual WASM initialization!
 * No more init(wasmBuffer) calls!
 * Just import and use!
 */

import { parseAssetmapTyped, getVersion } from './index.js';

console.log('🎬 IMF Parser - Auto-Init Demo');
console.log('==============================');
console.log('✨ No manual initialization required!');
console.log('✨ No WASM buffer loading!');
console.log('✨ Just import and use!\n');

// Sample AssetMap
const assetMapXml = `<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
    <Id>urn:uuid:75864667-c65e-4aae-a5b2-fa5ea5fe31b7</Id>
    <AnnotationText>Auto-Init Demo</AnnotationText>
    <Creator>IMF Parser</Creator>
    <VolumeCount>1</VolumeCount>
    <IssueDate>2024-12-12T12:00:00Z</IssueDate>
    <Issuer>imf-js</Issuer>
    <AssetList>
        <Asset>
            <Id>urn:uuid:demo-asset-001</Id>
            <ChunkList>
                <Chunk>
                    <Path>demo_content.xml</Path>
                    <VolumeIndex>1</VolumeIndex>
                </Chunk>
            </ChunkList>
        </Asset>
    </AssetList>
</AssetMap>`;

try {
    // This automatically initializes WASM on first call
    console.log(`📦 Version: ${await getVersion()}`);

    // Parse with full type safety - WASM already initialized!
    const assetMap = await parseAssetmapTyped(assetMapXml);

    console.log('\n✅ AssetMap Parsed Successfully (Auto-Init)');
    console.log(`   ID: ${assetMap.Id}`);
    console.log(`   Annotation: ${assetMap.AnnotationText}`);
    console.log(`   Creator: ${assetMap.Creator}`);

    console.log('\n🎯 Developer Experience:');
    console.log('   🟢 OLD: init(wasmBuffer); const result = parse(xml);');
    console.log('   🌟 NEW: const result = await parse(xml);');

    console.log('\n🚀 Benefits:');
    console.log('   ✅ Zero boilerplate initialization');
    console.log('   ✅ Automatic WASM loading');
    console.log('   ✅ Clean async/await API');
    console.log('   ✅ Full TypeScript support');
    console.log('   ✅ One-time initialization per process');

} catch (error) {
    console.error('❌ Parse error:', error.message);
}