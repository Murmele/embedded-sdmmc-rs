#!/bin/bash

mkdir -p DISK1_MOUNT
sudo losetup -o 135266304 /dev/loop0 DISK1.img
sudo mount /dev/loop0 DISK1_MOUNT
