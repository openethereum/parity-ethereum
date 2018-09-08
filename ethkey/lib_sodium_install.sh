#Tested on mac osx only
#This is by no means production ready. I'm just including it for any one who wants to use it
#Ideally you should download and install libsodium  https://download.libsodium.org/doc/
#and point the environment variables, SODIUM_INCL_DIR, SODIUM_LIB_DIR, SODIUM_STATIC_DIR



TARBALL="https://download.libsodium.org/libsodium/releases/LATEST.tar.gz"
DIR=${PWD}/deps
BUILD=${PWD}/build



if ! ls ${DIR}/lib/libsodium* 1> /dev/null 2>&1; then
	if ! ls /usr/local/lib/libsodium* 1> /dev/null 2>&1; then ###  

		set -e
		mkdir $DIR
		mkdir $BUILD && cd $BUILD

		#download and compile libsodium
		curl -L $TARBALL| tar zx

		$PWD/libsodium-stable/configure --prefix=$DIR

		make && make install && make clean
		set +e
		
		test -d $DIR/include && test -d $DIR/lib || return 11

		cd ${DIR}/..
		rm -rf $BUILD
	else
		DIR=/usr/local
		echo "Found libsodium in ${DIR}"
	fi

else
	test -f $DIR/include/sodium.h || echo "I encountered an issue with your libsodium installation.\n try removing the 'deps' directory and re-running this script" | return 1
fi	
 
LIB=$DIR/lib
INC=$DIR/include


export SODIUM_LIB_DIR=$LIB
export SODIUM_INC_DIR=$INC
export SODIUM_STATIC_DIR=$LIB

unset DIR TARBALL BUILD

test -f $SODIUM_INC_DIR/sodium.h && test -f $SODIUM_LIB_DIR/libsodium.a && echo "Done"
