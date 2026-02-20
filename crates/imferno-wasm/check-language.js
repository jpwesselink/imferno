"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g = Object.create((typeof Iterator === "function" ? Iterator : Object).prototype);
    return g.next = verb(0), g["throw"] = verb(1), g["return"] = verb(2), typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.LanguageValidator = void 0;
exports.checkLanguageTracks = checkLanguageTracks;
// TypeScript example: Check if a CPL has specific language tracks with full type safety
var fs_1 = require("fs");
var imf_wasm_js_1 = require("./pkg/imf_wasm.js");
// Sample CPL with multiple audio tracks (simplified for demonstration)
var cplWithLanguages = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\" ?>\n<CompositionPlaylist xmlns=\"http://www.smpte-ra.org/schemas/2067-3/2016\">\n    <Id>urn:uuid:0eb3d1b9-b77b-4d3f-bbe5-7c69b15dca85</Id>\n    <IssueDate>2024-12-12T12:00:00Z</IssueDate>\n    <Issuer>Path\u00E9 Thuis</Issuer>\n    <ContentTitle>\n        <text>Sample Movie</text>\n        <language>en</language>\n    </ContentTitle>\n    <LocaleList>\n        <Locale>\n            <LanguageList>\n                <Language>en</Language>\n                <Language>nl</Language>\n                <Language>fr</Language>\n            </LanguageList>\n        </Locale>\n    </LocaleList>\n    <SegmentList>\n        <Segment>\n            <Id>urn:uuid:segment-001</Id>\n            <SequenceList>\n                <!-- Main Audio Track - English -->\n                <MainAudioSequence>\n                    <Id>urn:uuid:audio-en-001</Id>\n                    <TrackId>urn:uuid:track-audio-en</TrackId>\n                    <ResourceList>\n                        <Resource>\n                            <Id>urn:uuid:resource-audio-en</Id>\n                            <Annotation>\n                                <text>English Audio</text>\n                                <language>en</language>\n                            </Annotation>\n                        </Resource>\n                    </ResourceList>\n                </MainAudioSequence>\n                <!-- Main Audio Track - Dutch -->\n                <MainAudioSequence>\n                    <Id>urn:uuid:audio-nl-001</Id>\n                    <TrackId>urn:uuid:track-audio-nl</TrackId>\n                    <ResourceList>\n                        <Resource>\n                            <Id>urn:uuid:resource-audio-nl</Id>\n                            <Annotation>\n                                <text>Dutch Audio</text>\n                                <language>nl</language>\n                            </Annotation>\n                        </Resource>\n                    </ResourceList>\n                </MainAudioSequence>\n            </SequenceList>\n        </Segment>\n    </SegmentList>\n</CompositionPlaylist>";
/**
 * Check if a CPL contains specific language tracks with full type safety
 * @param cplXml - The CPL XML content
 * @param requiredLanguages - Array of language codes to check (e.g., ['en', 'nl', 'fr'])
 * @returns Promise<LanguageCheckResult> with detailed language analysis
 */
function checkLanguageTracks(cplXml, requiredLanguages) {
    return __awaiter(this, void 0, void 0, function () {
        var wasmBuffer, cpl, result, contentLang;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    wasmBuffer = (0, fs_1.readFileSync)('./pkg/imf_wasm_bg.wasm');
                    return [4 /*yield*/, (0, imf_wasm_js_1.default)(wasmBuffer)];
                case 1:
                    _a.sent();
                    cpl = (0, imf_wasm_js_1.parseCplTyped)(cplXml);
                    result = {
                        requiredLanguages: requiredLanguages,
                        availableLanguages: [],
                        missingLanguages: [],
                        languageDetails: {},
                        hasAllRequired: false
                    };
                    // Method 1: Check localeList for declared languages with full type safety
                    if (cpl.localeList && cpl.localeList.locale) {
                        cpl.localeList.locale.forEach(function (locale) {
                            if (locale.languageList && locale.languageList.language) {
                                locale.languageList.language.forEach(function (lang) {
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
                        cpl.segmentList.segment.forEach(function (segment) {
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
                        contentLang = cpl.contentTitle.language;
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
                    result.missingLanguages = requiredLanguages.filter(function (lang) { return !result.availableLanguages.includes(lang); });
                    result.hasAllRequired = result.missingLanguages.length === 0;
                    return [2 /*return*/, result];
            }
        });
    });
}
/**
 * Business logic helper for market-specific validation
 */
var LanguageValidator = /** @class */ (function () {
    function LanguageValidator() {
    }
    /**
     * Check if content is ready for Netherlands market
     */
    LanguageValidator.isReadyForNetherlands = function (result) {
        return result.availableLanguages.includes('nl');
    };
    /**
     * Check if content is ready for Canadian market (requires EN and FR)
     */
    LanguageValidator.isReadyForCanada = function (result) {
        return result.availableLanguages.includes('en') &&
            result.availableLanguages.includes('fr');
    };
    /**
     * Check if content is ready for DACH region (requires German)
     */
    LanguageValidator.isReadyForDACH = function (result) {
        return result.availableLanguages.includes('de');
    };
    /**
     * Generate market readiness report
     */
    LanguageValidator.generateMarketReport = function (result) {
        return {
            netherlands: this.isReadyForNetherlands(result),
            canada: this.isReadyForCanada(result),
            dach: this.isReadyForDACH(result),
            uk: result.availableLanguages.includes('en'),
            france: result.availableLanguages.includes('fr'),
            spain: result.availableLanguages.includes('es'),
            italy: result.availableLanguages.includes('it')
        };
    };
    return LanguageValidator;
}());
exports.LanguageValidator = LanguageValidator;
/**
 * Demonstration function with full type safety
 */
function demonstrateLanguageChecking() {
    return __awaiter(this, void 0, void 0, function () {
        var separator, i, requiredLanguages, result, lang, details, marketReport, market, ready, status_1, error_1;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    console.log('🎬 IMF Language Track Checker (TypeScript)\n');
                    separator = '';
                    for (i = 0; i < 50; i++) {
                        separator += '=';
                    }
                    console.log(separator);
                    _a.label = 1;
                case 1:
                    _a.trys.push([1, 3, , 4]);
                    requiredLanguages = ['en', 'nl', 'fr', 'de'];
                    console.log("\n\uD83D\uDCCB Checking for required languages: ".concat(requiredLanguages.join(', ')));
                    return [4 /*yield*/, checkLanguageTracks(cplWithLanguages, requiredLanguages)];
                case 2:
                    result = _a.sent();
                    console.log("\n\u2705 Available languages: ".concat(result.availableLanguages.join(', ')));
                    if (result.missingLanguages.length > 0) {
                        console.log("\u274C Missing languages: ".concat(result.missingLanguages.join(', ')));
                    }
                    console.log("\n\uD83D\uDCCA Language Check Result:");
                    console.log("   Has all required: ".concat(result.hasAllRequired ? '✅ YES' : '❌ NO'));
                    console.log("   Available: ".concat(result.availableLanguages.length, "/").concat(requiredLanguages.length));
                    // Language details with type safety
                    console.log('\n🔍 Language Details:');
                    for (lang in result.languageDetails) {
                        details = result.languageDetails[lang];
                        console.log("   ".concat(lang, ": Declared=").concat(details.declaredInLocale, ", Audio tracks=").concat(details.audioTracks.length));
                    }
                    // Business logic with type safety
                    console.log('\n💼 Market Readiness Analysis:');
                    marketReport = LanguageValidator.generateMarketReport(result);
                    for (market in marketReport) {
                        ready = marketReport[market];
                        status_1 = ready ? '✅' : '❌';
                        console.log("   ".concat(market.toUpperCase(), ": ").concat(status_1));
                    }
                    // TypeScript benefits demonstration
                    console.log('\n🛡️  TypeScript Benefits:');
                    console.log('   ✅ Full IntelliSense support for all SMPTE fields');
                    console.log('   ✅ Compile-time type checking');
                    console.log('   ✅ Auto-completion prevents typos');
                    console.log('   ✅ Clear interfaces for complex nested structures');
                    console.log('   ✅ Type-safe business logic functions');
                    return [3 /*break*/, 4];
                case 3:
                    error_1 = _a.sent();
                    if (error_1 instanceof Error) {
                        console.error('❌ Error checking languages:', error_1.message);
                    }
                    else {
                        console.error('❌ Unknown error:', error_1);
                    }
                    return [3 /*break*/, 4];
                case 4: return [2 /*return*/];
            }
        });
    });
}
// Run the demonstration
if (typeof require !== 'undefined' && require.main === module) {
    demonstrateLanguageChecking().catch(console.error);
}
