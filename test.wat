(module
	(global $2$s$0 (mut i32) (i32.const 0))
	(global $2$s$1 (mut i32) (i32.const 0))
	(global $2$s$2 (mut f32) (f32.const 0))

	(elem declare funcref (ref.func $testfunc))
	(func $testfunc)

	(func $$main
		(local $test funcref)
		(local $$tmp0$0 i32)
		(local $$tmp1$0 i32)
		(ref.func $testfunc)
		(local.set $test)
	)
)
