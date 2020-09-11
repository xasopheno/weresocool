import unittest
from helpers import (
    normalize_data_to_tanh_space,
    denormalize_data_from_tanh_space,
    separate_channels,
    rejoin_channels,
    pad_image,
)
import numpy as np


class TestStrings(unittest.TestCase):
    def test_normalization(self):
        x = np.array([0.2, 0.5, 0.8])
        normalized = normalize_data_to_tanh_space(x)
        denormalized = denormalize_data_from_tanh_space(normalized)

        self.assertTrue(all([a == b for (a, b) in zip(list(denormalized), list(x))]))

    def test_separate_channels(self):
        x = np.array([[[1, 2, 3, 4, 5, 6, 7]], [[8, 9, 10, 11, 12, 13, 14]]])
        separated = separate_channels(x)
        rejoined = rejoin_channels(separated).flatten()

        self.assertTrue(
            all([a == b for (a, b) in zip(list(x.flatten()), list(rejoined))])
        )

    def test_pad_image(self):
        x = np.array([[1, 1, 1]])
        padded = pad_image(x)
        expected = np.array([[1, 1, 1], [-1, -1, -1], [-1, -1, -1]])
        self.assertEqual(padded.shape, expected.shape)
        self.assertTrue(
            all([a == b for (a, b) in zip(list(x.flatten()), list(expected.flatten()))])
        )


def pad_image(x):
    padding = np.array(
        [np.zeros_like(x[0]) - 1.0 for i in range(x[0].shape[0] - x.shape[0])]
    )
    return np.concatenate([x, padding])
