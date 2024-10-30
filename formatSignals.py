#!/usr/bin/env python
"""
This program is to format Signals Cell By Cell
data before entering HistDiff
"""
import argparse as argp
import sys

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


def majority_numeric(row: list, threshold=0.65):
    """
    Check if a row is mostly made up of floats, if it is
    then it is most likely a valid row
    """
    min_nfloats = int(threshold * len(row))  # make sure n number of cells are floats

    float_count = 0
    for i, value in enumerate(row):
        try:
            float(value)
            float_count += 1

            if float_count >= min_nfloats:
                return True
        except ValueError:
            pass

    return False


def integrity_check(
    inFile: str, outFile: str, header_len: int, buffer_size: int = 5000
) -> tuple[int, int]:
    """
    Check and clean up the cell by cell data
    """
    n_rows_before = 0
    n_rows_after = 0
    buffer = []

    with open(inFile, "r") as input_file, open(outFile, "w") as output_File:

        header = input_file.readline()
        output_File.write(header)

        for line in input_file:
            n_rows_before += 1
            row = line.strip().split("\t")

            if (len(row) == header_len) and (majority_numeric(row=row)):
                buffer.append(line)
                n_rows_after += 1

                if len(buffer) >= buffer_size:
                    output_File.writelines(buffer)
                    buffer.clear()  # reset buffer

        if buffer:
            output_File.writelines(buffer)

    return n_rows_before, n_rows_after


def cleanColNames(colName):
    return (
        colName.strip()
        .replace("\t", ",")
        .replace("%", "Pct")
        .replace(" - ", "-")
        .replace(" ", "_")
        .replace("µ", "u")
        .replace("²", "^2")
        .replace("_(RAWcells-CP2-Cyto_BMR)", "")
        .replace("_(RAWcells-CP2-EdU_BMR)", "")
    )


def preProcessChunk(
    chunk: pd.DataFrame, id_col: list[str] | str, useless_features: list[str]
) -> pd.DataFrame:
    """Correctly format the datframe"""

    def row(x):
        strs = map(str, x[id_col])
        return "_".join(strs)

    chunk["id"] = (
        chunk.apply(lambda x: row(x), axis=1) if len(id_col) > 1 else chunk[id_col]
    )
    chunk.set_index("id", inplace=True)
    chunk.drop(useless_features, axis=1, inplace=True)
    chunk.drop(id_col, axis=1, inplace=True)
    chunk.rename(columns=lambda x: cleanColNames(x), inplace=True)
    chunk.rename(index=lambda x: "".join([x[0], str(int(x[1:]))]), inplace=True)

    feature_verification = [feat for feat in chunk.columns if feat in useless_features]
    if len(feature_verification) > 0:
        print("Useless features still exist", file=sys.stderr)
        print(f"{feature_verification}")
        raise ValueError

    return chunk


def preProcessData(
    input_file: str,
    output_file: str,
    id_col: list[str] | str,
    useless_features: list[str],
    dtypes: dict,
    chunksize=10000,
) -> None:
    """Process large tab files"""

    with pd.read_table(input_file, chunksize=chunksize, dtype=dtypes) as reader:
        for i, chunk in enumerate(reader):

            processed_chunk = preProcessChunk(
                chunk=chunk, id_col=id_col, useless_features=useless_features
            )
            processed_chunk.to_csv(output_file, sep="\t", mode="a", header=(i == 0))


class CommandLine:
    def __init__(self, inOpts: list[str] | None = None) -> None:
        self.parser = argp.ArgumentParser(
            prog="formatSignals.py",
            description="An optional command line program that formats cell by cell data from Signals correctly",
            usage="python &(prog)s -[arguments] [value]",
            add_help=True,
            prefix_chars="-",
        )

        self.parser.add_argument(
            "-i",
            "--input",
            action="store",
            nargs="?",
            required=True,
            type=str,
            help="input data as .tsv",
        )
        self.parser.add_argument(
            "-o",
            "--output",
            action="store",
            nargs="?",
            required=True,
            type=str,
            help="output path and file as .tsv",
        )
        self.parser.add_argument(
            "-if",
            "--integrityFile",
            required=False,
            default=None,
            nargs="?",
            help="If used, then it will do an integrity check and clean up the file for proper use",
        )

        if inOpts is None:
            self.args = self.parser.parse_args()
        else:
            self.args = self.parser.parse_args(inOpts)


def main():

    cl = CommandLine()
    # file = "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/ff1301b8-94c2-11ee-ac86-02420a000112_cellbycell.tsv"
    file = cl.args.input

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
    useless_features = list(set(common_metaCols) - set(id_col))
    # print(useless_features, common_metaCols)

    header_len = len(headers)

    # temp_out = "./temp_store/cellbycell/test.tsv"
    temp_out = cl.args.integrityFile

    if temp_out is not None:
        n_rows_before, n_rows_after = integrity_check(
            inFile=file, outFile=temp_out, header_len=header_len
        )
        # print(n_rows_before, n_rows_after)

    preProcessData(
        input_file=file if temp_out is None else temp_out,
        output_file=cl.args.output,
        id_col=id_col,
        useless_features=useless_features,
        dtypes=df_dtypes,
        chunksize=50000,
    )

    test_df = pd.read_table(cl.args.output, nrows=1)
    print(test_df.shape)


if __name__ == "__main__":
    main()
