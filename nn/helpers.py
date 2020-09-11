import numpy as np
import os
from PIL import Image


def pad_image(x):
    padding = np.array(
        [np.zeros_like(x[0]) - 1.0 for i in range(x[0].shape[0] - x.shape[0])]
    )
    return np.concatenate([x, padding])


def separate_channels(x):
    return np.array([x[:, :, i] for i in range(7)])


def rejoin_channels(data):
    return np.array([np.array(data[:, i, :]).ravel("F") for i in range(data.shape[1])])


def normalize_data_to_tanh_space(x: np.array) -> np.array:
    return x * 2.0 - 1.0


def denormalize_data_from_tanh_space(d: np.array):
    return (d + 1) * (1 / 2)


def data_point_to_rgbxyz_img(data: np.array, i: int, song_name: str, img_dir: str):
    data = denormalize_data_from_tanh_space(data)

    r = data[0].numpy()
    g = data[1].numpy()
    b = data[2].numpy()

    x = data[3].numpy()
    y = data[4].numpy()
    z = data[5].numpy()

    channels = np.concatenate([r, g, b, x, y, z], axis=0) * 255

    channels = channels.astype(np.uint8)
    channels = Image.fromarray(channels)
    channels.save(f"{img_dir}/{song_name}/out_channels_{i:05d}.png")

    a = np.dstack((r, g, b))
    b = np.dstack((x, y, z))

    rgb_xyz = np.concatenate([a, b], axis=0)
    rgb_xyz = denormalize_data_from_tanh_space(rgb_xyz) * 255
    rgb_xyz = rgb_xyz.astype(np.uint8)

    result = Image.fromarray(rgb_xyz)
    result.save(f"{img_dir}/{song_name}/out_rgbxyz_{i:05d}.png")


def write_result_to_file(data, n_voices, n_op, filename):
    data = denormalize_data_from_tanh_space(data)
    d = rejoin_channels(data).flatten()

    np.savetxt(filename, d, delimiter=",", fmt="%1.53f")
