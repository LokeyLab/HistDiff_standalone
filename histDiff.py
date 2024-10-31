#!/usr/bin/env python
import argparse
import sys
from itertools import product

import numpy as np
import pandas as pd

from HistDiff.calc_hd import calcHistDiffScore, create_dtypes
from HistDiff.utils import WELL_384_LETTERS, WELL_384_NUMBERS


class CommandLine:
    def __init__(self, inOpts: list[str] | None = None) -> None:
        self.parser = argparse.ArgumentParser(
            prog="histDiff.py",
            description="A proper standalone CLI program for the HistDiff algorithm. NOTE: input cell by cell data is under the assumption that is properly formatted as: 1 index column, and the rest are numerical features. If you are using Signals and you need your cell by cell data formatted correctly please use formatSignals.py",
            usage="python %(prog)s -[flag|argument]",
            add_help=True,
            prefix_chars="-",
        )

        self.parser.add_argument(
            "-i",
            "--input",
            action="store",
            nargs="?",
            required=True,
            help="input for cell data as .tsv or tab file",
        )
        self.parser.add_argument(
            "-o",
            "--output",
            action="store",
            nargs="?",
            required=True,
            help="name and path of resulting csv file (must end in .csv)",
        )
        self.parser.add_argument(
            "-c",
            "--controlsFile",
            action="store",
            nargs="?",
            required=True,
            help="The platemap/file containing locations of reference and experimental wells (must be .csv file)",
        )
        self.parser.add_argument(
            "-ref",
            "--referenceColumn",
            action="store",
            nargs="?",
            required=True,
            help="Specify the name of the column where the REFERENCE labels are located (note: this column must contains cells with the label: REFERENCE as this specifies which well are the reference controls for HistDiff)",
        )
        self.parser.add_argument(
            "-wells",
            "--wellLocation",
            action="store",
            nargs="?",
            required=True,
            help="Specify the name of the column containing the well locations",
        )

        self.parser.add_argument(
            "-id",
            "--indexColumn",
            action="store",
            nargs="?",
            required=True,
            help="Specify the name of the index column of the cell by cell data file (This is under the assumption that the rest of the columns are numerical features. Note: program will break if there is additional meta columns not taken care of.)",
        )

        if inOpts is None:
            self.args = self.parser.parse_args()
        else:
            self.args = self.parser.parse_args(inOpts)


# TODO: Implement block recalculation
def main():
    cl = CommandLine()
    # cell_by_cell = "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/ff1301b8-94c2-11ee-ac86-02420a000112_cellbycell.tsv"
    # cell_by_cell = (
    #     "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/output.tsv"
    # )
    #
    # plate_map = (
    #     "/home/derfelt/git_repos/HistDiff_standalone/temp_store/platemaps/SP7238PMA.csv"
    # )

    cell_by_cell = cl.args.input

    plate_map = cl.args.controlsFile
    plate_map = pd.read_csv(plate_map)

    controls = plate_map[plate_map[cl.args.referenceColumn].str.upper() == "REFERENCE"][
        cl.args.wellLocation
    ].to_list()
    controls = ["".join([x[0], str(int(x[1:]))]) for x in controls]

    plate_definition = plate_map[cl.args.wellLocation].to_list()
    plate_definition = ["".join([x[0], str(int(x[1:]))]) for x in plate_definition]

    header = pd.read_table(cell_by_cell, nrows=0).columns.to_list()
    dtypes = create_dtypes(headers=header, meta_feats=["id"])

    df = calcHistDiffScore(
        cellData_file=cell_by_cell,
        idFeats="id",
        dtypes=dtypes,
        vehicleCntrlWells=tuple(controls),
        plateDef=plate_definition,
    )

    df.to_csv(cl.args.output)


if __name__ == "__main__":
    main()
