use lingua::{Language, LanguageDetector, LanguageDetectorBuilder};

pub fn detect_language(text: &String) -> Vec<(String, f64)> {
    let detector: LanguageDetector = LanguageDetectorBuilder::from_all_languages().build();
    let confidence_values: Vec<(Language, f64)> = detector
        .compute_language_confidence_values(text);
    let output_map: Vec<(String, f64)> = confidence_values
        .into_iter()
        .map(|(language, confidence)| (language.to_string(), (confidence * 100.0).round() / 100.0))
        .collect();
    output_map
}
