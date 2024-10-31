#!/usr/bin/env python
import sys

import numpy as np
import pandas as pd
from pandas.io.parsers.readers import TextFileReader


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


def getMinMaxPlate(
    chunks: TextFileReader, id_col: list[str] | str, verbose=True, probOut=None
) -> tuple[pd.DataFrame, list[str], pd.DataFrame]:
    """Gets the min and max of the features and returns features that are useful"""
    xlow = []
    xHigh = []

    feats: list[str] = []

    for count, chunk in enumerate(chunks, start=1):
        currDf = chunk
        currDf.set_index(id_col, inplace=True)
        currDf = currDf.replace(to_replace=-np.inf, value=np.nan)
        currDf = currDf.replace(to_replace=np.inf, value=np.nan)

        if count == 1:
            feats = currDf.columns.to_list()

        xlow.append(currDf.min(axis=0).to_list())
        xHigh.append(currDf.max(axis=0).to_list())

    xlow = pd.DataFrame(xlow).min(axis=0)
    xhigh = pd.DataFrame(xHigh).max(axis=0)

    # adjusting the high ranges
    xhigh[xhigh == xlow] = xlow[xhigh == xlow] + xlow[xhigh == xlow] * 0.5
    xhigh[xhigh == xlow] = xlow[xhigh == xlow] + 1

    min_max = pd.DataFrame(
        {"xlow": xlow.to_list(), "xhigh": xhigh.to_list()}, index=feats
    )

    bad_features = {
        feature: "noValues"
        for feature in min_max[
            min_max.apply(lambda x: all(np.isnan(x)), axis=1)
        ].index.values.tolist()
    }
    problematic_features_df = None
    if bad_features and verbose:
        print(
            f"MinMax: No values have been found in the following features: "
            f'{" | ".join(bad_features.keys())}',
            file=sys.stderr,
        )
        problematic_features_df = pd.Series(bad_features, name="histdiff_issue")
        if probOut is not None:
            problematic_features_df.to_csv(f"{probOut}_problematicFeats.csv")

    # Get good features and min_max table
    min_max = min_max[~min_max.apply(lambda x: all(np.isnan(x)), axis=1)]
    good_features = min_max.index.values.tolist()
    print(f"length of good features is: {len(good_features)}", file=sys.stderr)

    return (min_max, good_features, problematic_features_df)
