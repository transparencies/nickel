# capture = 'stdout'
# command = [ 'query', '--field', 'greeting', '--format', 'yaml' ]
{ greeting | doc "A friendly greeting" = "hello" }
