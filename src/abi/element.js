let print_output = document.getElementById("print-out");
let wat_output = document.getElementById("wat-out");

export const _compiler_exec = (b) => {
  const memory = new WebAssembly.Memory({initial: 10, maximum: 100});
  
  function print_string(offset, length) {
    const bytes = new Uint8Array(memory.buffer, offset, length);
    const string = new TextDecoder("utf8").decode(bytes);
    print_output.value = print_output.value.concat(string);
  }
  function print_integer(num) {
    print_output.value = print_output.value.concat(`${num}`);
  }
  function print_glyph(g) {
    const string = String.fromCharCode(g);
    print_output.value = print_output.value.concat(string);
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
export const _compiler_cls = () => {
  print_output.value = "";
}
export const _compiler_wat = (s) => {
  console.log(s);
}
export const _compiler_print = (s) => {
  print_output.value = print_output.value.concat(s);
}
