(module
(func $3$func
(param $4$s$0 i32)
(param $4$s$1 i32)
(param $4$s$2 f32)
(param $5$a$0 i32)
(result i32 i32 f32 )
local.get $4$s$2
local.get $4$s$1
local.get $4$s$0
return
)
(global $2$s$0 (mut i32))
(global $2$s$1 (mut i32))
(global $2$s$2 (mut f32))
(func $$main
(local $$tmp0$0 i32)
(local $$tmp1$0 i32)
i32.const 2
i32.const 1
local.set $$tmp0$0
i32.const 97
local.set $$tmp1$0
f32.const 1
local.get $$tmp1$0
local.get $$tmp0$0
call $3$func
global.set $2$s$0
global.set $2$s$1
global.set $2$s$2
)
)