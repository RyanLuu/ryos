#!/bin/bash
#
# Wrapper around qemu-system-riscv64 that sets up a pre-configured machine
# for running ryos or inspecting a device tree dump.

usage()
{
  echo "Usage: $0 [OPTIONS] -- [QEMU_OPTIONS]"
  echo
  echo "Options:"
  echo "  -e, --exec <BINARY>      Run BINARY in QEMU"
  echo "  -d, --dump               Generate a device tree dump"
  echo "  -h, --hard-drive <FILE>  Use FILE as the virtual hard drive"
  echo
  exit 1
}

DUMP=0
HARD_DRIVE=hdd.dsk
declare -a QEMU_ARGS
declare -a POSITIONAL

SHORT=-deh:
LONG=dump,exec:,hard-drive

OPTIONS=$(getopt --options ${SHORT} \
                 --longoptions ${LONG} \
                 --name "$0" \
                 -- "$@")

if [[ $? -ne 0 ]]; then
  usage
fi

eval set -- "${OPTIONS}"

end_of_options=0
while [[ $# -gt 0 ]]; do
  case $1 in
    -d|--dump)
      DUMP=1;;
    -e|--exec)
      shift
      EXEC="$1";;
    -h|--hard-drive)
      shift
      HARD_DRIVE="$1";;
    --)
      end_of_options=1;;
    *)
      if [[ end_of_options -eq 0 ]]; then
        POSITIONAL+=("$1")
      else
        QEMU_ARGS+=("$1")
      fi;;
  esac
  shift
done

set -- "${POSITIONAL[@]}"
if [[ $# -gt 0 ]]; then
  usage
fi


HARD_DRIVE=target/$HARD_DRIVE
if [[ ! -f $HARD_DRIVE ]]; then
  echo "Creating virtual hard drive at $HARD_DRIVE"
	dd if=/dev/zero of=$HARD_DRIVE bs=1M count=32
fi

MACH="virt"
CPUS=4
MEM="128M"
QEMU_FLAGS="-machine ${MACH} -cpu rv64 -smp ${CPUS} -m ${MEM} -nographic -serial mon:stdio -bios none -drive if=none,format=raw,file=${HARD_DRIVE},id=x0 -device virtio-blk-device,scsi=off,drive=x0"

if [[ $DUMP -eq 1 ]]; then
  mkdir -p target/dump
  QEMU_FLAGS+=" -machine dumpdtb=target/dump/riscv64-virt.dtb"
fi

if [[ -n $EXEC ]]; then
  QEMU_FLAGS+=" -kernel $EXEC"
fi

if [[ -n $QEMU_ARGS ]]; then
  QEMU_FLAGS += " ${QEMU_ARGS[@]}"
fi

qemu-system-riscv64 ${QEMU_FLAGS}

if [[ $DUMP -eq 1 ]]; then
  dtc -I dtb -O dts -o target/dump/riscv64-virt.dts target/dump/riscv64-virt.dtb
fi

