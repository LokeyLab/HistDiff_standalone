#!/usr/bin/env bash

# time python histDiff.py

#time python /home/derfelt/git_repos/HistDiffPipeline/pipelineCore/histdiff_pipelineV2.py -i "/home/derfelt/git_repos/HistDiff_standalone/temp_store/cellbycell/ff1301b8-94c2-11ee-ac86-02420a000112_cellbycell.tsv" -o "/home/derfelt/git_repos/HistDiff_standalone/temp_store/hd2.csv" -c "/home/derfelt/git_repos/HistDiff_standalone/temp_store/platemaps/SP7238PMA.csv" -cs; 

time python histDiff.py -i "~/git_repos/HistDiff_standalone/temp_store/cellbycell/output.tsv" -o "~/git_repos/HistDiff_standalone/temp_store/official_run.csv" -c "~/git_repos/HistDiff_standalone/temp_store/platemaps/SP7238PMA.csv" -ref "sample_type" -wells "384_Well" -id "id" 
