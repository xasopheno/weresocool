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
from PIL import Image


n_ops = 7
n_voices = 64

img_dir = "img"


def inv_tanh(d: np.array):
    return (d + 1) * (1 / 2)


def ravel_to_wsc(x, n_voices, n_ops):
    return x.ravel("F").reshape(n_voices, n_voices, n_ops)


def data_point_to_rgbxyz_img(data: np.array, i: int):
    r = data[0].numpy()
    g = data[1].numpy()
    b = data[2].numpy()

    x = data[3].numpy()
    y = data[4].numpy()
    z = data[5].numpy()

    channels = np.concatenate([r, g, b, x, y, z], axis=0) * 255
    print(channels.shape)
    channels = channels.astype(np.uint8)
    channels = Image.fromarray(channels)
    channels.save(f"{img_dir}/out_channels_{i:05d}.png")

    a = np.dstack((r, g, b))
    b = np.dstack((x, y, z))
    rgb_xyz = np.concatenate([a, b], axis=0)

    rgb_xyz = inv_tanh(rgb_xyz) * 255
    rgb_xyz = rgb_xyz.astype(np.uint8)
    #  print(rgb_xyz)
    print(rgb_xyz.shape)

    result = Image.fromarray(rgb_xyz)
    result.save(f"{img_dir}/out_rgbxyz_{i:05d}.png")


files = []
dirs = ["data/how_to_rest"]
for d in dirs:
    for f in os.listdir(d):
        if f.endswith(".csv"):
            #  print(os.path.join(d, f))
            files.append(os.path.join(d, f))


files = files[0:1000]
dataset = RealDataGenerator(files)
for i in range(0, 1000, 3):
    #  i = random.randint(0, len(dataset) - 1)
    data = dataset[i].numpy()
    data = inv_tanh(data)
    data_point_to_rgbxyz_img(torch.tensor(data), i)

d = ravel_to_wsc(data, n_voices, n_ops)
#  print("WSC:", d)
