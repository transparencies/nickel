# capture = 'stderr'
# command = ['export']
# extra_args = [
#  '--',
#  'inputs.user="me"',
# ]
{
  inputs = {
    user | String | default = "defaultuser",
  },
  result = inputs.user,
}
