import argparse
import os
import random
import torch
import torch.nn as nn
import torch.nn.parallel
import torch.backends.cudnn as cudnn
import torch.optim as optim
import torch.utils.data
import torchvision.datasets as dset
import torchvision.transforms as transforms
import torchvision.utils as vutils

nz = 256
ngf = 64
ndf = 64
nc = 7

# custom weights initialization called on netG and netD
def weights_init(m):
    classname = m.__class__.__name__
    if classname.find("Conv") != -1:
        torch.nn.init.normal_(m.weight, 0.0, 0.02)
    elif classname.find("BatchNorm") != -1:
        torch.nn.init.normal_(m.weight, 1.0, 0.02)
        torch.nn.init.zeros_(m.bias)

    #  x = np.arange(n_steps * op_len * n_voices * n_examples)


class Generator(nn.Module):
    def __init__(self, ngpu):
        super(Generator, self).__init__()
        self.ngpu = ngpu
        self.relu = nn.ReLU(True)
        self.pool = nn.MaxPool2d(
            kernel_size=2, stride=2, padding=1, return_indices=True
        )
        # input is Z, going into a convolution
        self.conv1 = nn.ConvTranspose2d(nz, ngf * 8, 4, 1, 0, bias=False)
        self.batch_norm1 = nn.BatchNorm2d(ngf * 8)
        # state size. (ngf*8) x 4 x 4
        self.conv2 = nn.ConvTranspose2d(ngf * 8, ngf * 4, 4, 2, 1, bias=False)
        self.batch_norm2 = nn.BatchNorm2d(ngf * 4)
        # state size. (ngf*4) x 8 x 8
        self.conv3 = nn.ConvTranspose2d(ngf * 4, ngf * 2, 4, 2, 1, bias=False)
        self.batch_norm3 = nn.BatchNorm2d(ngf * 2)
        # state size. (ngf*2) x 16 x 16
        self.conv4 = nn.ConvTranspose2d(ngf * 2, ngf, 4, 2, 1, bias=False)

        self.batch_norm4 = nn.BatchNorm2d(ngf)
        self.conv5 = nn.ConvTranspose2d(ngf, nc, 4, 2, 1, bias=False)

        self.tanh = nn.Tanh()

        #  self.main = nn.Sequential(
        # input is Z, going into a convolution
        #  nn.ConvTranspose2d(nz, ngf * 8, 4, 1, 0, bias=False),
        #  nn.BatchNorm2d(ngf * 8),
        #  nn.ReLU(True),

        # state size. (ngf*8) x 4 x 4
        #  nn.ConvTranspose2d(ngf * 8, ngf * 4, 4, 2, 1, bias=False),
        #  nn.BatchNorm2d(ngf * 4),
        #  nn.ReLU(True),

        # state size. (ngf*4) x 8 x 8
        #  nn.ConvTranspose2d(ngf * 4, ngf * 2, 4, 2, 1, bias=False),
        #  nn.BatchNorm2d(ngf * 2),
        #  nn.ReLU(True),

        # state size. (ngf*2) x 16 x 16
        #  nn.ConvTranspose2d(ngf * 2, ngf, 4, 2, 1, bias=False),
        #  nn.BatchNorm2d(ngf),
        #  nn.ReLU(True),

        # state size. (ngf) x 32 x 32
        #  nn.ConvTranspose2d(ngf, nc, 4, 2, 1, bias=False),
        #  nn.Tanh()
        # state size. (nc) x 64 x 64
        #  )

    def forward(self, x):
        #  print("Generator input.shape:", input.shape)
        #  if input.is_cuda and self.ngpu > 1:
        #  output = nn.parallel.data_parallel(self.main, input, range(self.ngpu))
        #  else:
        #  output = self.main(input)
        identity = x

        out = self.conv1(x)
        out = self.batch_norm1(out)
        out = self.relu(out)

        out = self.conv2(out)
        out = self.batch_norm2(out)
        out += identity
        out = self.relu(out)

        identity = out

        out = self.conv3(out)
        out = self.batch_norm3(out)
        out = self.relu(out)

        out = self.conv4(out)
        out = self.batch_norm4(out)
        #  out += identity
        out = self.relu(out)

        out = self.conv5(out)
        out = self.tanh(out)

        return out


class Discriminator(nn.Module):
    def __init__(self, ngpu):
        super(Discriminator, self).__init__()
        self.ngpu = ngpu
        self.main = nn.Sequential(
            # input is (nc) x 64 x 64
            nn.Conv2d(nc, ndf, 4, stride=2, padding=1, bias=False),
            nn.LeakyReLU(0.2, inplace=True),
            # state size. (ndf) x 32 x 32
            nn.Conv2d(ndf, ndf * 2, 4, 2, 1, bias=False),
            nn.BatchNorm2d(ndf * 2),
            nn.LeakyReLU(0.2, inplace=True),
            # state size. (ndf*2) x 16 x 16
            nn.Conv2d(ndf * 2, ndf * 4, 4, 2, 1, bias=False),
            nn.BatchNorm2d(ndf * 4),
            nn.LeakyReLU(0.2, inplace=True),
            # state size. (ndf*4) x 8 x 8
            nn.Conv2d(ndf * 4, ndf * 8, 4, 2, 1, bias=False),
            nn.BatchNorm2d(ndf * 8),
            nn.LeakyReLU(0.2, inplace=True),
            nn.Dropout2d(0.4),
            # state size. (ndf*8) x 4 x 4
            nn.Conv2d(ndf * 8, 1, 4, 1, 0, bias=False),
            nn.Sigmoid()
            # state size. 1
        )

    def forward(self, input):
        #  print("Discriminator input.shape:", input.shape)
        input = input + (0.18 ** 0.5) * torch.randn(input.shape).to("cuda")
        #  if input.is_cuda and self.ngpu > 1:
        #  output = nn.parallel.data_parallel(self.main, input, range(self.ngpu))
        #  else:
        output = self.main(input)

        return output.view(-1, 1).squeeze(1)

