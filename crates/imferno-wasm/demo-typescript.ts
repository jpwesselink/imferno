#!/usr/bin/env ts-node
/**
 * TypeScript Demo: Auto-Initializing IMF Parser
 *
 * Shows full type safety with zero initialization boilerplate
 */

import { parseAssetmapTyped, parseCplTyped, AssetMap, CompositionPlaylist } from './index.js';

async function demonstrateTypeSafety() {
    console.log('🎯 TypeScript Demo: Full Type Safety + Auto-Init');
    console.log('===============================================\n');

    const assetMapXml = `<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
    <AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
        <Id>urn:uuid:typescript-demo</Id>
        <AnnotationText>TypeScript Demo</AnnotationText>
        <Creator>imf-js</Creator>
        <VolumeCount>1</VolumeCount>
        <IssueDate>2024-12-12T12:00:00Z</IssueDate>
        <Issuer>TypeScript</Issuer>
        <AssetList>
            <Asset>
                <Id>urn:uuid:asset-001</Id>
                <ChunkList>
                    <Chunk>
                        <Path>demo.mxf</Path>
                        <VolumeIndex>1</VolumeIndex>
                    </Chunk>
                </ChunkList>
            </Asset>
        </AssetList>
    </AssetMap>`;

    try {
        // Auto-init + full IntelliSense + compile-time type checking
        const assetMap: AssetMap = await parseAssetmapTyped(assetMapXml);

        console.log('✅ Parsed with Full Type Safety');
        console.log(`   Asset Map ID: ${assetMap.Id}`);
        console.log(`   Creator: ${assetMap.Creator}`);
        console.log(`   Assets: ${assetMap.AssetList.Asset.length}`);

        // Demonstrate type-safe object navigation
        const firstAsset = assetMap.AssetList.Asset[0];
        console.log(`   First Asset: ${firstAsset.Id}`);
        console.log(`   First Chunk: ${firstAsset.ChunkList.Chunk[0].Path}`);

        console.log('\n🎯 TypeScript Benefits:');
        console.log('   ✅ Full IntelliSense autocomplete');
        console.log('   ✅ Compile-time type checking');
        console.log('   ✅ No manual type casting');
        console.log('   ✅ Zero runtime type errors');
        console.log('   ✅ Auto-completion shows object structure');

    } catch (error: any) {
        console.error('❌ Parse error:', error.message);
    }
}

// Run the demo
demonstrateTypeSafety().catch(console.error);