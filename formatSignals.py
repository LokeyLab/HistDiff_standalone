#!/usr/bin/env python
"""
This program is to format Signals Cell By Cell
data before entering HistDiff
"""
# import numpy as np
import pandas as pd


def findCommonFeats(trueFeats: list, badFeats: list) -> list:
    commonFeats = list(set(trueFeats) & set(badFeats))
    for feat in trueFeats:
        if any([i in feat for i in badFeats if len(i) > 2]) and feat not in commonFeats:
            commonFeats.append(feat)
        if (
            any([i == feat for i in badFeats if len(i) <= 2])
            and feat not in commonFeats
        ):  # check for the small substrings like 'X' or 'Y'
            commonFeats.append(feat)

    # print(*commonFeats, sep="\n", file=sys.stderr)
    return commonFeats


def create_dtypes(headers: list, meta_feats: list | None) -> dict:
    """
    Define th datatypes for each column in the dataset
    """
    if meta_feats is not None:
        dtypes = {feat: str for feat in meta_feats}
        dtypes.update({feat: float for feat in headers if feat not in meta_feats})
    else:
        dtypes = {feat: float for feat in headers}

    return dtypes


def main():
    file = "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/ff1301b8-94c2-11ee-ac86-02420a000112_cellbycell.tsv"
    headers = pd.read_table(file, nrows=0).columns.to_list()

    meta_cols = [
        "ScreenName",
        "ScreenID",
        "PlateName",
        "PlateID",
        "MeasurementDate",
        "MeasurementID",
        "WellName",
        "Row",
        "Column",
        "Timepoint",
        "Field",
        "RefId",
        "Object Number",
        "X",
        "Y",
        "Bounding Box",
        "ax",
        "ay",
        "Cell Count",
        "Cell ID",
        "Instance",
        "Laser focus score",
        "Plate ID",
        "Run Settings ID",
        "Series ID",
        "Site ID",
        "Well Name",
        "Well X",
        "Well Y",
    ]

    common_metaCols = findCommonFeats(trueFeats=headers, badFeats=meta_cols)
    df_dtypes = create_dtypes(headers=headers, meta_feats=meta_cols)

    id_col = ["WellName"] if "WellName" in common_metaCols else ["Well Name"]


if __name__ == "__main__":
    main()
