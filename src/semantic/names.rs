// <mangle> ::= "$" <path> <salt>
// <path> ::= {<path-element>}*
// <path-element> ::= <length> <ident>
// <ident> ::= <_a-zA-Z> {<_a-zA-Z0-9>}*
// <length> ::= {<0-9>}+
// <salt> ::= {<a-zA-Z>}*

pub fn mangle(path: Vec<String>, salt: &str) -> String {
  let mut buf: Vec<u8> = vec![];
  for p in path {
    let bytes = format!("{}{}", p.len(), punycode::encode(&p).unwrap());
    buf.extend_from_slice(bytes.as_bytes());
  }
  buf.extend_from_slice(salt.as_bytes());
  String::from_utf8(buf).unwrap()
}
