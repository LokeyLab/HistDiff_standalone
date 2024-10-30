#!/usr/bin/env python
import numpy as np

WELL_384_LETTERS = [i for i in ascii_uppercase[:16]]
WELL_384_NUMBERS = [str(i) for i in range(1, 25)]


def exponentialSmoothing(x, alpha=0.25):
    """

    :param x:
    :param alpha:
    :return:
    """
    n = len(x)
    s = list()
    for i, x_i in enumerate(x):
        if i == 0:
            s.append(x_i + alpha * (x[i + 1] - x_i))
        elif i == n - 1:
            s.append(alpha * (x[i - 1] - x_i) + x_i)
        else:
            s.append(alpha * (x[i - 1] - x_i) + x_i + alpha * (x[i + 1] - x_i))
    return np.array(s)


def normalize(x):
    """generic normalize histogram by sum of all bins function"""
    # TODO: In numpy 1.23.5 I had to extend the where part of the function to the length of x for it to work
    #  It would run with the simple x.sum() != 0 statement before, but now throws a np.putmask-related error.
    #  This might be a specific Mac issue for whatever reason, since this was not encountered on Windows
    return np.divide(
        x,
        x.sum(),
        out=np.zeros_like(x, dtype="longdouble"),
        where=np.repeat(x.sum(), len(x)) != 0,
    )
