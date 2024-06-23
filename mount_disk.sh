#!/bin/bash

mkdir -p DISK1_MOUNT
# fdisk -lu DISK1.img
sudo losetup -o 135266304 /dev/loop0 DISK1.img
sudo mount /dev/loop0 DISK1_MOUNT
