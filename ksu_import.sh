#!/bin/bash

echo Importing KSU...

KSU_DIR=drivers/staging/kernelsu
if [ -d "$KSU_DIR" ];
then
    echo "$KSU_DIR directory exists. Removing it!"
    rm -rf drivers/staging/kernelsu
    echo Commit changes and re-run the script
else
    if git ls-remote --exit-code kernelsu; 
    then
    	git read-tree --prefix=drivers/staging/kernelsu/ -u kernelsu/main
    else
    	git remote add kernelsu https://github.com/tiann/KernelSU.git
	git fetch kernelsu
	git read-tree --prefix=drivers/staging/kernelsu/ -u kernelsu/main
    fi
    echo Done. Now commit changes.
fi
