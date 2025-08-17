module TypeTest = 
  import std

  // Struct field inference
  type specific_struct1 = {
    specific_name1: std:integer
  }
  type specific_struct2 = {
    specific_name2: specific_struct1
  }
  type specific_struct3 = {
    specific_name3: specific_struct2
  }

  
  let tricky_inference = fn a => a.specific_name3.specific_name2.specific_name1
end
