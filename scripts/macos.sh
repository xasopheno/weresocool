set -e

CURRENT_DIR="$(dirname "$0")"
echo "${CURRENT_DIR}"

just build-release
ls
cd target/release
tar -czf weresocool-mac .
ar.gz weresocool
shasum -a 256 weresocool-mac.tar.gz
