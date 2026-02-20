// Node.js example for IMF package analysis
// Run with: node node-example.js

import { readFileSync } from 'fs';
import { resolve } from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

// Initialize WASM module
const wasmModule = await import('../pkg/imf_wasm.js');
await wasmModule.default();

const {
    parse_imf_package_from_files,
    analyze_imf_features,
    check_language_availability,
    has_dolby_atmos,
    has_hdr10,
    get_version
} = wasmModule;

console.log(`🎬 IMF Package Analyzer v${get_version()}\n`);

/**
 * Load IMF package files from a directory
 */
function loadImfPackage(packageDir) {
    const files = {};

    try {
        // Load VOLINDEX.xml
        const volindexPath = resolve(packageDir, 'VOLINDEX.xml');
        files['VOLINDEX.xml'] = readFileSync(volindexPath, 'utf8');
        console.log('✅ Loaded VOLINDEX.xml');

        // Load ASSETMAP.xml
        const assetmapPath = resolve(packageDir, 'ASSETMAP.xml');
        files['ASSETMAP.xml'] = readFileSync(assetmapPath, 'utf8');
        console.log('✅ Loaded ASSETMAP.xml');

        // Find and load CPL files
        const fs = await import('fs');
        const cplFiles = fs.readdirSync(packageDir)
            .filter(name => name.startsWith('CPL_') && name.endsWith('.xml'));

        for (const cplFile of cplFiles) {
            const cplPath = resolve(packageDir, cplFile);
            files[cplFile] = readFileSync(cplPath, 'utf8');
            console.log(`✅ Loaded ${cplFile}`);
        }

        if (cplFiles.length === 0) {
            throw new Error('No CPL files found');
        }

        return files;

    } catch (err) {
        console.error('❌ Error loading package:', err.message);
        process.exit(1);
    }
}

/**
 * Analyze an IMF package
 */
async function analyzePackage(packageDir) {
    console.log(`📁 Loading IMF package from: ${packageDir}\n`);

    // Step 1: Load files
    const files = loadImfPackage(packageDir);

    // Step 2: Parse IMF package (now includes comprehensive feature analysis)
    console.log('\n🔍 Parsing IMF package...');
    const parseResult = parse_imf_package_from_files(JSON.stringify(files));

    if (!parseResult.success) {
        console.error('❌ Parse failed:', parseResult.error);
        process.exit(1);
    }

    const packageData = JSON.parse(parseResult.data);
    console.log(`✅ Parsed successfully: ${packageData.cpl_count} CPLs, ${packageData.asset_count} assets`);

    // Features are now automatically included in the parse result
    const features = packageData.features || {};
    if (Object.keys(features).length > 0) {
        console.log('✅ Comprehensive feature analysis complete');
        console.log(`   • Audio formats: ${features.audio_formats?.length || 0}`);
        console.log(`   • Video codecs: ${features.video_codecs?.length || 0}`);
        console.log(`   • Languages: ${features.audio_languages?.length || 0}`);
    } else {
        console.log('⚠️  No features detected');
    }

    // Step 4: Display results
    displayResults(features);

    // Step 5: Check requirements
    await checkRequirements(features);
}

/**
 * Display analysis results
 */
function displayResults(features) {
    console.log('\n' + '='.repeat(60));
    console.log('📊 ANALYSIS RESULTS');
    console.log('='.repeat(60));

    // Package metadata
    console.log('\n📋 Package Information:');
    console.log(`  • Title: ${features.metadata.title || 'Unknown'}`);
    console.log(`  • Content Kind: ${features.metadata.content_kind || 'Unknown'}`);
    console.log(`  • CPL Count: ${features.metadata.cpl_count}`);

    // Video specifications
    console.log('\n🎬 Video Specifications:');
    const specs = features.video_specs;
    if (specs.resolution) {
        console.log(`  • Resolution: ${specs.resolution} (${specs.resolution_category || 'Unknown'})`);
    }
    if (specs.frame_rate) {
        console.log(`  • Frame Rate: ${specs.frame_rate_category || specs.frame_rate}`);
    }
    if (specs.color_space) {
        console.log(`  • Color Space: ${specs.color_space}`);
    }
    if (specs.hdr_format) {
        console.log(`  • HDR Format: ${specs.hdr_format}`);
    }
    if (specs.bit_depth) {
        console.log(`  • Bit Depth: ${specs.bit_depth}-bit`);
    }
    if (specs.stereoscopic_3d) {
        console.log(`  • 3D: ${specs.stereoscopic_3d}`);
    }

    // Audio formats
    console.log('\n🔊 Audio Formats:');
    const audioFormatNames = {
        'DolbyAtmos': 'Dolby Atmos (Immersive)',
        'Pcm': 'PCM (Uncompressed)',
        'DtsX': 'DTS:X (Immersive)',
        'DolbyDigital': 'Dolby Digital (AC-3)',
        'DolbyDigitalPlus': 'Dolby Digital Plus (E-AC-3)',
        'DolbyTrueHD': 'Dolby TrueHD (Lossless)',
        'DtsHdMa': 'DTS-HD Master Audio (Lossless)'
    };

    if (features.audio_formats.length === 0) {
        console.log('  • No audio formats detected');
    } else {
        features.audio_formats.forEach(format => {
            console.log(`  ✅ ${audioFormatNames[format] || format}`);
        });
    }

    // Audio languages
    console.log('\n🌐 Audio Languages:');
    if (features.audio_languages.length === 0) {
        console.log('  • No languages detected');
    } else {
        features.audio_languages.forEach(lang => {
            console.log(`  ✅ ${lang}`);
        });
    }

    // Video codecs
    console.log('\n📹 Video Codecs:');
    if (features.video_codecs.length === 0) {
        console.log('  • No video codecs detected');
    } else {
        features.video_codecs.forEach(codec => {
            console.log(`  ✅ ${codec}`);
        });
    }

    // Subtitle formats
    console.log('\n📝 Subtitle Formats:');
    if (features.subtitle_formats.length === 0) {
        console.log('  • No subtitle formats detected');
    } else {
        features.subtitle_formats.forEach(format => {
            console.log(`  ✅ ${format}`);
        });
    }

    // Summary
    console.log('\n📈 Summary:');
    console.log(`  • Audio Formats: ${features.audio_formats.length}`);
    console.log(`  • Audio Languages: ${features.audio_languages.length}`);
    console.log(`  • Video Codecs: ${features.video_codecs.length}`);
    console.log(`  • Subtitle Formats: ${features.subtitle_formats.length}`);
}

/**
 * Check streaming platform requirements
 */
async function checkRequirements(features) {
    console.log('\n' + '='.repeat(60));
    console.log('📋 STREAMING PLATFORM REQUIREMENTS');
    console.log('='.repeat(60));

    try {
        // Language requirements
        const requiredLanguages = ['en', 'en-US', 'fr', 'de', 'es'];
        const langCheck = check_language_availability(
            JSON.stringify(features),
            JSON.stringify(requiredLanguages)
        );

        if (langCheck.success) {
            const langResults = JSON.parse(langCheck.data);
            console.log('\n🌐 Language Availability:');
            langResults.forEach(result => {
                const icon = result.available ? '✅' : '❌';
                console.log(`  ${icon} ${result.language}`);
            });
        }

        // Dolby Atmos check
        const atmosCheck = has_dolby_atmos(JSON.stringify(features));
        if (atmosCheck.success) {
            const atmosResult = JSON.parse(atmosCheck.data);
            const icon = atmosResult.has_dolby_atmos ? '✅' : '❌';
            console.log(`\n🔊 Immersive Audio:\n  ${icon} Dolby Atmos`);
        }

        // HDR10 check
        const hdrCheck = has_hdr10(JSON.stringify(features));
        if (hdrCheck.success) {
            const hdrResult = JSON.parse(hdrCheck.data);
            const icon = hdrResult.has_hdr10 ? '✅' : '❌';
            console.log(`\n🎨 HDR Support:\n  ${icon} HDR10`);
            if (hdrResult.hdr_format) {
                console.log(`  • Format: ${hdrResult.hdr_format}`);
            }
        }

        // Netflix/Prime/Disney+ typical requirements
        console.log('\n🎯 Typical Platform Requirements:');
        const hasEnglish = features.audio_languages.some(lang =>
            lang === 'en' || lang === 'en-US'
        );
        const hasAtmos = features.audio_formats.includes('DolbyAtmos');
        const hasHDR = features.video_specs.hdr_format;
        const is4K = features.video_specs.resolution_category &&
            features.video_specs.resolution_category.includes('4K');

        console.log(`  ${hasEnglish ? '✅' : '❌'} English audio`);
        console.log(`  ${hasAtmos ? '✅' : '❌'} Dolby Atmos`);
        console.log(`  ${hasHDR ? '✅' : '❌'} HDR support`);
        console.log(`  ${is4K ? '✅' : '❌'} 4K resolution`);

    } catch (err) {
        console.error('❌ Requirements check failed:', err.message);
    }
}

// Main execution
const packageDir = process.argv[2];

if (!packageDir) {
    console.error('Usage: node node-example.js <imf-package-directory>');
    console.error('Example: node node-example.js ./test-data/IAB/CompleteIMP');
    process.exit(1);
}

await analyzePackage(packageDir);