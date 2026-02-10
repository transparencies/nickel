# capture = 'stderr'
# command = ['eval']
let my_div = fun x =>
  if x == 0 then
    std.fail_with "div by zero"
  else
    2 / x
in
my_div 0
