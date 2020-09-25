import argparse
import os
import torch.nn.functional as F
import math
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

#  nz = 256
nc = 128
ndf = 128
#  ngf = 64


class SelfAttention(nn.Module):
    def __init__(self, emb, heads=2, mask=False):
        """
        :param emb:
        :param heads:
        :param mask:
        """

        super().__init__()

        self.emb = emb
        self.heads = heads
        self.mask = mask

        self.tokeys = nn.Linear(emb, emb * heads, bias=False)
        self.toqueries = nn.Linear(emb, emb * heads, bias=False)
        self.tovalues = nn.Linear(emb, emb * heads, bias=False)

        self.unifyheads = nn.Linear(heads * emb, emb)

    def forward(self, x):
        b, t, e = x.size()
        h = self.heads
        assert (
            e == self.emb
        ), f"Input embedding dim ({e}) should match layer embedding dim ({self.emb})"

        keys = self.tokeys(x).view(b, t, h, e)
        queries = self.toqueries(x).view(b, t, h, e)
        values = self.tovalues(x).view(b, t, h, e)

        # compute scaled dot-product self-attention

        # - fold heads into the batch dimension
        keys = keys.transpose(1, 2).contiguous().view(b * h, t, e)
        queries = queries.transpose(1, 2).contiguous().view(b * h, t, e)
        values = values.transpose(1, 2).contiguous().view(b * h, t, e)

        # - get dot product of queries and keys, and scale
        dot = torch.bmm(queries, keys.transpose(1, 2))
        dot = dot / math.sqrt(
            e
        )  # dot contains b*h  t-by-t matrices with raw self-attention logits

        assert dot.size() == (
            b * h,
            t,
            t,
        ), f"Matrix has size {dot.size()}, expected {(b*h, t, t)}."

        dot = F.softmax(dot, dim=2)  # dot now has row-wise self-attention probabilities

        # apply the self attention to the values
        out = torch.bmm(dot, values).view(b, h, t, e)

        # swap h, t back, unify heads
        out = out.transpose(1, 2).contiguous().view(b, t, h * e)

        return self.unifyheads(out)


class TransformerBlock(nn.Module):
    def __init__(self, emb, heads, mask, ff_hidden_mult=4, dropout=0.0):
        super().__init__()

        self.attention = SelfAttention(emb, heads=heads, mask=mask)
        self.mask = mask

        self.norm1 = nn.LayerNorm(emb)
        self.norm2 = nn.LayerNorm(emb)

        self.ff = nn.Sequential(
            nn.Linear(emb, ff_hidden_mult * emb),
            nn.LayerNorm(ff_hidden_mult * emb),
            nn.ReLU(),
            nn.Linear(ff_hidden_mult * emb, emb),
            nn.LayerNorm(emb),
            nn.ReLU(),
        )

        self.last = nn.Linear(emb, nc)

        self.do = nn.Dropout(dropout)
        self.tanh = nn.Tanh()

    def forward(self, x):
        attended = self.attention(x)
        x = self.norm1(attended + x)
        x = self.do(x)
        feedforward = self.ff(x)
        x = self.norm2(feedforward + x)
        x = self.last(x)
        x = self.do(x)

        return self.tanh(x)


class TransformerDiscriminator(nn.Module):
    def __init__(self):
        super(TransformerDiscriminator, self).__init__()
        #  self.ngpu = ngpu
        self.main = nn.Sequential(
            # input is (nc) x 64 x 64
            #  nn.Conv1d(nc, ndf, 4, stride=2, padding=1, bias=False),
            nn.Linear(nc, ndf),
            nn.LeakyReLU(0.2, inplace=True),
            nn.Dropout(0.1),
            #
            nn.Linear(ndf, ndf * 2),
            nn.LayerNorm(ndf * 2),
            nn.LeakyReLU(0.2, inplace=True),
            nn.Dropout(0.1),
            #
            nn.Linear(ndf * 2, ndf * 4),
            nn.LayerNorm(ndf * 4),
            nn.LeakyReLU(0.2, inplace=True),
            nn.Dropout(0.1),
            #
            nn.Linear(ndf * 4, ndf * 8),
            nn.LayerNorm(ndf * 8),
            nn.LeakyReLU(0.2, inplace=True),
            nn.Dropout(0.1),
            #
            nn.Linear(ndf * 8, 1),
            nn.Sigmoid(),
        )

    def forward(self, input):
        #  print("Discriminator input.shape:", input.shape)
        #  input = input + (0.1 ** 0.5) * torch.randn(input.shape).to("cuda")
        output = self.main(input)

        return output.view(-1, 1).squeeze(1)
