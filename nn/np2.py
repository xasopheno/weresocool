import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim
import csv
from torchvision import transforms
from torch.utils.data import DataLoader, Dataset
from network import Generator, Discriminator, weights_init
from typing import List
import random
import os
from PIL import Image


def normalize_data_to_tanh_space(x: np.array) -> np.array:
    return x * 2.0 - 1.0


class RealDataGenerator(Dataset):
    def __init__(self, files: List[str]):
        self.x = np.array([])
        self.files = files

    def __len__(self):
        return len(files)

    def prepare_image(self, x: np.array) -> np.array:
        x = x[:64]
        print(x[0])
        x = normalize_data_to_tanh_space(x)
        padding = np.array(
            [np.zeros_like(x[0]) - 1.0 for i in range(x[0].shape[0] - x.shape[0])]
        )
        x = np.concatenate([x, padding])

        #  print("data_shape:", x.shape)
        x = np.array([x[:, :, i] for i in range(7)])
        #  print("transformed_shape:", im.shape)
        #  print(im)
        return x

    def __getitem__(self, idx: int):
        n_steps = None
        op_len = None
        with open(self.files[idx]) as csv_file:
            x = np.array([])
            csv_reader = csv.reader(csv_file, delimiter=",")
            line_count = 0
            for row in csv_reader:
                if line_count == 0:
                    r = list(map(int, row))
                    n_steps, op_len = r

                    line_count += 1
                    continue

                r = np.array(list(map(float, row)))
                x = np.append(x, [r])
                line_count += 1
            #  print("lines in file:", line_count)

        x = x.reshape(-1, n_steps, op_len)

        return torch.tensor(self.prepare_image(x))


files = []
dirs = ["data/template"]
for d in dirs:
    for f in os.listdir(d):
        if f.endswith(".csv"):
            #  print(os.path.join(d, f))
            files.append(os.path.join(d, f))

#  print(files)


def de_tanh(d: np.array):
    return (d + 1) * (1 / 2)


files = files[0:1000]
dataset = RealDataGenerator(files)


data = dataset[0].numpy()
data = de_tanh(data)
#  print(data)
#  d = np.array([data[:, :, :] for i in range(7)])
#  print(d)
# Concatenate three (height, width)s into one (height, width, 3).
def concat_channels(x):
    #  rgb = [r[..., np.newaxis], g[..., np.newaxis], b[..., np.newaxis]
    #  x = np.array([x[i, :, :] for i in range(7)])

    return x.ravel("F").reshape(4, 4, 7)


d = concat_channels(data)
print("DONE:", d)


def datapoint_to_image(data: np.array):
    r = data[0].numpy()
    g = data[1].numpy()
    b = data[2].numpy()

    x = data[3].numpy()
    y = data[4].numpy()
    z = data[5].numpy()

    a = np.dstack((r, g, b))
    b = np.dstack((x, y, z))
    d = np.concatenate([a, b], axis=0)

    d = de_tanh(d) * 255
    d = d.astype(np.uint8)
    #  print(d)
    print(d.shape)

    result = Image.fromarray(d)
    result.save("out.png")

