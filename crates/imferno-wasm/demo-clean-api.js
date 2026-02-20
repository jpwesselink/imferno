#!/usr/bin/env node
/**
 * Demo: Clean IMF WASM API - Option A Implementation
 *
 * This demonstrates our streamlined public interface with only the essential,
 * well-designed functions that developers actually want to use.
 */

import init, {
    getVersion,
    parseVolindexTyped,
    parseAssetmapTyped,
    parseCplTyped
} from './pkg/imf_wasm.js';
import { readFileSync } from 'fs';

// Initialize WASM
const wasmBuffer = readFileSync('./pkg/imf_wasm_bg.wasm');
await init(wasmBuffer);

console.log('🎬 IMF WASM Parser - Clean API Demo');
console.log('==========================================');
console.log(`📦 Version: ${getVersion()}`);
console.log(`🛡️  Type Safety: Full TypeScript support`);
console.log(`🧹 Clean API: Only essential functions exposed\n`);

// Sample AssetMap
const assetMapXml = `<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
    <Id>urn:uuid:75864667-c65e-4aae-a5b2-fa5ea5fe31b7</Id>
    <AnnotationText>Clean API Demo</AnnotationText>
    <Creator>IMF WASM Parser</Creator>
    <VolumeCount>1</VolumeCount>
    <IssueDate>2024-12-12T12:00:00Z</IssueDate>
    <Issuer>Pathé Thuis</Issuer>
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
    // Parse with full type safety - no casting needed!
    const assetMap = parseAssetmapTyped(assetMapXml);

    console.log('✅ AssetMap Parsed Successfully');
    console.log(`   ID: ${assetMap.Id}`);
    console.log(`   Annotation: ${assetMap.AnnotationText}`);
    console.log(`   Creator: ${assetMap.Creator}`);
    console.log(`   Assets: ${assetMap.AssetList.Asset.length}`);

    // Demonstrate nested object access with full IntelliSense
    const firstAsset = assetMap.AssetList.Asset[0];
    const firstChunk = firstAsset.ChunkList.Chunk[0];

    console.log('\n🔍 Nested Object Access (Full Type Safety):');
    console.log(`   First Asset ID: ${firstAsset.Id}`);
    console.log(`   First Chunk Path: ${firstChunk.Path}`);
    console.log(`   Volume Index: ${firstChunk.VolumeIndex}`);

    console.log('\n🎯 Benefits of Clean API:');
    console.log('   ✅ No JSON string parsing required');
    console.log('   ✅ No manual type casting needed');
    console.log('   ✅ Full IntelliSense in your IDE');
    console.log('   ✅ Compile-time error checking');
    console.log('   ✅ Native JavaScript object structure');
    console.log('   ✅ Zero dependencies on result wrapper types');

    console.log('\n📊 Developer Experience:');
    console.log('   🟢 Simple: parseAssetmapTyped(xml) → AssetMap');
    console.log('   🔴 NOT: parseAssetmap(xml) → ImfResult → JSON.parse()');

} catch (error) {
    console.error('❌ Parse error:', error.message);
}

console.log('\n🚀 Ready for Production:');
console.log('   • Clean, minimal public API');
console.log('   • Industry-standard TypeScript interfaces');
console.log('   • Zero cruft - only what developers need');
console.log('   • Future-proof architecture');