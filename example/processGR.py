#!/usr/bin/env python
import os
import sys
from glob import glob
from subprocess import run

import pandas as pd

sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), "..")))

from HistDiff.calc_hd import calcHistDiffScore, create_dtypes

print(sys.path, file=sys.stderr)


def convertXlSX_to_CSV(dir: str, outDir: str):
    for file in glob(os.path.join(dir, "*.xlsx")):
        base_n = os.path.splitext(os.path.basename(file))[0]
        final_out = os.path.join(outDir, f"{base_n}.csv")

        print(final_out)
        df = pd.read_excel(file)
        df.to_csv(final_out, index=False)


def main():
    cell_by_cellData = "/home/derfelt/LokeyLabFiles/TargetMol/GR_followup/dataset/cell_by_cell_data/024ebc52-9579-11ef-b032-02420a00010f_cellbycell_HD_input.tsv"

    outDir = "/home/derfelt/LokeyLabFiles/TargetMol/GR_followup/dataset/HD_outs"
    platemap_dir = "/home/derfelt/LokeyLabFiles/TargetMol/GR_followup/dataset/platemaps"
    csv_platemap_dir = "/home/derfelt/LokeyLabFiles/TargetMol/GR_followup/dataset/platemaps/csv_converted/"

    # convertXlSX_to_CSV(
    #     dir=platemap_dir,
    #     outDir=csv_platemap_dir,
    # )
    script = "/home/derfelt/git_repos/HistDiff_standalone/histDiff.py"
    for platemap in glob(os.path.join(csv_platemap_dir, "*.csv")):
        base_n = os.path.splitext(os.path.basename(platemap))[0]
        final_out = os.path.join(outDir, f"{base_n}.csv")

        args = lambda x, y: [
            "-i",
            cell_by_cellData,
            "-c",
            x,
            "-o",
            y,
            "-ref",
            "sample_type",
            "-wells",
            "384_Well",
            "-id",
            "id",
        ]
        run(["python", script] + args(x=platemap, y=final_out))

        print(final_out)


if __name__ == "__main__":
    main()
