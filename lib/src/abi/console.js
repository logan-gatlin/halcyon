export const _compiler_exec = (b) => {
  const memory = new WebAssembly.Memory({initial: 10, maximum: 100});
  
  function print_string(offset, length) {
    const bytes = new Uint8Array(memory.buffer, offset, length);
    const string = new TextDecoder("utf8").decode(bytes);
    console.log(string);
  }
  function print_integer(num) {
    console.log(`${num}`);
  }
  function print_glyph(g) {
    const string = String.fromCharCode(g);
    console.log(string);
  }
  function print_boolean(b) {
    if (b == 1) {
      console.log("true");
    } else if (b == 2) {
      console.log("false");
    } else {
      console.log("invalid boolean");
    }
  }
  const imports = {
    js: {
      memory: memory,
      print_string: print_string,
      print_integer: print_integer,
      print_real: print_integer,
      print_glyph: print_glyph,
      print_boolean: print_boolean,
    }
  };
  WebAssembly.instantiate(b, imports);
}
export const _compiler_cls = () => {}
export const _compiler_wat = (s) => {
  console.log(s);
}
export const _compiler_print = (s) => {
  console.log(s);
}
