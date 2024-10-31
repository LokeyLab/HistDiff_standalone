#!/usr/bin/env bash

python histDiff.py

python /home/derfelt/git_repos/HistDiffPipeline/pipelineCore/histdiff_pipelineV2.py -i "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/ff1301b8-94c2-11ee-ac86-02420a000112_cellbycell.tsv" -o "/home/derfelt/git_repos/HistDiff_standalone/temp_store/hd2.csv" -c "/home/derfelt/git_repos/HistDiff_standalone/temp_store/platemaps/SP7238PMA.csv" -cs; 
