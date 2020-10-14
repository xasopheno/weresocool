import numpy as np
import os
from PIL import Image


def pad_image(x):
    padding = np.array(
        [np.zeros_like(x[0]) - 1.0 for i in range(x[0].shape[0] - x.shape[0])]
    )
    return np.concatenate([x, padding])


def separate_channels(x):
    return np.array([x[:, :, i] for i in range(2)])


def rejoin_channels(data):
    #  return np.array([np.array(data[:, i, :]).ravel("F") for i in range(data.shape[1])])
    data = data.reshape(4, -1)
    #  print(data.shape)
    a = np.array([data[0], data[2]])
    a = np.array([np.array(a[:, i]).ravel("F") for i in range(a.shape[1])])
    #  print(a.shape)
    b = np.array([data[1], data[3]])
    b = np.array([np.array(b[:, i]).ravel("F") for i in range(b.shape[1])])
    #  print(b.shape)
    #  data = np.array([np.array(data[:, i]).ravel("F") for i in range(data.shape[1])])
    #  print(data.shape)

    result = np.array([a.flatten(), b.flatten()]).reshape(4, -1)
    #  result = result.flatten()
    #  print(result.shape)
    #  exit(0)
    return result


def normalize_data_to_tanh_space(x: np.array) -> np.array:
    return x * 2.0 - 1.0


def denormalize_data_from_tanh_space(d: np.array):
    return (d + 1) * (1 / 2)


def data_point_to_rgbxyz_img(
    data: np.array, i: int, epoch: int, img_dir: str, song_name: str,
):
    data = denormalize_data_from_tanh_space(data)

    #  r = data[0]
    #  g = data[1]
    #  b = data[2]

    #  x = data[3]
    #  y = data[4]
    #  z = data[5]

    #  channels = np.concatenate([r, g, b, x, y, z], axis=0) * 255

    channels = data * 255.0
    channels = channels.astype(np.uint8)
    channels = Image.fromarray(channels)
    channels.save(f"{img_dir}/{song_name}/out_channels_{epoch:04d}_{i:09d}.png")

    #  a = np.dstack((r, g, b))
    #  b = np.dstack((x, y, z))

    #  rgb_xyz = np.concatenate([a, b], axis=0)
    #  rgb_xyz = denormalize_data_from_tanh_space(rgb_xyz) * 255
    #  rgb_xyz = rgb_xyz.astype(np.uint8)

    #  result = Image.fromarray(rgb_xyz)
    #  result.save(f"{img_dir}/{song_name}/out_rgbxyz_{epoch:04d}_{i:09d}.png")
    #  return rgb_xyz


def write_result_to_file(data, filename):
    data = denormalize_data_from_tanh_space(data)
    #  d = rejoin_channels(data).flatten()

    np.savetxt(filename, data, delimiter="\n", fmt="%1.53f")


def write_result_to_file_bak(data, filename):
    data = denormalize_data_from_tanh_space(data)
    #  d = rejoin_channels(data).flatten()

    np.savetxt(filename, data, delimiter=",", fmt="%1.53f")
