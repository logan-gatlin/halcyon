use super::format_source_text;

fn format_ok(source: &str) -> String {
    format_source_text("test.hc", source).expect("format should succeed")
}

#[test]
fn format_basic_module() {
    let source = "module M = let x = 1 end";
    let expected = "module M =\n  let x = 1\nend\n";
    assert_eq!(format_ok(source), expected);
}

#[test]
fn format_match_without_pipe() {
    let source = "module M =\n  let x = match y with\n    0 => 1\n    | n => n\nend";
    let expected = "module M =\n  let x =\n    match y with\n      | 0 => 1\n      | n => n\nend\n";
    assert_eq!(format_ok(source), expected);
}

#[test]
fn format_expr_precedence() {
    let source = "module M =\n  let a = f x + g y\n  let b = a * (b + c)\n  let c = f r.x\nend";
    let expected = "module M =\n  let a = f x + g y\n  let b = a * (b + c)\n  let c = f r.x\nend\n";
    assert_eq!(format_ok(source), expected);
}

#[test]
fn format_array_splat() {
    let source = "module M =\n  let xs = [1 2 ..ys]\nend";
    let expected = "module M =\n  let xs = [1, 2, ..ys]\nend\n";
    assert_eq!(format_ok(source), expected);
}

#[test]
fn format_type_precedence() {
    let source = "module M =\n  type t = (int -> int) list\n  type u = int -> list int\nend";
    let expected = "module M =\n  type t = (int -> int) list\n  type u = int -> list int\nend\n";
    assert_eq!(format_ok(source), expected);
}
