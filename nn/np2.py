import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim
import csv
from torchvision import transforms
from torch.utils.data import DataLoader, Dataset
from network import Generator, Discriminator, weights_init
from datagen import RealDataGenerator
from typing import List
import random
import os

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


files = sorted(files[0:1000])
dataset = RealDataGenerator(files)
for i in range(0, 3):
    data = dataset[i].numpy()
    data_point_to_rgbxyz_img(
        torch.tensor(data), i, 0, img_dir, song_name,
    )


write_result_to_file(dataset[0], n_voices, n_ops, "output/out.csv")

