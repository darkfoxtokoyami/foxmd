use std::cmp::Ordering;

use super::*;

#[test]
fn pre_tokenize_italics() {
    let a = pre_tokenize("[i]");
    assert_eq!(a[0].cmp(&"[i]"), Ordering::Equal, "a: [i] == [i]");

    let b = pre_tokenize("The quick brown fox jumped over the [i]lazy[/i] dog");

    assert_eq!(b[1].cmp(&"[i]"), Ordering::Equal, "b: [i] == [i]");
    assert_eq!(b[3].cmp(&"[/i]"), Ordering::Equal, "b: [/i] == [/i]");

    let c = pre_tokenize("[ i ]");
    assert_eq!(c[0].cmp(&"[ i ]"), Ordering::Equal, "a: [ i ] == [ i ]");
}

#[test]
fn pre_tokenize_bold() {
    let a = pre_tokenize("[b]");
    assert_eq!(a[0].cmp(&"[b]"), Ordering::Equal, "a: [b] == [b]");

    let b = pre_tokenize("The quick brown fox jumped over the [b]lazy[/b] dog");

    assert_eq!(b[1].cmp(&"[b]"), Ordering::Equal, "b: [b] == [b]");
    assert_eq!(b[3].cmp(&"[/b]"), Ordering::Equal, "b: [/b] == [/b]");

    let c = pre_tokenize("[ b ]");
    assert_eq!(c[0].cmp(&"[ b ]"), Ordering::Equal, "a: [ b ] == [ b ]");
}

#[test]
fn pre_tokenize_underline() {
    let a = pre_tokenize("[u]");
    assert_eq!(a[0].cmp(&"[u]"), Ordering::Equal, "a: [u] == [u]");

    let b = pre_tokenize("The quick brown fox jumped over the [u]lazy[/u] dog");

    assert_eq!(b[1].cmp(&"[u]"), Ordering::Equal, "b: [u] == [u]");
    assert_eq!(b[3].cmp(&"[/u]"), Ordering::Equal, "b: [/u] == [/u]");

    let c = pre_tokenize("[ u ]");
    assert_eq!(c[0].cmp(&"[ u ]"), Ordering::Equal, "a: [ u ] == [ u ]");
}

#[test]
fn pre_tokenize_strikethrough() {
    let a = pre_tokenize("[s]");
    assert_eq!(a[0].cmp(&"[s]"), Ordering::Equal, "a: [s] == [s]");

    let b = pre_tokenize("The quick brown fox jumped over the [s]lazy[/s] dog");

    assert_eq!(b[1].cmp(&"[s]"), Ordering::Equal, "b: [s] == [s]");
    assert_eq!(b[3].cmp(&"[/s]"), Ordering::Equal, "b: [/s] == [/s]");

    let c = pre_tokenize("[ s ]");
    assert_eq!(c[0].cmp(&"[ s ]"), Ordering::Equal, "a: [ s ] == [ s ]");
}

#[test]
fn pre_tokenize_superscript() {
    let a = pre_tokenize("[sup]");
    assert_eq!(a[0].cmp(&"[sup]"), Ordering::Equal, "a: [sup] == [sup]");

    let b = pre_tokenize("The quick brown fox jumped over the [sup]lazy[/sup] dog");

    assert_eq!(b[1].cmp(&"[sup]"), Ordering::Equal, "b: [sup] == [sup]");
    assert_eq!(b[3].cmp(&"[/sup]"), Ordering::Equal, "b: [/sup] == [/sup]");

    let c = pre_tokenize("[ sup ]");
    assert_eq!(
        c[0].cmp(&"[ sup ]"),
        Ordering::Equal,
        "a: [ sup ] == [ sup ]"
    );
}

#[test]
fn pre_tokenize_subscript() {
    let a = pre_tokenize("[sub]");
    assert_eq!(a[0].cmp(&"[sub]"), Ordering::Equal, "a: [sub] == [sub]");

    let b = pre_tokenize("The quick brown fox jumped over the [sub]lazy[/sub] dog");

    assert_eq!(b[1].cmp(&"[sub]"), Ordering::Equal, "b: [sub] == [sub]");
    assert_eq!(b[3].cmp(&"[/sub]"), Ordering::Equal, "b: [/sub] == [/sub]");

    let c = pre_tokenize("[ sub ]");
    assert_eq!(
        c[0].cmp(&"[ sub ]"),
        Ordering::Equal,
        "a: [ sub ] == [ sub ]"
    );
}
