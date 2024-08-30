// Copyright: (c) 2024, J. Zane Cook <z@agentartificial.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)

use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};

/// Will only check against supported languages.
pub fn detect_language(text: &String) -> std::result::Result<String, String> {
    let languages = vec![
        Language::English,
        Language::Polish,
        Language::French,
        Language::German,
        Language::Spanish,
        Language::Romanian,
        Language::Turkish,
        Language::Dutch,
        Language::Swedish,
        Language::Slovene,
        Language::Portuguese,
    ];
    let detector: LanguageDetector = LanguageDetectorBuilder::from_languages(&languages).build();
    match detector.detect_language_of(text) {
        Some(lang) => Ok(lang.to_string()),
        None => {
            let likely_language = find_likely_language(text);
            let error_text = format!("Unfortunately, I was unable to find the language of this text. It looks like you're trying to convert text from {}. This may not be supported currently. Use the `/supported_languages` command to see a list.", likely_language);
            Err(error_text)
        }
    }
}

pub fn detect_all_languages(text: &String) -> Vec<(String, f64)> {
    let detector: LanguageDetector = LanguageDetectorBuilder::from_all_languages().build();
    let confidence_values: Vec<(Language, f64)> = detector
        .compute_language_confidence_values(text);
    let output_map: Vec<(String, f64)> = confidence_values
        .into_iter()
        .map(|(language, confidence)| (language.to_string(), (confidence * 100.0).round() / 100.0))
        .collect();
    output_map
}

pub fn find_likely_language(text: &String) -> String {
    let detected_languages = detect_all_languages(text);
    let top_confidence = detected_languages.into_iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
    top_confidence.0
}
