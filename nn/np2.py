import numpy as np
import torch
import csv
import random
from torch.utils.data import Dataset
from typing import List
import os

#  n_steps = None
#  op_len = None
#  with open("data/template_00000000.socool") as csv_file:
#  x = np.array([])
#  csv_reader = csv.reader(csv_file, delimiter=",")
#  line_count = 0
#  for row in csv_reader:
#  if line_count == 0:
#  r = list(map(int, row))
#  n_steps, op_len = r

#  line_count += 1
#  continue

#  r = np.array(list(map(float, row)))
#  x = np.append(x, [r])
#  line_count += 1
#  print("lines in file:", line_count)

#  print("n_steps:", n_steps)
#  print("op_len:", op_len)
#  x = x.reshape(-1, 4, 7)
#  print(x)


#  def __getitem__(self, batch_size: int):
#  def make_image():
#  im = x[random.randint(0, x.shape[0] - 1)]
#  im = im[0:64]
#  padding = np.array(
#  [
#  np.zeros_like(im[0]) - 1.0
#  for i in range(im[0].shape[0] - im.shape[0])
#  ]
#  )
#  im = np.concatenate([im, padding])

#  #  print("data_shape:", im.shape)
#  im = np.array([im[:, :, i] for i in range(7)])
#  #  print("transformed_shape:", im.shape)
#  #  print(im)
#  return im

#  return torch.tensor([make_image() for _ in range(batch_size)])
