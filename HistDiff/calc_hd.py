#!/usr/bin/env python

import sys
from itertools import product

import numpy as np
import pandas as pd

from HistDiff.histograms import *
from HistDiff.utils import *


def cleanWellNameList(x: list) -> list:
    return ["".join([i[0], str(int(i[1:]))]) for i in x]


def calcHistDiffScore(
    cellData_file: str,
    idFeats: list[str] | str,
    dtypes: dict,
    vehicleCntrlWells: tuple,
    nbins: int = 20,
    chunksize: int = 50000,
    probOut: str = None,
    blockDef: tuple = tuple(),
    plateDef: list[str] | None = None,
) -> pd.DataFrame:

    plateDef = (
        ["".join([x, y]) for x, y in product(WELL_384_LETTERS, WELL_384_NUMBERS)]
        if plateDef is None
        else plateDef
    )

    chunks = pd.read_table(
        cellData_file, chunksize=chunksize, dtype=dtypes, on_bad_lines="skip"
    )

    min_max, features, _ = getMinMaxPlate(chunks=chunks, id_col=idFeats, verbose=False)

    dtypes = create_dtypes(
        headers=features,
        meta_feats=list(idFeats) if not isinstance(idFeats, list) else idFeats,
    )

    chunks = pd.read_table(
        cellData_file,
        chunksize=chunksize,
        dtype=dtypes,
        on_bad_lines="skip",
    )

    hist_df: pd.DataFrame = pd.DataFrame()
    for count, chunk in enumerate(chunks):
        currDf: pd.DataFrame = chunk
        currDf.set_index(idFeats, inplace=True)
        currDf = currDf[features]
        currDf = currDf[currDf.index.isin(plateDef)]

        wells_used = currDf.index.unique()

        # first iter
        if count == 0:
            hist_df = pd.concat(
                [
                    createHistRow(min_max=min_max, nbins=nbins)
                    for _ in range(len(wells_used))
                ]
            )
            hist_df.index = wells_used

        unencountered_wells = set(wells_used) - set(hist_df.index)
        if len(unencountered_wells) > 0:
            hist_df_temp: pd.DataFrame = pd.concat(
                [
                    createHistRow(min_max, nbins=nbins)
                    for _ in range(len(unencountered_wells))
                ]
            )
            hist_df_temp.index = unencountered_wells
            hist_df = pd.concat([hist_df, hist_df_temp])

        for well in wells_used:
            well_chunk = currDf[currDf.index == well]
            hist_df[hist_df.index == well].apply(
                lambda x: x[well].fill(well_chunk.loc[:, x.name])
            )

    # Unpacking the histograms
    print(f"Unpacking histograms", file=sys.stderr)
    hist_df = hist_df.map(lambda x: x.hist)

    # well384 = ["".join([x, y]) for x, y in product(WELL_384_LETTERS, WELL_384_NUMBERS)]
    well384 = plateDef
    if len(blockDef) == 0:
        blockDef = (well384,)
    else:
        undefinedBlocks = set().union(*(cleanWellNameList(i) for i in blockDef))
        undefinedBlocks = set(well384) - undefinedBlocks
        blockDef = blockDef + (list(undefinedBlocks),)

    aggDf = []
    for group in blockDef:
        select_wells = set(["".join([i[0], i[1:]]) for i in group])

        hd_group = hist_df[hist_df.index.isin(select_wells)]

        print(select_wells, group, file=sys.stderr)
        print(hd_group.shape, file=sys.stderr)

        # control wells for group and sum all histograms from each vehicle control well
        vehicle_controls = list(set(vehicleCntrlWells).intersection(hd_group.index))
        vehicle_control_df = (
            hd_group[hd_group.index.isin(vehicle_controls)].to_numpy().sum(axis=0)
        )
        vehicle_control_df = pd.DataFrame(
            vehicle_control_df, index=hd_group.columns, columns=["VEHICLE_CONTROL"]
        ).T

        # Add controls
        print("Adding vehicle controls to table", file=sys.stderr)
        hd_group = pd.concat([hd_group, vehicle_control_df])

        # Smooth the hists
        print("Smoothing histograms", file=sys.stderr)
        smoothed_hists = hd_group.map(lambda x: exponentialSmoothing(x, alpha=0.25))

        # normalize hists
        normalized_hists = smoothed_hists.map(normalize)

        hds = []
        for feat in features:
            control = np.array(normalized_hists.loc["VEHICLE_CONTROL", feat])

            experimental = np.transpose(np.array(normalized_hists[feat].to_list()))

            # WARNING: delete the below
            # if feat == "Nuclei-Cell_Region_Alexa_488_(global)_Axial_Length_Ratio":
            #     print(experimental, experimental.shape)

            hd = HistSquareDiff(exp=experimental, ctrl=control)

            hds.append(hd)
        histdiff_df = pd.DataFrame(hds, index=features, columns=hd_group.index).T

        # remove controls
        print("Dropping controls", file=sys.stderr)

        histdiff_df = histdiff_df.drop("VEHICLE_CONTROL")
        aggDf.append(histdiff_df)

    return pd.concat(aggDf, axis=0)
