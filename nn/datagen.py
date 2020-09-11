import numpy as np
import csv
from torch.utils.data import DataLoader, Dataset
import torch
from typing import List


def normalize_data_to_tanh_space(x: np.array) -> np.array:
    return x * 2.0 - 1.0


class RealDataGenerator(Dataset):
    def __init__(self, files: List[str]):
        self.x = np.array([])
        self.files = files

    def __len__(self):
        return len(self.files)

    def prepare_image(self, x: np.array) -> np.array:
        x = x[:64]
        #  x = normalize_data_to_tanh_space(x)
        padding = np.array(
            #  [np.zeros_like(x[0]) - 1.0 for i in range(x[0].shape[0] - x.shape[0])]
            [np.zeros_like(x[0]) for i in range(x[0].shape[0] - x.shape[0])]
        )
        #  x = np.concatenate([x, padding])

        #  print("data_shape:", x.shape)
        x = np.array([x[:, :, i] for i in range(7)])
        #  print("transformed_shape:", im.shape)
        #  print(im)
        return x

    def __getitem__(self, idx: int):
        n_steps = None
        op_len = None
        with open(self.files[idx]) as csv_file:
            print(self.files[idx])
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

        #  print("n_steps:", n_steps)
        #  print("op_len:", op_len)
        x = x.reshape(-1, n_steps, op_len)

        return torch.tensor(self.prepare_image(x))

