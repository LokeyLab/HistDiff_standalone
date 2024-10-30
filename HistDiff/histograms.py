#!/usr/bin/env python
import numpy as np


class Hist1D(object):
    """taken from https://stackoverflow.com/a/45092548"""

    def __init__(self, nbins, xlow, xhigh):
        self.nbins = nbins
        self.xlow = xlow
        self.xhigh = xhigh
        self.hist, edges = np.histogram([], bins=nbins, range=(xlow, xhigh))
        self.bins = (edges[:-1] + edges[1:]) / 2.0

    def fill(self, arr):
        hist, edges = np.histogram(arr, bins=self.nbins, range=(self.xlow, self.xhigh))
        self.hist += hist

    @property
    def data(self):
        return self.bins, self.hist


def HistSquareDiff(exp, ctrl, factor=1):
    """The actual workhorse HistDiff scoring function"""

    # we transpose twice to ensure the final result is a 1 x m feature score vector
    ctrl_meanProxy = (np.arange(1, ctrl.shape[0] + 1) * ctrl.T).T.sum(axis=0)
    exp_meanProxy = (np.arange(1, exp.shape[0] + 1) * exp.T).T.sum(axis=0)

    # evaluate where and when to adjust the score to be negative
    negScore = np.where(ctrl_meanProxy > exp_meanProxy, -1, 1)
    diff = ctrl - (exp.T * factor)
    diff **= 2

    return diff.sum(axis=1) * negScore
