// TypeScript example: Check if a CPL has specific language tracks with full type safety
import { readFileSync } from 'fs';
import init, { parseCplTyped } from './pkg/imf_wasm.js';
import type { CompositionPlaylist } from './pkg/imf_wasm.d.ts';

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

// Type definitions for language checking results
interface LanguageTrackDetails {
    declaredInLocale: boolean;
    audioTracks: string[];
    subtitleTracks: string[];
}

interface LanguageCheckResult {
    requiredLanguages: string[];
    availableLanguages: string[];
    missingLanguages: string[];
    languageDetails: Record<string, LanguageTrackDetails>;
    hasAllRequired: boolean;
}

/**
 * Check if a CPL contains specific language tracks with full type safety
 * @param cplXml - The CPL XML content
 * @param requiredLanguages - Array of language codes to check (e.g., ['en', 'nl', 'fr'])
 * @returns Promise<LanguageCheckResult> with detailed language analysis
 */
async function checkLanguageTracks(
    cplXml: string,
    requiredLanguages: string[]
): Promise<LanguageCheckResult> {
    // Initialize WASM
    const wasmBuffer = readFileSync('./pkg/imf_wasm_bg.wasm');
    await init(wasmBuffer);

    // Parse the CPL with full type safety
    const cpl: CompositionPlaylist = parseCplTyped(cplXml);

    const result: LanguageCheckResult = {
        requiredLanguages,
        availableLanguages: [],
        missingLanguages: [],
        languageDetails: {},
        hasAllRequired: false
    };

    // Method 1: Check localeList for declared languages with full type safety
    if (cpl.localeList && cpl.localeList.locale) {
        cpl.localeList.locale.forEach((locale) => {
            if (locale.languageList && locale.languageList.language) {
                locale.languageList.language.forEach((lang) => {
                    if (result.availableLanguages.indexOf(lang) === -1) {
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
    if (cpl.segmentList && cpl.segmentList.segment) {
        cpl.segmentList.segment.forEach((segment) => {
            if (segment.sequenceList) {
                // Check for audio tracks with language info
                // Note: The actual structure depends on your CPL format
                // This is a simplified example based on common SMPTE structures

                // In real CPLs, you'd need to:
                // 1. Parse MainAudioSequence for audio tracks
                // 2. Parse SubtitlesSequence for subtitle tracks
                // 3. Extract language from Resource annotations or metadata

                // Note: The WASM interface currently has limited SequenceList support
                // This would need to be expanded when more sequence types are available in WASM
                // For now, we rely on LocaleList and ContentTitle for language detection
            }
        });
    }

    // Method 3: Extract from contentTitle language with full type safety
    if (cpl.contentTitle && cpl.contentTitle.language) {
        const contentLang = cpl.contentTitle.language;
        if (result.availableLanguages.indexOf(contentLang) === -1) {
            result.availableLanguages.push(contentLang);
            if (!(contentLang in result.languageDetails)) {
                result.languageDetails[contentLang] = {
                    declaredInLocale: false,
                    audioTracks: [],
                    subtitleTracks: []
                };
            }
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
 * Business logic helper for market-specific validation
 */
class LanguageValidator {
    /**
     * Check if content is ready for Netherlands market
     */
    static isReadyForNetherlands(result: LanguageCheckResult): boolean {
        return result.availableLanguages.includes('nl');
    }

    /**
     * Check if content is ready for Canadian market (requires EN and FR)
     */
    static isReadyForCanada(result: LanguageCheckResult): boolean {
        return result.availableLanguages.includes('en') &&
               result.availableLanguages.includes('fr');
    }

    /**
     * Check if content is ready for DACH region (requires German)
     */
    static isReadyForDACH(result: LanguageCheckResult): boolean {
        return result.availableLanguages.includes('de');
    }

    /**
     * Generate market readiness report
     */
    static generateMarketReport(result: LanguageCheckResult): Record<string, boolean> {
        return {
            netherlands: this.isReadyForNetherlands(result),
            canada: this.isReadyForCanada(result),
            dach: this.isReadyForDACH(result),
            uk: result.availableLanguages.includes('en'),
            france: result.availableLanguages.includes('fr'),
            spain: result.availableLanguages.includes('es'),
            italy: result.availableLanguages.includes('it')
        };
    }
}

/**
 * Demonstration function with full type safety
 */
async function demonstrateLanguageChecking(): Promise<void> {
    console.log('🎬 IMF Language Track Checker (TypeScript)\n');
    let separator = '';
    for (let i = 0; i < 50; i++) {
        separator += '=';
    }
    console.log(separator);

    try {
        // Check for specific languages with type safety
        const requiredLanguages: string[] = ['en', 'nl', 'fr', 'de'];
        console.log(`\n📋 Checking for required languages: ${requiredLanguages.join(', ')}`);

        const result: LanguageCheckResult = await checkLanguageTracks(cplWithLanguages, requiredLanguages);

        console.log(`\n✅ Available languages: ${result.availableLanguages.join(', ')}`);

        if (result.missingLanguages.length > 0) {
            console.log(`❌ Missing languages: ${result.missingLanguages.join(', ')}`);
        }

        console.log(`\n📊 Language Check Result:`);
        console.log(`   Has all required: ${result.hasAllRequired ? '✅ YES' : '❌ NO'}`);
        console.log(`   Available: ${result.availableLanguages.length}/${requiredLanguages.length}`);

        // Language details with type safety
        console.log('\n🔍 Language Details:');
        for (const lang in result.languageDetails) {
            const details = result.languageDetails[lang];
            console.log(`   ${lang}: Declared=${details.declaredInLocale}, Audio tracks=${details.audioTracks.length}`);
        }

        // Business logic with type safety
        console.log('\n💼 Market Readiness Analysis:');
        const marketReport = LanguageValidator.generateMarketReport(result);

        for (const market in marketReport) {
            const ready = marketReport[market];
            const status = ready ? '✅' : '❌';
            console.log(`   ${market.toUpperCase()}: ${status}`);
        }

        // TypeScript benefits demonstration
        console.log('\n🛡️  TypeScript Benefits:');
        console.log('   ✅ Full IntelliSense support for all SMPTE fields');
        console.log('   ✅ Compile-time type checking');
        console.log('   ✅ Auto-completion prevents typos');
        console.log('   ✅ Clear interfaces for complex nested structures');
        console.log('   ✅ Type-safe business logic functions');

    } catch (error: unknown) {
        if (error instanceof Error) {
            console.error('❌ Error checking languages:', error.message);
        } else {
            console.error('❌ Unknown error:', error);
        }
    }
}

// Export types and functions
export type { LanguageCheckResult, LanguageTrackDetails };
export { checkLanguageTracks, LanguageValidator };

// Run the demonstration
if (typeof require !== 'undefined' && require.main === module) {
    demonstrateLanguageChecking().catch(console.error);
}