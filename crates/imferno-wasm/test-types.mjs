import { readFileSync } from 'fs';
import init, { parseVolindexTyped, parseAssetmapTyped, parseCplTyped } from './pkg/imf_wasm.js';

// Test data
const volindexXml = `<?xml version="1.0" encoding="UTF-8" standalone="no" ?><VolumeIndex xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM"><Index>1</Index></VolumeIndex>`;

const assetmapXml = `<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<AssetMap xmlns="http://www.smpte-ra.org/schemas/429-9/2007/AM">
    <Id>urn:uuid:75864667-c65e-4aae-a5b2-fa5ea5fe31b7</Id>
    <AnnotationText>MERIDIAN</AnnotationText>
    <Creator>Clipster 6.1.0.0 Beta</Creator>
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
    </AssetList>
</AssetMap>`;

async function test() {
    // Initialize WASM with explicit file path for Node.js
    const wasmBuffer = readFileSync('./pkg/imf_wasm_bg.wasm');
    await init(wasmBuffer);

    console.log('Testing parseVolindexTyped...');
    try {
        const volindex = parseVolindexTyped(volindexXml);
        console.log('✅ VolumeIndex parsed:', volindex);
        console.log('  index value:', volindex.index);
        console.log('  Type check: index is', typeof volindex.index);
    } catch (e) {
        console.error('❌ Failed:', e);
    }

    console.log('\nTesting parseAssetmapTyped...');
    try {
        const assetmap = parseAssetmapTyped(assetmapXml);
        console.log('✅ AssetMap parsed:', assetmap);
        console.log('  id:', assetmap.id);
        console.log('  annotationText:', assetmap.annotationText);
        console.log('  volumeCount:', assetmap.volumeCount);
        console.log('  Assets count:', assetmap.assetList.asset.length);
        console.log('  First asset path:', assetmap.assetList.asset[0].chunkList.chunk[0].path);
    } catch (e) {
        console.error('❌ Failed:', e);
    }
}

test().catch(console.error);