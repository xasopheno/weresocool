import numpy as np

n = 5
op_len = 7
n_voices = 3

x = np.arange(n * op_len * n_voices)

x = x.reshape(n_voices, -1, op_len)

print(x)

