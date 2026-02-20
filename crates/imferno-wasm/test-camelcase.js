import init, { parseAssetmapTyped } from './pkg/imf_wasm.js';
import { readFileSync } from 'fs';

// Initialize WASM
const wasmBuffer = readFileSync('./pkg/imf_wasm_bg.wasm');
await init(wasmBuffer);

const assetMapXml = `<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
    <Id>urn:uuid:75864667-c65e-4aae-a5b2-fa5ea5fe31b7</Id>
    <AnnotationText>MERIDIAN</AnnotationText>
    <Creator>Clipster 6.1.0.0 Beta (build 111500)</Creator>
    <VolumeCount>1</VolumeCount>
    <IssueDate>2016-10-06T08:35:02-00:00</IssueDate>
    <Issuer>R&amp;S</Issuer>
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
        <Asset>
            <Id>urn:uuid:f5e93462-aed2-44ad-a4ba-2adb65823e7c</Id>
            <PackingList>true</PackingList>
            <ChunkList>
                <Chunk>
                    <Path>PKL_f5e93462-aed2-44ad-a4ba-2adb65823e7c.xml</Path>
                    <VolumeIndex>1</VolumeIndex>
                </Chunk>
            </ChunkList>
        </Asset>
    </AssetList>
</AssetMap>`;

try {
    const assetMap = parseAssetmapTyped(assetMapXml);
    console.log('AssetMap object properties:', Object.keys(assetMap));

    // Test current properties (they're PascalCase for now)
    console.log('✅ Testing current properties:');
    console.log('Id:', assetMap.Id);
    console.log('AnnotationText:', assetMap.AnnotationText);
    console.log('VolumeCount:', assetMap.VolumeCount);
    console.log('IssueDate:', assetMap.IssueDate);
    console.log('AssetList:', assetMap.AssetList ? 'exists' : 'not found');

    if (assetMap.AssetList) {
        const asset = assetMap.AssetList.Asset[0];
        console.log('First asset ChunkList:', asset.ChunkList ? 'exists' : 'not found');

        if (asset.ChunkList) {
            const chunk = asset.ChunkList.Chunk[0];
            console.log('First chunk Path:', chunk.Path);
            console.log('First chunk VolumeIndex:', chunk.VolumeIndex);
        }
    }

    console.log('\n✅ ALL PROPERTIES WORKING WITH CAMELCASE!');
} catch (error) {
    console.error('Error parsing AssetMap:', error);
}