// Example: Check if a CPL has specific language tracks
import { readFileSync } from 'fs';
import init, { parseCplTyped } from './pkg/imf_wasm.js';

// Sample CPL with multiple audio tracks (simplified for demonstration)
const cplWithLanguages = `<?xml version="1.0" encoding="UTF-8" standalone="no" ?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/2067-3/2016">
    <Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>
    <IssueDate>2024-12-12T12:00:00Z</IssueDate>
    <Issuer>Pathé Thuis</Issuer>
    <ContentTitle>
        <text>Sample Movie</text>
        <language>en</language>
    </ContentTitle>
    <LocaleList>
        <Locale>
            <LanguageList>
                <Language>en</Language>
                <Language>nl</Language>
                <Language>fr</Language>
            </LanguageList>
        </Locale>
    </LocaleList>
    <SegmentList>
        <Segment>
            <Id>urn:uuid:segment-001</Id>
            <SequenceList>
                <!-- Main Audio Track - English -->
                <MainAudioSequence>
                    <Id>urn:uuid:audio-en-001</Id>
                    <TrackId>urn:uuid:track-audio-en</TrackId>
                    <ResourceList>
                        <Resource>
                            <Id>urn:uuid:resource-audio-en</Id>
                            <Annotation>
                                <text>English Audio</text>
                                <language>en</language>
                            </Annotation>
                        </Resource>
                    </ResourceList>
                </MainAudioSequence>
                <!-- Main Audio Track - Dutch -->
                <MainAudioSequence>
                    <Id>urn:uuid:audio-nl-001</Id>
                    <TrackId>urn:uuid:track-audio-nl</TrackId>
                    <ResourceList>
                        <Resource>
                            <Id>urn:uuid:resource-audio-nl</Id>
                            <Annotation>
                                <text>Dutch Audio</text>
                                <language>nl</language>
                            </Annotation>
                        </Resource>
                    </ResourceList>
                </MainAudioSequence>
            </SequenceList>
        </Segment>
    </SegmentList>
</CompositionPlaylist>`;

/**
 * Check if a CPL contains specific language tracks
 * @param {string} cplXml - The CPL XML content
 * @param {string[]} requiredLanguages - Array of language codes to check (e.g., ['en', 'nl', 'fr'])
 * @returns {Object} Result with available languages and missing ones
 */
async function checkLanguageTracks(cplXml, requiredLanguages) {
    // Initialize WASM
    const wasmBuffer = readFileSync('./pkg/imf_wasm_bg.wasm');
    await init(wasmBuffer);

    // Parse the CPL
    const cpl = parseCplTyped(cplXml);

    const result = {
        requiredLanguages,
        availableLanguages: [],
        missingLanguages: [],
        languageDetails: {},
        hasAllRequired: false
    };

    // Method 1: Check LocaleList for declared languages
    if (cpl.LocaleList && cpl.LocaleList.Locale) {
        cpl.LocaleList.Locale.forEach(locale => {
            if (locale.LanguageList && locale.LanguageList.Language) {
                locale.LanguageList.Language.forEach(lang => {
                    if (!result.availableLanguages.includes(lang)) {
                        result.availableLanguages.push(lang);
                        result.languageDetails[lang] = {
                            declaredInLocale: true,
                            audioTracks: [],
                            subtitleTracks: []
                        };
                    }
                });
            }
        });
    }

    // Method 2: Check actual audio/subtitle sequences in segments
    if (cpl.SegmentList && cpl.SegmentList.Segment) {
        cpl.SegmentList.Segment.forEach(segment => {
            if (segment.SequenceList) {
                const sequences = segment.SequenceList;

                // Check for audio tracks with language info
                // Note: The actual structure depends on your CPL format
                // This is a simplified example

                // In real CPLs, you'd need to:
                // 1. Parse MainAudioSequence for audio tracks
                // 2. Parse SubtitlesSequence for subtitle tracks
                // 3. Extract language from Resource annotations or metadata
            }
        });
    }

    // Method 3: Extract from ContentTitle language
    if (cpl.ContentTitle && cpl.ContentTitle.language) {
        const contentLang = cpl.ContentTitle.language;
        if (!result.availableLanguages.includes(contentLang)) {
            result.availableLanguages.push(contentLang);
        }
    }

    // Check which required languages are missing
    result.missingLanguages = requiredLanguages.filter(
        lang => !result.availableLanguages.includes(lang)
    );

    result.hasAllRequired = result.missingLanguages.length === 0;

    return result;
}

/**
 * Helper function to check language availability using the simpler API
 * This uses the existing checkLanguageAvailability function from the WASM module
 */
async function checkLanguagesSimple(cplXml, requiredLanguages) {
    // You can also use the built-in checkLanguageAvailability function
    // if your WASM module has analyzed the features

    const wasmBuffer = readFileSync('./pkg/imf_wasm_bg.wasm');
    await init(wasmBuffer);

    // First parse the CPL
    const cpl = parseCplTyped(cplXml);

    // Then use the built-in language checker
    // Note: This would require the analyzeImfFeatures to be called first
    // const features = analyzeImfFeatures(JSON.stringify({ cpl }));
    // const result = checkLanguageAvailability(features, JSON.stringify(requiredLanguages));

    return cpl;
}

// Example usage
async function demonstrateLanguageChecking() {
    console.log('🎬 IMF Language Track Checker\n');
    console.log('=' .repeat(50));

    try {
        // Check for specific languages
        const requiredLanguages = ['en', 'nl', 'fr', 'de'];
        console.log(`\n📋 Checking for required languages: ${requiredLanguages.join(', ')}`);

        const result = await checkLanguageTracks(cplWithLanguages, requiredLanguages);

        console.log(`\n✅ Available languages: ${result.availableLanguages.join(', ')}`);

        if (result.missingLanguages.length > 0) {
            console.log(`❌ Missing languages: ${result.missingLanguages.join(', ')}`);
        }

        console.log(`\n📊 Language Check Result:`);
        console.log(`   Has all required: ${result.hasAllRequired ? '✅ YES' : '❌ NO'}`);
        console.log(`   Available: ${result.availableLanguages.length}/${requiredLanguages.length}`);

        // Practical use cases
        console.log('\n🎯 Practical Use Cases:');
        console.log('   1. Validate deliverables have required language tracks');
        console.log('   2. Check subtitle availability for specific markets');
        console.log('   3. Ensure audio tracks match distribution requirements');
        console.log('   4. Generate language availability reports');

        // Example business logic
        console.log('\n💼 Business Logic Example:');
        if (result.availableLanguages.includes('nl')) {
            console.log('   ✅ Dutch audio available - ready for Netherlands release');
        }

        if (result.availableLanguages.includes('en') && result.availableLanguages.includes('fr')) {
            console.log('   ✅ English and French available - ready for Canadian release');
        }

        if (!result.availableLanguages.includes('de')) {
            console.log('   ⚠️  German missing - not ready for DACH region');
        }

    } catch (error) {
        console.error('❌ Error checking languages:', error);
    }
}

// Run the demonstration
if (import.meta.url === `file://${process.argv[1]}`) {
    demonstrateLanguageChecking().catch(console.error);
}