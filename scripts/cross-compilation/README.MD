# **Cross compilation** #

**Build Dependencies**

Parity Ethereum requires latest stable Rust version to build.

We recommend installing Rust through [rustup](https://www.rustup.rs/). If you don't already have rustup, you can install it like this:

`curl https://sh.rustup.rs -sSf | sh`

Parity Ethereum also requires gcc, g++, libudev-dev, pkg-config, file, make, and cmake packages to be installed.
For Debian based distributives:

```
apt-get -y update; 
apt-get install -y --no-install-recommends \
  software-properties-common git \
  make cmake ca-certificates g++ \
  gcc pkg-config libudev-dev \
	libssl-dev libc6-dev
```

*AARCH64 (armv8)*

Install aarch64(armv8) dependencies and tools

```
dpkg --add-architecture arm64;
echo '# source urls for arm64 \n\
deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports/ xenial main \n\
deb-src [arch=arm64] http://ports.ubuntu.com/ubuntu-ports/ xenial main \n\
deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports/ xenial-updates main \n\
deb-src [arch=arm64] http://ports.ubuntu.com/ubuntu-ports/ xenial-updates main \n\
deb [arch=arm64] http://ports.ubuntu.com/ubuntu-ports/ xenial-security main \n\
deb-src [arch=arm64] http://ports.ubuntu.com/ubuntu-ports/ xenial-security main \n # end arm64 section' >> /etc/apt/sources.list;
sed -r 's/deb h/deb \[arch=amd64\] h/g' /etc/apt/sources.list > /tmp/sources-tmp.list;
cp /tmp/sources-tmp.list /etc/apt/sources.list;
sed -r 's/deb-src h/deb-src \[arch=amd64\] h/g' /etc/apt/sources.list > /tmp/sources-tmp.list;
cp /tmp/sources-tmp.list /etc/apt/sources.list;
apt-get -y update;
apt -y install g++-aarch64-linux-gnu gcc-aarch64-linux-gnu libudev-dev libudev-dev:arm64
```

Install aarch64 RUST toolchain

`rustup target add aarch64-unknown-linux-gnu`

Set cross-compiler ENV

```
export CC=aarch64-linux-gnu-gcc; export CXX=aarch64-linux-gnu-g++;
export HOST_CC=gcc; export HOST_CXX=g++;
export LDFLAGS="-L/usr/lib/arm-linux-aarch64"; export CPPFLAGS="-I/usr/include"
```

Copy configuration keys to cargo config

`cp -f scripts/cross-compilation/config .cargo/config`

Build parity-ethereum

`cargo build --target aarch64-unknown-linux-gnu $CARGO_OPTIONS`

*ARMHF (armv7)*

Install armv7 dependencies and tools

```
dpkg --add-architecture armhf;
echo '# source urls for armhf \n\
deb [arch=armhf] http://ports.ubuntu.com/ubuntu-ports/ xenial main \n\
deb-src [arch=armhf] http://ports.ubuntu.com/ubuntu-ports/ xenial main \n\
deb [arch=armhf] http://ports.ubuntu.com/ubuntu-ports/ xenial-updates main \n\
deb-src [arch=armhf] http://ports.ubuntu.com/ubuntu-ports/ xenial-updates main \n\
deb [arch=armhf] http://ports.ubuntu.com/ubuntu-ports/ xenial-security main \n\
deb-src [arch=armhf] http://ports.ubuntu.com/ubuntu-ports/ xenial-security main\
\n # end armhf section' >> /etc/apt/sources.list;
sed -r 's/deb h/deb \[arch=amd64\] h/g' /etc/apt/sources.list > /tmp/sources-tmp.list;
cp /tmp/sources-tmp.list /etc/apt/sources.list;
sed -r 's/deb-src h/deb-src \[arch=amd64\] h/g' /etc/apt/sources.list > /tmp/sources-tmp.list;
cp /tmp/sources-tmp.list /etc/apt/sources.list;
apt-get -y update;
apt-get install -y g++-arm-linux-gnueabihf gcc-arm-linux-gnueabihf libudev-dev libudev-dev:armhf
```

Install armv7 RUST toolchain

`rustup target add armv7-unknown-linux-gnueabihf`

Set cross-compiler ENV

```
export CC=arm-linux-gnueabihf-gcc; export CXX=arm-linux-gnueabihf-g++;
export HOST_CC=gcc; export HOST_CXX=g++;
export LDFLAGS="-L/usr/lib/arm-linix-gnueabihf"; export CPPFLAGS="-I/usr/include"
```

Copy configuration keys to cargo config

`cp -f scripts/cross-compilation/config .cargo/config`

Build parity-ethereum

`cargo build --target armv7-unknown-linux-gnueabihf $CARGO_OPTIONS`

*i386*

Install i386 dependencies and tools

```
dpkg --add-architecture i386;
apt-get -y update;	apt install -y gcc-multilib g++-multilib libudev-dev libudev-dev:i386
```

Install i686 RUST toolchain

`rustup target add i686-unknown-linux-gnu`

Set cross-compiler ENV

```
export CC=gcc; export CXX=g++;
export HOST_CC=gcc; export HOST_CXX=g++;
export LDFLAGS="-L/usr/lib32"; export CPPFLAGS="-I/usr/include"
```

Build parity-ethereum

`cargo build --target i686-unknown-linux-gnu $CARGO_OPTIONS`
