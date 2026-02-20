// TypeScript example demonstrating IMF WASM parser with proper types
import { readFileSync } from 'fs';
import init, { parseVolindexTyped, parseAssetmapTyped, parseCplTyped } from './pkg/imf-types';
import type { VolumeIndex, AssetMap, CompositionPlaylist } from './pkg/imf-types';

// Sample SMPTE XML data
const volindexXml = `<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<VolumeIndex xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
    <Index>1</Index>
</VolumeIndex>`;

const assetmapXml = `<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
    <Id>urn:uuid:75864667-c65e-4aae-a5b2-fa5ea5fe31b7</Id>
    <AnnotationText>MERIDIAN Sample</AnnotationText>
    <Creator>IMF-RS TypeScript Example</Creator>
    <VolumeCount>1</VolumeCount>
    <IssueDate>2024-12-12T11:30:00Z</IssueDate>
    <Issuer>Pathé Thuis</Issuer>
    <AssetList>
        <Asset>
            <Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
            <ChunkList>
                <Chunk>
                    <Path>CPL_0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85.xml</Path>
                    <VolumeIndex>1</VolumeIndex>
                </Chunk>
            </ChunkList>
        </Asset>
    </AssetList>
</AssetMap>`;

async function demonstrateImfParsing(): Promise<void> {
    try {
        // Initialize WASM module
        console.log('🚀 Initializing IMF WASM parser...');
        const wasmBuffer = readFileSync('./pkg/imf_wasm_bg.wasm');
        await init(wasmBuffer);
        console.log('✅ WASM module initialized\n');

        // Parse VOLINDEX with full type safety
        console.log('📄 Parsing VOLINDEX.xml...');
        const volindex: VolumeIndex = parseVolindexTyped(volindexXml);
        console.log('✅ VolumeIndex parsed successfully');
        console.log(`   📝 Type information: VolumeIndex`);
        console.log(`   🔢 Index: ${volindex.Index} (NOTE: Currently PascalCase from XML)`);
        console.log(`   🏷️  Type check: index field is ${typeof volindex.Index}\n`);

        // Parse ASSETMAP with full type safety
        console.log('📄 Parsing ASSETMAP.xml...');
        const assetmap: AssetMap = parseAssetmapTyped(assetmapXml);
        console.log('✅ AssetMap parsed successfully');
        console.log(`   📝 Type information: AssetMap`);
        console.log(`   🆔 ID: ${assetmap.Id}`);
        console.log(`   📝 Annotation: ${assetmap.AnnotationText}`);
        console.log(`   👤 Creator: ${assetmap.Creator}`);
        console.log(`   📊 Volume Count: ${assetmap.VolumeCount}`);
        console.log(`   📅 Issue Date: ${assetmap.IssueDate}`);
        console.log(`   🏢 Issuer: ${assetmap.Issuer}`);
        console.log(`   📦 Assets: ${assetmap.AssetList.Asset.length} asset(s)`);

        // Access nested asset data with type safety
        const firstAsset = assetmap.AssetList.Asset[0];
        const firstChunk = firstAsset.ChunkList.Chunk[0];
        console.log(`   📁 First asset path: ${firstChunk.Path}`);
        console.log(`   💿 Volume index: ${firstChunk.VolumeIndex}\n`);

        // Demonstrate type safety benefits
        console.log('🛡️  TypeScript Benefits:');
        console.log('   ✅ Full IntelliSense support for all SMPTE fields');
        console.log('   ✅ Compile-time type checking');
        console.log('   ✅ Auto-completion in IDEs');
        console.log('   ✅ Prevents typos in field names');
        console.log('   ✅ Clear documentation of data structure');

        // Note about current limitations
        console.log('\n📝 Current Status:');
        console.log('   🟡 Runtime data uses PascalCase (SMPTE XML format)');
        console.log('   🟢 TypeScript definitions provide camelCase interfaces');
        console.log('   🔄 Both approaches ensure SMPTE specification compliance');

    } catch (error) {
        console.error('❌ Error during IMF parsing:', error);
        process.exit(1);
    }
}

// Export types for external use
export type { VolumeIndex, AssetMap, CompositionPlaylist };

// Run the demonstration
if (require.main === module) {
    demonstrateImfParsing().catch(console.error);
}