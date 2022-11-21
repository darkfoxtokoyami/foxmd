use std::cmp::Ordering;

use super::*;

fn pre_tokenize_test(test_input: &str, expected_output: &str, token_idx: usize) {
    let mut fmd = FMD::new();
    let a = expected_output;
    fmd = fmd.pre_tokenize(test_input);
    let b = &fmd._tokens[token_idx];
    println!("Tokens: {}", b);
    assert_eq!(
        a.cmp(b),
        Ordering::Equal,
        "Expected: a, {} == test_input, {}; found: b, {}\nTokens: {:?}",
        a,
        test_input,
        b,
        fmd._tokens
    );
}
fn pre_tokenize_test_with_data(tag: &str) {
    pre_tokenize_test(
        format!("[{}]", tag).as_str(),
        format!("[{}]", tag).as_str(),
        0,
    );
    pre_tokenize_test(
        format!(
            "The quick brown fox jumped over the [{}]lazy[/{}] dog",
            tag, tag
        )
        .as_str(),
        format!("[{}]", tag).as_str(),
        1,
    );
    pre_tokenize_test(
        format!(
            "The quick brown fox jumped over the [{}]lazy[/{}] dog",
            tag, tag
        )
        .as_str(),
        format!("[/{}]", tag).as_str(),
        3,
    );
    pre_tokenize_test(
        format!(
            "The quick brown fox jumped over the [ {} ]lazy[ /{} ] dog",
            tag, tag
        )
        .as_str(),
        format!("[ {} ]", tag).as_str(),
        1,
    );
    pre_tokenize_test(
        format!(
            "The quick brown fox jumped over the [ {} ]lazy[ /{} ] dog",
            tag, tag
        )
        .as_str(),
        format!("[ /{} ]", tag).as_str(),
        3,
    );
    pre_tokenize_test(
        format!(
            "The quick brown fox jumped over the [ {} ]lazy[ /  {} ] dog",
            tag, tag
        )
        .as_str(),
        format!("[ /  {} ]", tag).as_str(),
        3,
    );
}
#[test]
fn run_pre_tokenize_test_with_tags() {
    pre_tokenize_test_with_data("i");
    pre_tokenize_test_with_data("b");
    pre_tokenize_test_with_data("u");
    pre_tokenize_test_with_data("s");
    pre_tokenize_test_with_data("sup");
    pre_tokenize_test_with_data("sub");
    //pre_tokenize_test_with_data("color");     // Need a separate test for [tag="string"] Otherwise it won't match regex properly
    //pre_tokenize_test_with_data("definition");
}
