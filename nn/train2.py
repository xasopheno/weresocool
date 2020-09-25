import numpy as np
import math
import torch
import torch.nn as nn
import torch.optim as optim
import csv
from torchvision import transforms
from torch.utils.data import DataLoader, Dataset
import torchvision.utils as vutils
from network2 import TransformerDiscriminator, TransformerBlock
from network import weights_init
from datagen import RealDataGenerator
from typing import List
from helpers import data_point_to_rgbxyz_img, write_result_to_file, rejoin_channels
import random
import os


files = []
dirs = [
    #  "data/growing",
    #  "data/monica"
    #  "data/madness",
    #  "data/day_3",
    #  "data/slice",
    #  "data/simple2_fifths"
    "data/simple"
]

for d in dirs:
    for f in os.listdir(d):
        if f.endswith(".csv"):
            #  print(os.path.join(d, f))
            files.append(os.path.join(d, f))


def cyclical_lr(stepsize, min_lr=3e-4, max_lr=3e-3):

    # Scaler: we can adapt this if we do not want the triangular CLR
    scaler = lambda x: 1.0

    # Lambda function to calculate the LR
    lr_lambda = lambda it: min_lr + (max_lr - min_lr) * relative(it, stepsize)

    # Additional function to see where on the cycle we are
    def relative(it, stepsize):
        cycle = math.floor(1 + it / (2 * stepsize))
        x = abs(it / stepsize - 2 * cycle + 1)
        return max(0, (1 - x)) * scaler(cycle)

    return lr_lambda


#  print(files)
#  files = files[0:1000]
random.shuffle(files)
print(files[0:30])
r = RealDataGenerator(files[0:500])

if __name__ == "__main__":
    nz = 256
    batch_size = 8
    n_ops = 1
    device = "cuda"
    fixed_noise = torch.randn(batch_size, n_ops, nz, device=device, dtype=torch.float)
    ngpu = 2
    real_label = 0.9
    fake_label = 0.0
    epochs = 400
    lr = 0.0001
    beta1 = 0.5
    criterion = nn.BCELoss()

    netG = TransformerBlock(nz, 8, None).to(device, dtype=torch.float)
    #  netG = TransformerGenerator().to(device, dtype=torch.float)

    netG = nn.DataParallel(netG)
    netG.apply(weights_init)
    #  if opt.netG != "":
    #  netG.load_state_dict(torch.load("./trained_models/netG.pt"))
    print(netG)

    netD = TransformerDiscriminator().to(device, dtype=torch.float)
    netD = nn.DataParallel(netD)
    netD.apply(weights_init)
    #  if opt.netD != "":
    #  netD.load_state_dict(torch.load("./trained_models/netD.pt"))
    print(netD)

    optimizerG = optim.Adam(netG.parameters(), lr=0.0001, betas=(beta1, 0.999))
    #  schedulerG = torch.optim.lr_scheduler.ReduceLROnPlateau(
    #  optimizerG, "min", verbose=True, patience=500
    #  )
    optimizerD = optim.Adam(netD.parameters(), lr=0.0004, betas=(beta1, 0.999))
    #  schedulerD = torch.optim.lr_scheduler.ReduceLROnPlateau(
    #  optimizerD, "min", verbose=True, patience=500)

    for epoch in range(epochs):
        for i in range(len(r)):
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

            noise = torch.randn(batch_size, n_ops, nz, device=device, dtype=torch.float)
            fake = netG(noise)
            label.fill_(fake_label)
            output = netD(fake.detach())
            errD_fake = criterion(output, label)
            errD_fake.backward()
            D_G_z1 = output.mean().item()
            errD = errD_real + errD_fake
            optimizerD.step()
            #  schedulerD.step(errD)

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
            #  schedulerG.step(errG)

            if i % 20 == 0:
                print(
                    "[%d/%d][%d/%d] Loss_D: %.4f Loss_G: %.4f D(x): %.4f D(G(z)): %.8f / %.8f"
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
            #  if i % 50 == 0:
            if i == 0:
                print("___CREATING EXAMPLE___")
                fake = netG(fixed_noise).detach()
                #  fake = torch.tensor(
                #  np.array(
                #  [
                #  r[random.randint(0, len(r) - 1)].numpy()
                #  for i in range(batch_size)
                #  ]
                #  )
                #  )
                for img_no in range(batch_size):
                    file_number = i + img_no

                    #  real_batch = torch.tensor(real_batch).to(device, dtype=torch.float)
                    data = fake[img_no].cpu().numpy()
                    data = rejoin_channels(data)

                    write_result_to_file(
                        data, f"output/{epoch:04}_{file_number:09d}.csv"
                    )
                    data_point_to_rgbxyz_img(
                        data.transpose(), file_number, epoch, "result_img", "network"
                    )

        torch.save(netG.state_dict(), "trained_models/netG.pt")
        torch.save(netD.state_dict(), "trained_models/netD.pt")

        print(
            "[%d/%d][%d/%d] Loss_D: %.4f Loss_G: %.8f D(x): %.8f D(G(z)): %.8f / %.8f"
            % (epoch, epochs, i, len(r), errD.item(), errG.item(), D_x, D_G_z1, D_G_z2,)
        )
    #  op_len = 7
    #  n_steps = 4
    #  n_voices = 2
    #  n_examples = 3

    #  x = np.arange(n_steps * op_len * n_voices * n_examples)

