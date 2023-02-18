#!/bin/bash
rm .version
# Bash Color
green='\033[01;32m'
red='\033[01;31m'
blink_red='\033[05;31m'
restore='\033[0m'

clear

# Resources
export CLANG_PATH=~/pixelos/prebuilts/clang/host/linux-x86/clang-playground/bin
export PATH=${CLANG_PATH}:${PATH}
export CROSS_COMPILE=${CLANG_PATH}/aarch64-linux-gnu-
export CROSS_COMPILE_ARM32=${CLANG_PATH}/arm-linux-gnueabi-
DEFCONFIG="cepheus_defconfig"

# Kernel Details
VER=""

# Paths
KERNEL_DIR=`pwd`
REPACK_DIR=~/anykernel
ZIP_MOVE=~/TREES

# Functions
function clean_all {
		rm -rf $REPACK_DIR/Image*
		cd $KERNEL_DIR
		echo
		make clean && make mrproper
}

function make_kernel {
		echo
		make LLVM=1 LLVM_IAS=1 $DEFCONFIG
		make LLVM=1 LLVM_IAS=1 -j$(grep -c ^processor /proc/cpuinfo)

}


function make_boot {
		cp out/arch/arm64/boot/Image.gz-dtb $REPACK_DIR
}


function make_zip {
		cd $REPACK_DIR
		zip -r9 `echo $ZIP_NAME`.zip *
		mv  `echo $ZIP_NAME`*.zip $ZIP_MOVE
		cd $KERNEL_DIR
}


DATE_START=$(date +"%s")


echo -e "${green}"
echo "Making Kernel:"
echo -e "${restore}"


# Vars
BASE_AK_VER="POST-SOVIET-MI9-"
DATE=`date +"%Y%m%d-%H%M"`
AK_VER="$BASE_AK_VER$VER"
ZIP_NAME="$AK_VER"-"$DATE"
#export LOCALVERSION=~`echo $AK_VER`
#export LOCALVERSION=~`echo $AK_VER`
export ARCH=arm64
export SUBARCH=arm64
export KBUILD_BUILD_USER=balgxmr
export KBUILD_BUILD_HOST=balgxmr

echo
echo Starting cleaning...
echo
clean_all
echo
echo All cleaned.
echo
echo Building kernel...
echo

make_kernel
make_boot
make_zip

DATE_END=$(date +"%s")
DIFF=$(($DATE_END - $DATE_START))

echo
echo -e "${green}"
find $ZIP_MOVE -type f -printf "%p\n" | sort -n | tail -1
echo "### build completed in ($(($DIFF / 60)):$(($DIFF % 60)) (mm:ss))."
echo -e "${restore}"
echo
