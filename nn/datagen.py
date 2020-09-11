import numpy as np
import csv
from torch.utils.data import DataLoader, Dataset
import torch
from typing import List
from helpers import pad_image, separate_channels, normalize_data_to_tanh_space


class RealDataGenerator(Dataset):
    def __init__(self, files: List[str]):
        self.files = files

    def __len__(self):
        return len(self.files)

    def prepare_image(self, x: np.array) -> np.array:
        x = x[:64]
        x = normalize_data_to_tanh_space(x)
        x = pad_image(x)
        x = separate_channels(x)

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

        x = x.reshape(-1, n_steps, op_len)

        return torch.tensor(self.prepare_image(x))

