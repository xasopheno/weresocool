set -e

CURRENT_DIR="$(dirname "$0")"
cd $CURRENT_DIR

echo "Preparing weresocool-mac-${1}.tar.gz"
TAGNAME=$1
FILENAME=weresocool-mac-${TAGNAME}.tar.gz

just build-release
cd ../target/release
tar -czf "${FILENAME}" weresocool
