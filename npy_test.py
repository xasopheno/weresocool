import numpy as np

op_len = 7
n_voices = 60
n_steps = 100
n_examples = 2


x = np.arange(n_steps * op_len * n_voices * n_examples)

x = x.reshape(n_examples, n_voices, -1, op_len)

print(x)

