use scout::common::{Text, TextBuilder};
use scout::fuzzy::*;

fn as_pool(subjects: &[&str]) -> Vec<Text> {
    subjects.iter().map(|s| TextBuilder::build(s)).collect()
}

fn perform_search(query: &str, cases: &[&str]) -> Vec<Candidate> {
    let pool = as_pool(cases);

    search(query, &pool, false)
}

fn assert_candidate(candidate: &Candidate, expected: &str) {
    let actual = format!("{}", candidate);

    assert_eq!(actual, expected)
}

fn assert_best_match(query: &str, cases: &[&str], expected: &str) {
    let results = perform_search(query, cases);

    assert_candidate(&results[0], expected);
}

#[test]
fn search_when_the_query_is_empty_test() {
    let cases = vec!["foo", "bar", "other"];

    let results = perform_search("", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[0]);
    assert_candidate(&results[1], cases[1]);
    assert_candidate(&results[2], cases[2]);
}

#[test]
fn search_when_there_are_no_results_test() {
    let cases = vec!["foo", "wta"];

    let results = perform_search("wat", &cases);

    assert!(results.is_empty());
}

#[test]
fn search_using_fancy_letters_test() {
    let cases = vec!["Markdown Preview: Copy Html", "Y̆xxxxxxx公xxxxxxxxxxxx🍣.js"];

    let results = perform_search("y̆公🍣", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
}

#[test]
fn search_exact_match_test() {
    let cases = vec!["filter", "Cargofile", "Nope"];

    let results = perform_search("file", &cases);

    assert!(!results.is_empty());
    assert_eq!(results.len(), 2);

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);
}

#[test]
fn search_exact_match_end_word_boundaries_test() {
    let table = vec![
        // End of world bonus (string limit)
        vec!["cargofile0", "0cargofile"],
        // End of world bonus (separator limit)
        vec!["hello cargofile0", "0cargofile world"],
        // End of world bonus (camelCase limit)
        vec!["helloCargofile0", "0cargofileWorld"],
    ];

    for cases in table {
        let results = perform_search("file", &cases);
        assert!(!results.is_empty());
        assert_candidate(&results[0], cases[1]);
        assert_candidate(&results[1], cases[0]);
    }
}

#[test]
fn search_exact_match_start_word_boundaries_test() {
    let table = vec![
        // Start of world bonus (string limit)
        vec!["0cargofile0", "cargofile0"],
        // Start of world bonus (separator limit)
        vec!["0cargofile world", "hello cargofile0"],
        // Start of world bonus (camelCase limit)
        vec!["0cargofileWorld", "helloCargofile0"],
    ];

    for cases in table {
        let results = perform_search("cargo", &cases);
        assert!(!results.is_empty());
        assert_candidate(&results[0], cases[1]);
        assert_candidate(&results[1], cases[0]);
    }
}

#[test]
fn search_exact_match_preference_test() {
    let cases = vec![
        "controller x",
        "0_co_re_00 x",
        "0core0_000 x",
        "0core_0000 x",
        "0_core0_00 x",
        "0_core_000 x",
    ];

    let results = perform_search("core", &cases);

    assert!(!results.is_empty());

    // full-word > start-of-word > end-of-word > middle-of-word > split > scattered letters
    assert_candidate(&results[0], cases[5]);
    assert_candidate(&results[1], cases[4]);
    assert_candidate(&results[2], cases[3]);
    assert_candidate(&results[3], cases[2]);
    assert_candidate(&results[4], cases[1]);
    assert_candidate(&results[5], cases[0]);

    // multi word search
    let results = perform_search("core x", &cases);

    assert!(!results.is_empty());

    // full-word > start-of-word > end-of-word > middle-of-word > split > scattered letters
    assert_candidate(&results[0], cases[5]);
    assert_candidate(&results[1], cases[4]);
    assert_candidate(&results[2], cases[3]);
    assert_candidate(&results[3], cases[2]);
    assert_candidate(&results[4], cases[1]);
    assert_candidate(&results[5], cases[0]);
}

#[test]
fn search_exact_match_case_insensitive_over_complete_word_test() {
    let cases = vec!["fil e", "ZFILEZ"];

    let results = perform_search("file", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);
}

#[test]
fn search_exact_match_prefers_smaller_haystack_test() {
    let cases = vec!["core_", "core"];

    let results = perform_search("core", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);
}

#[test]
fn search_exact_match_prefers_match_at_start_of_string_test() {
    let cases = vec!["data_core", "core_data"];

    let results = perform_search("core", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);

    let cases = vec!["hello_data_core", "hello_core_data"];

    let results = perform_search("core", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);
}

#[test]
fn search_exact_match_prefers_single_letter_start_of_world_test() {
    let cases = vec!["Timecop: View", "Markdown Preview: Copy Html"];

    let results = perform_search("m", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);

    let cases = vec!["Welcome: Show", "Markdown Preview: Toggle Break On Newline"];

    let results = perform_search("m", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);

    let cases = vec!["TODO", "doc/README"];

    let results = perform_search("d", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);
}

#[test]
fn search_exact_match_selects_better_occurences_test() {
    let cases = vec!["Portugues", "Test Español"];

    let results = perform_search("es", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);
}

#[test]
fn search_consecutive_letters_preference_test() {
    let cases = vec![
        "model-controller.x",
        "model-0core0-000.x",
        "model-0core-0000.x",
        "model-0-core0-00.x",
        "model-0-core-000.x",
    ];

    let results = perform_search("modelcore", &cases);

    assert!(!results.is_empty());

    // full-word > start-of-word > end-of-word > middle-of-word > scattered letters
    assert_candidate(&results[0], cases[4]);
    assert_candidate(&results[1], cases[3]);
    assert_candidate(&results[2], cases[2]);
    assert_candidate(&results[3], cases[1]);
    assert_candidate(&results[4], cases[0]);

    // multi word search
    let results = perform_search("modelcorex", &cases);

    assert!(!results.is_empty());

    // full-word > start-of-word > end-of-word > middle-of-word > scattered letters
    assert_candidate(&results[0], cases[4]);
    assert_candidate(&results[1], cases[3]);
    assert_candidate(&results[2], cases[2]);
    assert_candidate(&results[3], cases[1]);
    assert_candidate(&results[4], cases[0]);
}

#[test]
fn search_consecutive_letters_preference_vs_directory_depth_test() {
    let cases = vec![
        "model/controller.x",
        "0/model/0core0_0.x",
        "0/0/model/0core_00.x",
        "0/0/0/model/core0_00.x",
        "0/0/0/0/model/core_000.x",
    ];

    let results = perform_search("model core", &cases);

    assert!(!results.is_empty());

    // full-word > start-of-word > end-of-word > middle-of-word > scattered letters
    assert_candidate(&results[0], cases[4]);
    assert_candidate(&results[1], cases[3]);
    assert_candidate(&results[2], cases[2]);
    assert_candidate(&results[3], cases[1]);
    assert_candidate(&results[4], cases[0]);

    // multi word search
    let results = perform_search("model core x", &cases);

    assert!(!results.is_empty());

    // full-word > start-of-word > end-of-word > middle-of-word > scattered letters
    assert_candidate(&results[0], cases[4]);
    assert_candidate(&results[1], cases[3]);
    assert_candidate(&results[2], cases[2]);
    assert_candidate(&results[3], cases[1]);
    assert_candidate(&results[4], cases[0]);
}

#[test]
fn search_consecutive_letters_vs_scattered_test() {
    let cases = vec!["application.rb", "application_controller.rb"];

    let results = perform_search("acon", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);
}

#[test]
fn search_prefers_larger_groups_of_consecutive_letters_test() {
    let cases = vec!["ab cd ef", " abc def", " abcd ef", " abcde f", "  abcdef"];

    let results = perform_search("abcdef", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[4]);
    assert_candidate(&results[1], cases[3]);
    assert_candidate(&results[2], cases[2]);
    assert_candidate(&results[3], cases[1]);
    assert_candidate(&results[4], cases[0]);
}

#[test]
fn search_prefers_larger_groups_of_consecutive_letters_vs_better_context_test() {
    let cases = vec!["ab cd ef", "0abc0def0"];

    let results = perform_search("abcdef", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);

    let cases = vec!["ab cd ef", "0abcd0ef0"];

    let results = perform_search("abcdef", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);
}

#[test]
fn search_consecutive_letters_in_path_overcome_deeper_path_test() {
    let cases = vec!["controller/app.rb", "controller/core/app.rb"];

    let results = perform_search("core app", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);
}

#[test]
fn search_consecutive_matches_weigh_higher_at_start_of_word_or_base_name_test() {
    let cases = vec!["a_b_c", "a_b"];

    let results = perform_search("ab", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);

    let cases = vec!["z_a_b", "a_b"];

    let results = perform_search("ab", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);

    let cases = vec!["c_a_b", "a_b_c"];

    let results = perform_search("ab", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[1]);
    assert_candidate(&results[1], cases[0]);
}

#[test]
fn search_weighs_exact_case_matches_higher_test() {
    let cases = vec!["statusurl", "StatusUrl"];

    assert_best_match("Status", &cases, cases[1]);
    assert_best_match("StatusUrl", &cases, cases[1]);
    assert_best_match("status", &cases, cases[0]);
    assert_best_match("statusurl", &cases, cases[0]);

    let cases = vec!["Diagnostic", "diagnostics0000"];
    assert_best_match("diag", &cases, cases[1]);
    assert_best_match("diago", &cases, cases[1]);

    let cases = vec!["download_thread", "DownloadTask"];
    assert_best_match("down", &cases, cases[0]);
    assert_best_match("downt", &cases, cases[0]);
    assert_best_match("downta", &cases, cases[1]);
    assert_best_match("dt", &cases, cases[0]);
    assert_best_match("DT", &cases, cases[1]);
}

#[test]
fn search_accounts_for_case_while_selecting_an_acronym() {
    let cases = vec!["statusurl", "status_url", "StatusUrl"];

    assert_best_match("SU", &cases, cases[2]);
    assert_best_match("su", &cases, cases[1]);
    assert_best_match("st", &cases, cases[0]);
}

#[test]
fn search_weighs_case_sensitive_machtes_vs_directory_depth_test() {
    let cases = vec!["0/Diagnostic", "0/0/0/diagnostics00"];

    assert_best_match("diag", &cases, cases[1]);
    assert_best_match("diago", &cases, cases[1]);
}

#[test]
fn search_weighs_abbreviation_matches_after_spaces_underscores_and_dashes_the_same_test() {
    let cases = vec!["sub-zero", "sub zero", "sub_zero"];
    assert_best_match("sz", &cases, cases[0]);

    let cases = vec!["sub zero", "sub_zero", "sub-zero"];
    assert_best_match("sz", &cases, cases[0]);

    let cases = vec!["sub_zero", "sub-zero", "sub zero"];
    assert_best_match("sz", &cases, cases[0]);
}

#[test]
fn search_weighs_acronyms_higher_than_middle_word_letter_test() {
    let cases = vec!["FilterFactor.html", "FilterFactorTests.html"];

    assert_best_match("FFT", &cases, cases[1]);
}

#[test]
fn search_prefers_longer_acronym_to_a_smaller_case_sensitive_one_test() {
    let cases = vec![
        "efficient",
        "fun fact",
        "FileFactory",
        "FilterFactorTests.html",
    ];

    assert_best_match("fft", &cases, cases[3]);
    assert_best_match("ff", &cases, cases[1]);
    assert_best_match("FF", &cases, cases[2]);
}

#[test]
fn search_weighs_acronyms_higher_than_middle_word_exact_match_test() {
    let cases = vec!["switch.css", "ImportanceTableCtrl.js"];

    assert_best_match("itc", &cases, cases[1]);
    assert_best_match("ITC", &cases, cases[1]);
}

#[test]
fn search_allows_to_select_between_snake_case_and_camel_case_using_case_of_query_test() {
    let cases = vec![
        "switch.css",
        "user_id_to_client.rb",
        "ImportanceTableCtrl.js",
    ];

    assert_best_match("itc", &cases, cases[1]);
    assert_best_match("ITC", &cases, cases[2]);
}

#[test]
fn search_prefers_camel_case_that_happens_sooner_test() {
    let cases = vec!["anotherCamelCase", "thisCamelCase000"];

    assert_best_match("CC", &cases, cases[1]);
    assert_best_match("CCs", &cases, cases[1]);
}

#[test]
fn search_prefers_camel_case_in_shorter_haystacks_test() {
    let cases = vec!["CamelCase0", "CamelCase"];

    assert_best_match("CC", &cases, cases[1]);
    assert_best_match("CCs", &cases, cases[1]);
}

#[test]
fn search_allows_camel_case_to_match_across_words_test() {
    let cases = vec!["Gallas", "Git Plus: Add All"];

    assert_best_match("gaa", &cases, cases[1]);
}

#[test]
fn search_allows_camel_case_to_match_even_outside_of_acronym_prefix_test() {
    let cases = vec![
        "Git Plus: Stash Save",
        "Git Plus: Add And Commit",
        "Git Plus: Add All",
    ];

    let results = perform_search("git AA", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[2]);
    assert_candidate(&results[1], cases[1]);
    assert_candidate(&results[2], cases[0]);

    let results = perform_search("git aa", &cases);

    assert!(!results.is_empty());

    assert_candidate(&results[0], cases[2]);
    assert_candidate(&results[1], cases[1]);
    assert_candidate(&results[2], cases[0]);
}

#[test]
fn search_accounts_for_match_structure_in_camel_case_vs_substring_matches_test() {
    let cases = vec!["Input: Select All", "Application: Install"];

    assert_best_match("install", &cases, cases[1]);
    assert_best_match("isa", &cases, cases[0]);
    assert_best_match("isall", &cases, cases[0]);

    let cases = vec!["Git Plus: Stage Hunk", "Git Plus: Push"];

    assert_best_match("push", &cases, cases[1]);
    assert_best_match("git push", &cases, cases[1]);
    assert_best_match("psh", &cases, cases[0]);
}

#[test]
fn search_accounts_for_case_in_camel_case_vs_substring_matches_test() {
    let cases = vec!["CamelCaseClass.js", "cccManagerUI.java"];

    // extact acronym
    assert_best_match("CCC", &cases, cases[0]);
    assert_best_match("ccc", &cases, cases[1]);

    // general purpose
    assert_best_match("CCCa", &cases, cases[0]);
    assert_best_match("ccca", &cases, cases[1]);
}

#[test]
fn search_prefers_acronym_matches_that_correspond_to_the_full_candidate_acronym_test() {
    let cases = vec!["JaVaScript", "JavaScript"];

    assert_best_match("js", &cases, cases[1]);

    let cases = vec!["JSON", "J.S.O.N.", "JavaScript"];

    assert_best_match("js", &cases, cases[2]);

    let cases = vec!["CSON", "C.S.O.N.", "CoffeeScript"];

    assert_best_match("cs", &cases, cases[2]);
}
