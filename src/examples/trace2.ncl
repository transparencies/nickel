let foo = fun x => Assume(Num, x) + 3 in
let id = fun x => x in
let bar = fun b y => if b (id true) then foo y else -1 in
let k = fun y x => Assume(Num, y) + (foo x) in
bar (k (id 4)) false
