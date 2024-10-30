#!/usr/bin/env python
import numpy as np
import pandas as pd


def main():

    cell_by_cell = "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/ff1301b8-94c2-11ee-ac86-02420a000112_cellbycell.tsv"
    cell_by_cell = pd.read_csv(cell_by_cell, sep="\t", nrows=100)

    print(*cell_by_cell.columns, sep="\n")


if __name__ == "__main__":
    main()
