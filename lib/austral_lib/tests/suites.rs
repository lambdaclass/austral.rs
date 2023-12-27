use austral_lib::lexer::{lex, Token};
use pretty_assertions::assert_eq;
use std::{fs, path::Path};
use test_case::test_case;

#[test_case("programs/suites/001-trivial/001-null-program")]
// #[test_case("programs/suites/001-trivial/002-embed")]
// #[test_case("programs/suites/001-trivial/003-sizeof")]
// #[test_case("programs/suites/001-trivial/004-for-loop")]
// #[test_case("programs/suites/001-trivial/005-hello-world")]
// #[test_case("programs/suites/001-trivial/006-printable")]
// #[test_case("programs/suites/001-trivial/007-root-cap")]
// #[test_case("programs/suites/001-trivial/008-abort")]
// #[test_case("programs/suites/001-trivial/009-docstrings")]
// #[test_case("programs/suites/001-trivial/010-cli")]
// #[test_case("programs/suites/001-trivial/011-integer-conversions")]
// #[test_case("programs/suites/001-trivial/012-else-if")]
// #[test_case("programs/suites/001-trivial/013-simple-assignment")]
// #[test_case("programs/suites/001-trivial/014-named-records")]
// #[test_case("programs/suites/001-trivial/015-float-conversions")]
fn suite(path: impl AsRef<Path>) {
    let base_path = Path::new("../..").join(path);

    dbg!(&base_path);
    if base_path.join("Test.aui").exists() {
        compare(base_path.join("Test.aui"), base_path.join("aui-syntax.ron"));
    }

    compare(base_path.join("Test.aum"), base_path.join("aum-syntax.ron"));
}

#[track_caller]
fn compare(source_path: impl AsRef<Path>, target_path: impl AsRef<Path>) {
    let source_code = fs::read_to_string(source_path).unwrap();
    let target_code = fs::read_to_string(target_path).unwrap();

    let source_stream: Vec<Token> = lex(&source_code)
        .map(|(token, span)| token.map_err(|_| span))
        .collect::<Result<_, _>>()
        .unwrap();
    let target_stream: Vec<Token> = ron::from_str(&target_code).unwrap();

    assert_eq!(source_stream, target_stream);
}
