#!/bin/sh

echo "#################################################################################"
echo "THIS IMAGE IS DEPRECATED."
echo ""
echo "At some point in the future we will stop pushing new versions of 'parity/parity'."
echo "Please use 'parity/ethereum' image instead, it is the new canonical location for "
echo "'parity/parity' images."
echo ""
echo "We maintain the same set of tags there, so just replacing the image name"
echo "everywhere would be safe."
echo "#################################################################################"

exec /bin/parity $@
