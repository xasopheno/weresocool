import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim
import csv
from torchvision import transforms
from torch.utils.data import DataLoader, Dataset
from network import Generator, Discriminator, weights_init
import random


def normalize_data_to_tanh_space(x: np.array) -> np.array:
    return x * 2.0 - 1.0


class RealDataGenerator(Dataset):
    def __init__(self, x: np.array):
        self.x = x

    def __len__(self):
        return x.shape[0]

    def __getitem__(self, batch_size: int):
        """
        Given d: RealDataGenerator, you can call d[batch_size] to get a training batch of that size.
        Terrible python, but I'm having fun.
        """

        def make_image():
            im = x[random.randint(0, x.shape[0] - 1)]
            im = im[0:64]
            padding = np.array(
                [
                    np.zeros_like(im[0]) - 1.0
                    for i in range(im[0].shape[0] - im.shape[0])
                ]
            )
            im = np.concatenate([im, padding])

            #  print("data_shape:", im.shape)
            im = np.array([im[:, :, i] for i in range(7)])
            #  print("transformed_shape:", im.shape)
            #  print(im)
            return im

        return torch.tensor([make_image() for _ in range(batch_size)])


if __name__ == "__main__":
    nz = 100
    x = np.array([])

    n_voices = None
    n_steps = None
    op_len = None
    with open("data.csv") as csv_file:
        csv_reader = csv.reader(csv_file, delimiter=",")
        line_count = 0
        for row in csv_reader:
            if line_count == 0:
                r = list(map(int, row))
                n_voices, n_steps, op_len = r

                line_count += 1
                continue

            r = np.array(list(map(float, row)))
            x = np.append(x, [r])
            line_count += 1
        #  print(x)
        print(line_count)

    print("n_voices:", n_voices)
    print("n_steps:", n_steps)
    print("op_len:", op_len)
    x = x.reshape(-1, n_voices, n_steps, op_len)
    print("n_examples:", x.shape[0])
    x = normalize_data_to_tanh_space(x)
    #  x = [:, :, channel]
    batch_size = 1

    #  data = DataLoader(
    #  x,
    #  batch_size=2,
    #  shuffle=True,
    #  sampler=None,
    #  batch_sampler=None,
    #  num_workers=0,
    #  collate_fn=None,
    #  pin_memory=False,
    #  drop_last=True,
    #  timeout=0,
    #  worker_init_fn=None,
    #  )

    r = RealDataGenerator(x)
    #  f = FakeDataGenerator(x[0].shape)

    device = "cuda"
    ngpu = 2
    real_label = 1
    fake_label = 0
    epochs = 1
    lr = 0.0002
    beta1 = 0.5
    criterion = nn.BCELoss()

    netG = Generator(ngpu).to(device, dtype=torch.double)
    netG.apply(weights_init)
    #  if opt.netG != "":
    #  netG.load_state_dict(torch.load(opt.netG))
    print(netG)

    netD = Discriminator(ngpu).to(device, dtype=torch.double)
    netD.apply(weights_init)
    print(netD)
    #  if opt.netD != "":
    #  netD.load_state_dict(torch.load(opt.netD))

    optimizerD = optim.Adam(netD.parameters(), lr=lr, betas=(beta1, 0.999))
    optimizerG = optim.Adam(netG.parameters(), lr=lr, betas=(beta1, 0.999))

    for epoch in range(epochs):
        for i in range(len(r)):
            ############################
            # (1) Update D network: maximize log(D(x)) + log(1 - D(G(z)))
            ###########################
            # train with real
            #  print("real________")
            netD.zero_grad()
            real_batch = r[batch_size].to(device, dtype=torch.double)
            #  real_batch = r[batch_size].to(device)
            #  batch_size = real_batch.size(0)
            label = torch.full(
                (batch_size,), real_label, dtype=torch.double, device=device
            )

            output = netD(real_batch)
            errD_real = criterion(output, label)
            errD_real.backward()
            D_x = output.mean().item()
            #  print("D_x:", D_x)

            #  print("fake________")
            noise = torch.randn(batch_size, nz, 1, 1, device=device, dtype=torch.double)
            #  noise = torch.randn((batch_size,) + x[0].shape)

            fake = netG(noise)
            output = netD(fake.detach())
            errD_fake = criterion(output, label)
            errD_fake.backward()
            D_G_z1 = output.mean().item()
            #  print("D_G_z1:", D_G_z1)
            errD = errD_real + errD_fake
            optimizerD.step()

    #  op_len = 7
    #  n_steps = 4
    #  n_voices = 2
    #  n_examples = 3

    #  x = np.arange(n_steps * op_len * n_voices * n_examples)

