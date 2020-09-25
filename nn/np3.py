import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim
import torch.nn.functional as F
import csv
from torchvision import transforms
from torch.utils.data import DataLoader, Dataset
from network2 import TransformerBlock, TransformerDiscriminator
from datagen import RealDataGenerator
from typing import List
import random
import os
import math

#  from PIL import Image
from helpers import write_result_to_file, data_point_to_rgbxyz_img

n_ops = 7
n_voices = 64

img_dir = "img"
song_name = "simple"

files = []
dirs = [f"data/{song_name}"]
for d in dirs:
    for f in os.listdir(d):
        if f.endswith(".csv"):
            files.append(os.path.join(d, f))


#  class Attention(nn.Module):
#  def __init__(self):
#  super(Attention, self).__init__()
#  self.softmax = nn.Softmax()

#  def forward(self, x):
#  #  raw_weights = torch.mm(x, x.T)
#  state = x
#  x = x.view(-1, 1) @ x.view(1, -1)
#  x = self.softmax(x)

#  return state @ x

batch_size = 1


#  class SelfAttention(nn.Module):
#  def __init__(self, k, heads=1):
#  super().__init__()
#  self.k, self.heads = (
#  k,
#  heads,
#  )
#  # These compute the queries, keys and values for all
#  # heads (as a single concatenated vector)
#  self.toqueries = nn.Linear(k, k * heads, bias=False)
#  self.tokeys = nn.Linear(k, k * heads, bias=False)
#  self.tovalues = nn.Linear(k, k * heads, bias=False)
#  # This unifies the outputs of the different heads into
#  # a single k-vector
#  self.unifyheads = nn.Linear(heads * k, k)

#  def forward(self, x):
#  h = self.heads
#  k = 1
#  b = batch_size
#  t = 64

#  queries = self.toqueries(x).view(b, -1, h, self.k)
#  keys = self.tokeys(x).view(b, -1, h, self.k)
#  values = self.tovalues(x).view(b, -1, h, self.k)

#  keys = keys.transpose(1, 2).contiguous().view(b * h, t, k)
#  queries = queries.transpose(1, 2).contiguous().view(b * h, t, k)
#  values = values.transpose(1, 2).contiguous().view(b * h, t, k)

#  queries = queries / (k ** (1 / 4))
#  keys = keys / (k ** (1 / 4))

#  dot = torch.bmm(queries, keys.transpose(1, 2))
#  # - dot has size (b*h, t, t) containing raw weights

#  dot = F.softmax(dot, dim=2)
#  out = torch.bmm(dot, values).view(b, h, t, k)
#  out = out.transpose(1, 2).contiguous().view(b, t, h * k)
#  return self.unifyheads(out)
#  # - dot now contains row-wise normalized weights


files = sorted(files[0:1000])
r = RealDataGenerator(files)
for i in range(0, 3):
    #  data = dataset[i].unsqueeze(0).unsqueeze(0)

    real_batch = [r[random.randint(0, len(r) - 1)].numpy() for i in range(batch_size)]

    real_batch = torch.tensor(real_batch).to("cuda", dtype=torch.float)
    label = torch.full((batch_size,), 1.0, dtype=torch.float, device="cuda")
    noise = torch.randn(batch_size, 1, 128, device="cuda", dtype=torch.float)
    print("real_batch.shape", real_batch.shape)
    print("noise", noise.shape)
    #  print(data)
    network = TransformerBlock(128, 1, None).to("cuda")
    #  disc = TransformerDiscriminator().to("cuda")
    #  y = network.forward(real_batch.float())
    #  print(y)
    #  print(y.shape)
    #  y = y.squeeze(1)
    pred = network.forward(real_batch)
    print(pred.detach())

    #  data_point_to_rgbxyz_img(
    #  torch.tensor(data).numpy(), i, 0, img_dir, song_name,
    #  )


#  write_result_to_file(dataset[0], n_voices, n_ops, "output/out.csv")

