use assert_cmd::Command;
use predicates::prelude::*;

const KEY: &str = "6863652d6b61742d7374616e646172642d6b65792d33322d62797465732121";

fn hce() -> Command {
    Command::cargo_bin("hce").unwrap()
}

#[test]
fn encode_decode_sealed() {
    let input = "00112233445566778899aabbccddeeff";
    let encoded = hce()
        .arg("encode")
        .arg("-k")
        .arg(KEY)
        .arg("-m")
        .arg("sealed")
        .arg(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let encoded_str = String::from_utf8(encoded).unwrap().trim().to_string();

    hce()
        .arg("decode")
        .arg("-k")
        .arg(KEY)
        .arg("-m")
        .arg("sealed")
        .arg(&encoded_str)
        .assert()
        .success()
        .stdout(predicate::str::contains(input));
}

#[test]
fn encode_decode_plain() {
    let input = "00112233445566778899aabbccddeeff";
    let encoded = hce()
        .arg("encode")
        .arg("-m")
        .arg("plain")
        .arg(input)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let encoded_str = String::from_utf8(encoded).unwrap().trim().to_string();

    hce()
        .arg("decode")
        .arg("-m")
        .arg("plain")
        .arg(&encoded_str)
        .assert()
        .success()
        .stdout(predicate::str::contains(input));
}

#[test]
fn encode_with_bit_width() {
    hce()
        .arg("encode")
        .arg("-k")
        .arg(KEY)
        .arg("-w")
        .arg("64")
        .arg("deadbeefcafebabe")
        .assert()
        .success();
}

#[test]
fn encode_with_case() {
    let out = hce()
        .arg("encode")
        .arg("-k")
        .arg(KEY)
        .arg("--case")
        .arg("lower")
        .arg("deadbeefcafebabe")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(!s.chars().any(|c| c.is_uppercase() && c != '-'));
}

#[test]
fn encode_with_chunk_none() {
    let out = hce()
        .arg("encode")
        .arg("-k")
        .arg(KEY)
        .arg("--chunk-none")
        .arg("deadbeefcafebabe")
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let s = String::from_utf8(out).unwrap();
    assert!(!s.contains('-'));
}

#[test]
fn encode_with_cipher() {
    hce()
        .arg("encode")
        .arg("-k")
        .arg(KEY)
        .arg("--cipher")
        .arg("shuffle")
        .arg("deadbeefcafebabe")
        .assert()
        .success();
}

#[test]
fn encode_with_modulus() {
    hce()
        .arg("encode")
        .arg("-k")
        .arg(KEY)
        .arg("--modulus")
        .arg("1000000")
        .arg("2a")
        .assert()
        .success();
}

#[test]
fn decode_missing_key_fails() {
    hce()
        .arg("decode")
        .arg("-m")
        .arg("sealed")
        .arg("ANYTHING")
        .assert()
        .failure();
}

#[test]
fn invalid_hex_rejected() {
    hce().arg("encode").arg("zzz").assert().failure();
}

#[test]
fn kat_verification() {
    hce()
        .arg("kat")
        .arg("shared/kat")
        .arg("-k")
        .arg(KEY)
        .assert()
        .success()
        .stderr(predicate::str::contains("ALL"));
}

#[test]
fn kat_with_default_key() {
    hce()
        .arg("kat")
        .arg("shared/kat")
        .assert()
        .success()
        .stderr(predicate::str::contains("ALL"));
}

#[test]
fn encode_empty_input() {
    hce()
        .arg("encode")
        .arg("-k")
        .arg(KEY)
        .arg("")
        .assert()
        .failure();
}
