#!/usr/bin/env python
import sys

import numpy as np
import pandas as pd

from HistDiff.calc_hd import calcHistDiffScore, create_dtypes


def main():

    # cell_by_cell = "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/ff1301b8-94c2-11ee-ac86-02420a000112_cellbycell.tsv"
    cell_by_cell = (
        "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/output.tsv"
    )

    plate_map = (
        "/home/derfelt/git_repos/HistDiff_standalone/temp_store/platemaps/SP7238PMA.csv"
    )
    plate_map = pd.read_csv(plate_map)
    print(plate_map)
    controls = plate_map[plate_map["sample_type"] == "REFERENCE"]["384_Well"].to_list()
    controls = ["".join([x[0], str(int(x[1:]))]) for x in controls]
    print(len(controls))
    # cell_by_cell = pd.read_csv(cell_by_cell, sep="\t", chunksize=50000)
    #
    # min_max, _, _ = getMinMaxPlate(chunks=cell_by_cell, id_col="id")
    # print(min_max.head(5))
    header = pd.read_table(cell_by_cell, nrows=0).columns.to_list()
    dtypes = create_dtypes(headers=header, meta_feats=["id"])

    df = calcHistDiffScore(
        cellData_file=cell_by_cell,
        idFeats="id",
        dtypes=dtypes,
        vehicleCntrlWells=tuple(controls),
    )

    print(df)

    df.to_csv("./temp_store/test.csv")


if __name__ == "__main__":
    main()
