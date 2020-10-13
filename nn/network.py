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


nz = 128
ngf = 512
ndf = 128
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


#  class Upsample2d(nn.Module):
#  def __init__(self, ngf, mult):
#  super(Upsample2d, self).__init__()
#  self.upsample = nn.Upsample(scale_factor=2, mode="nearest")
#  self.pad = nn.ReflectionPad2d(1)
#  self.conv = nn.Conv2d(ngf * mult, int(ngf * mult) / 2, kernel_size=3, padding=0)

#  def forward(self, x):
#  return x


class TransformerGenerator(nn.Module):
    def __init__(self):
        super(TransformerGenerator, self).__init__()
        self.relu = nn.ReLU(True)
        self.encoder_layers = nn.TransformerEncoderLayer(nz, 8)

        self.decoder = nn.Conv2d(ngf, nc, kernel_size=3, stride=1, padding=0)

        self.tanh = nn.Tanh()

    def forward(self, x):
        #  print("Generator input.shape:", input.shape)
        #  residual = x
        x = self.encoder_layers(x[0:nz])
        print(x.shape)
        #  print(x.shape)
        #  x = self.decoder(x)

        #  out = self.tanh(x)

        return x


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
        self.upsample2 = nn.Upsample(scale_factor=2, mode="nearest")
        self.pad2 = nn.ReflectionPad2d(1)
        self.conv2 = nn.Conv2d(
            ngf * 8, int(ngf * 8) // 2, kernel_size=3, stride=1, padding=0
        )

        self.batch_norm1 = nn.BatchNorm2d(ngf * 8)
        # state size. (ngf*8) x 4 x 4

        #  self.conv2 = nn.ConvTranspose2d(ngf * 8, ngf * 4, 4, 2, 1, bias=False)
        #  https://github.com/junyanz/pytorch-CycleGAN-and-pix2pix/issues/190
        self.upsample2 = nn.Upsample(scale_factor=2, mode="nearest")
        self.pad2 = nn.ReflectionPad2d(1)
        self.conv2 = nn.Conv2d(
            ngf * 8, int(ngf * 8) // 2, kernel_size=3, stride=1, padding=0
        )

        self.batch_norm2 = nn.BatchNorm2d(ngf * 4)
        # state size. (ngf*4) x 8 x 8
        #  self.conv3 = nn.ConvTranspose2d(ngf * 4, ngf * 2, 4, 2, 1, bias=False)
        self.upsample3 = nn.Upsample(scale_factor=2, mode="nearest")
        self.pad3 = nn.ReflectionPad2d(1)
        self.conv3 = nn.Conv2d(
            ngf * 4, int(ngf * 4) // 2, kernel_size=3, stride=1, padding=0
        )

        self.batch_norm3 = nn.BatchNorm2d(ngf * 2)
        # state size. (ngf*2) x 16 x 16
        #  self.conv4 = nn.ConvTranspose2d(ngf * 2, ngf, 4, 2, 1, bias=False)
        self.upsample4 = nn.Upsample(scale_factor=2, mode="nearest")
        self.pad4 = nn.ReflectionPad2d(1)
        self.conv4 = nn.Conv2d(
            ngf * 2, int(ngf * 2) // 2, kernel_size=3, stride=1, padding=0
        )

        self.batch_norm4 = nn.BatchNorm2d(ngf)
        #  self.conv5 = nn.ConvTranspose2d(ngf, nc, 4, 2, 1, bias=False)
        self.upsample5 = nn.Upsample(scale_factor=2, mode="nearest")
        self.pad5 = nn.ReflectionPad2d(1)
        self.conv5 = nn.Conv2d(ngf, nc, kernel_size=3, stride=1, padding=0)

        self.tanh = nn.Tanh()

    def forward(self, x):
        #  print("Generator input.shape:", input.shape)
        #  residual = x

        out = self.conv1(x)
        out = self.batch_norm1(out)
        out = self.relu(out)

        out = self.upsample2(out)
        out = self.pad2(out)
        out = self.conv2(out)

        out = self.batch_norm2(out)
        #  out += residual
        out = self.relu(out)
        #  residual = out

        out = self.upsample3(out)
        out = self.pad3(out)
        out = self.conv3(out)

        out = self.batch_norm3(out)
        out = self.relu(out)

        out = self.upsample4(out)
        out = self.pad4(out)
        out = self.conv4(out)

        out = self.batch_norm4(out)
        out = self.relu(out)

        out = self.upsample5(out)
        out = self.pad5(out)
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
            nn.Dropout2d(0.1),
            # state size. (ndf) x 32 x 32
            nn.Conv2d(ndf, ndf * 2, 4, 2, 1, bias=False),
            nn.BatchNorm2d(ndf * 2),
            nn.LeakyReLU(0.2, inplace=True),
            nn.Dropout2d(0.1),
            # state size. (ndf*2) x 16 x 16
            (nn.Conv2d(ndf * 2, ndf * 4, 4, 2, 1, bias=False)),
            nn.BatchNorm2d(ndf * 4),
            nn.LeakyReLU(0.2, inplace=True),
            nn.Dropout2d(0.1),
            # state size. (ndf*4) x 8 x 8
            nn.Conv2d(ndf * 4, ndf * 8, 4, 2, 1, bias=False),
            nn.BatchNorm2d(ndf * 8),
            nn.LeakyReLU(0.2, inplace=True),
            nn.Dropout2d(0.1),
            # state size. (ndf*8) x 4 x 4
            nn.Conv2d(ndf * 8, 1, 4, 1, 0, bias=False),
            # state size. 1
            nn.Sigmoid(),
        )

    def forward(self, input):
        #  print("Discriminator input.shape:", input.shape)
        #  input = input + (0.1 ** 0.5) * torch.randn(input.shape).to("cuda")
        output = self.main(input)

        return output.view(-1, 1).squeeze(1)

