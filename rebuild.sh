#!/usr/bin/env bash
# Everything you push to main will do a test build, and let you know if it breaks.
#
# Things only get released if you tag it. And the actual build is based on the tag.
# Without tagging it, nothing is released and it doesn't affect anyone at all, aside
# from people building it from source.
#
# Look at the list of tags:
#
# https://github.com/betrusted-io/rust/tags
#
# We increment the 4th decimal. So far with the 1.59.0 branch, we've had two releases: 1.59.0.1 and 1.59.0.2. If you decided to release a new version of libstd, you would do:
#
# git tag -a 1.59.0.3 # Commit a message, indicating what you've changed
# git push --tags
#
# That would build and release a new version.

set -e
set -u
# set -x
set -o pipefail

rust_sysroot=$(rustc --print sysroot)

export RUST_COMPILER_RT_ROOT="$(pwd)/src/llvm-project/compiler-rt"
export CARGO_PROFILE_RELEASE_DEBUG=2
export CARGO_PROFILE_RELEASE_OPT_LEVEL="3"
export CARGO_PROFILE_RELEASE_DEBUG_ASSERTIONS="false"
export RUSTC_BOOTSTRAP=1
export TARGET=riscv32imac-unknown-xous-elf
export RUSTFLAGS='-Csymbol-mangling-version=legacy -Zunstable-options -Zmacro-backtrace -Clink-args=-Wl,-z,origin -Clink-args=-Wl,-rpath,$ORIGIN/../lib -Csplit-debuginfo=off -Cprefer-dynamic -Cllvm-args=-import-instr-limit=10 -Zinline-mir -Cembed-bitcode=yes -Clto=off -Cforce-unwind-tables=yes'
export __CARGO_DEFAULT_LIB_METADATA="stablestd"
export CFLAGS_riscv32imac_unknown_xous_elf='-ffunction-sections -fdata-sections -fPIC -march=rv32imac -mabi=ilp32 -mcmodel=medany'
export CXXFLAGS_riscv32imac_unknown_xous_elf='-ffunction-sections -fdata-sections -fPIC -march=rv32imac -mabi=ilp32 -mcmodel=medany'

command_exists() {
    which $1 &> /dev/null && $1 --version 2>&1 > /dev/null
}

# Set up the C compiler. We need to explicitly specify these variables
# because the `cc` package obviously doesn't recognize our target triple.
if command_exists riscv32-unknown-elf-gcc
then
    export CC="riscv32-unknown-elf-gcc"
    export AR="riscv32-unknown-elf-ar"
elif command_exists riscv-none-embed-gcc
then
    export CC ="riscv-none-embed-gcc"
    export AR ="riscv-none-embed-ar"
elif command_exists riscv64-unknown-elf-gcc
then
    export CC="riscv64-unknown-elf-gcc"
    export AR="riscv64-unknown-elf-ar"
else
    echo "No C compiler found for riscv" 1>&2
    exit 1
fi

src_path="./target/$TARGET/release/deps"
dest_path="$rust_sysroot/lib/rustlib/$TARGET"
dest_lib_path="$dest_path/lib"
# function Get-ItemBaseName {
#     param ($ItemName)
#     # Write-Host "Item name: $ItemName"
#     $sub_strings = $ItemName -split "-"
#     $last_string_count = $sub_strings.Count
#     $ItemName -replace "-$($sub_strings[$last_string_count-1])", ""
#     # return $result
# }

mkdir -p $dest_lib_path

rustc --version | awk '{print $2}' > "$dest_path/RUST_VERSION"

# Remove stale objects
rm -f $dest_lib_path/*.rlib

# TODO: Use below to remove duplicates
# previous_libraries=$(ls -1 $src_path/*.rlib)

cargo build \
    --target $TARGET \
    -Zbinary-dep-depinfo \
    --release \
    --features "panic-unwind llvm-libunwind backtrace compiler-builtins-c compiler-builtins-mem" \
    --manifest-path "library/sysroot/Cargo.toml" || exit 1

# TODO: Remove duplicates here by comparing it with $previous_libraries
for new_item in $(ls -1 $src_path/*.rlib)
do
    file=$(basename $new_item)
    base_string=$(echo $file | rev | cut -d- -f2- | rev)
done

cp $src_path/*.rlib "$dest_lib_path"
