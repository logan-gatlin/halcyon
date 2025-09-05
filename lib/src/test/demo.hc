module test = 
  let my_result = result::Ok (2)
  
  let () = 
  if result::is_ok my_result 
    then string::print "Ok!"
  else
      string::print "Error!"

  (* prints "Ok!" *)
end
