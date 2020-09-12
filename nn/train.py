import numpy as np
import torch
import torch.nn as nn
import torch.optim as optim
import csv
from torchvision import transforms
from torch.utils.data import DataLoader, Dataset
import torchvision.utils as vutils
from network import Generator, Discriminator, weights_init
from datagen import RealDataGenerator
from typing import List
from helpers import data_point_to_rgbxyz_img, write_result_to_file
import random
import os


files = []
dirs = [
    "data/growing",
    "data/monica",
    "data/madness",
    "data/day_3",
    "data/slice",
]

for d in dirs:
    for f in os.listdir(d):
        if f.endswith(".csv"):
            #  print(os.path.join(d, f))
            files.append(os.path.join(d, f))

#  print(files)

#  files = files[0:1000]
random.shuffle(files)
print(files[0:30])
r = RealDataGenerator(files)
if __name__ == "__main__":
    nz = 256
    batch_size = 8
    device = "cuda"
    fixed_noise = torch.randn(batch_size, nz, 1, 1, device=device)
    ngpu = 2
    real_label = 0.9
    fake_label = 0.0
    epochs = 100
    lr = 0.0001
    beta1 = 0.5
    criterion = nn.BCELoss()

    netG = Generator(ngpu).to(device, dtype=torch.float)
    netG = nn.DataParallel(netG)
    netG.apply(weights_init)
    #  if opt.netG != "":
    #  netG.load_state_dict(torch.load(opt.netG))
    print(netG)

    netD = Discriminator(ngpu).to(device, dtype=torch.float)
    netD = nn.DataParallel(netD)
    netD.apply(weights_init)
    print(netD)
    #  if opt.netD != "":
    #  netD.load_state_dict(torch.load(opt.netD))

    optimizerG = optim.Adam(netG.parameters(), lr=0.00025, betas=(beta1, 0.99))
    optimizerD = optim.Adam(netD.parameters(), lr=0.001, betas=(beta1, 0.99))

    for epoch in range(epochs):
        for i in range(1000):
            ############################
            # (1) Update D network: maximize log(D(x)) + log(1 - D(G(z)))
            ###########################
            # train with real
            #  print("real________")
            netD.zero_grad()
            real_batch = [
                r[random.randint(0, len(r) - 1)].numpy() for i in range(batch_size)
            ]

            real_batch = torch.tensor(real_batch).to(device, dtype=torch.float)
            label = torch.full(
                (batch_size,), real_label, dtype=torch.float, device=device
            )

            #  label = torch.from_numpy(
            #  np.random.uniform(low=0.8, high=1.0, size=(batch_size,))
            #  ).to(device, dtype=torch.float)

            output = netD(real_batch)
            errD_real = criterion(output, label)
            errD_real.backward()
            D_x = output.mean().item()

            noise = torch.randn(batch_size, nz, 1, 1, device=device, dtype=torch.float)
            fake = netG(noise)
            label.fill_(fake_label)
            output = netD(fake.detach())
            errD_fake = criterion(output, label)
            errD_fake.backward()
            D_G_z1 = output.mean().item()
            errD = errD_real + errD_fake
            optimizerD.step()

            ############################
            # (2) Update G network: maximize log(D(G(z)))
            ###########################
            netG.zero_grad()
            label.fill_(real_label)  # fake labels are real for generator cost
            output = netD(fake)
            errG = criterion(output, label)
            errG.backward()
            D_G_z2 = output.mean().item()
            optimizerG.step()

            if i % 10 == 0:
                print(
                    "[%d/%d][%d/%d] Loss_D: %.4f Loss_G: %.4f D(x): %.4f D(G(z)): %.4f / %.4f"
                    % (
                        epoch,
                        epochs,
                        i,
                        len(r),
                        errD.item(),
                        errG.item(),
                        D_x,
                        D_G_z1,
                        D_G_z2,
                    )
                )
            if i % 500 == 0:
                print("___CREATING EXAMPLE___")
                fake = netG(fixed_noise).detach()
                for img_no in range(batch_size):
                    file_number = i + img_no
                    data = fake[img_no].cpu().numpy()

                    write_result_to_file(
                        data, 64, 7, f"output/{epoch:04}_{file_number:09d}.csv"
                    )
                    data_point_to_rgbxyz_img(
                        data, file_number, epoch, "result_img", "network"
                    )

        print(
            "[%d/%d][%d/%d] Loss_D: %.4f Loss_G: %.8f D(x): %.8f D(G(z)): %.8f / %.8f"
            % (epoch, epochs, i, len(r), errD.item(), errG.item(), D_x, D_G_z1, D_G_z2,)
        )
    #  op_len = 7
    #  n_steps = 4
    #  n_voices = 2
    #  n_examples = 3

    #  x = np.arange(n_steps * op_len * n_voices * n_examples)

