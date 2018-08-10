FROM ubuntu:xenial
LABEL maintainer="Parity Technologies <devops@parity.io>"

RUN apt-get update && \
    apt-get install -yq sudo curl file build-essential wget git g++ cmake pkg-config bison flex \
                        unzip lib32stdc++6 lib32z1 python autotools-dev automake autoconf libtool \
                        gperf xsltproc docbook-xsl

# Rust & Cargo
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH /root/.cargo/bin:$PATH
RUN rustup toolchain install stable
RUN rustup target add --toolchain stable arm-linux-androideabi
RUN rustup target add --toolchain stable armv7-linux-androideabi

# Android NDK and toolchain
RUN cd /usr/local && \
    wget -q https://dl.google.com/android/repository/android-ndk-r16b-linux-x86_64.zip && \
    unzip -q android-ndk-r16b-linux-x86_64.zip && \
    rm android-ndk-r16b-linux-x86_64.zip
ENV NDK_HOME /usr/local/android-ndk-r16b
RUN /usr/local/android-ndk-r16b/build/tools/make-standalone-toolchain.sh \
    --arch=arm --install-dir=/opt/ndk-standalone --stl=libc++ --platform=android-26
ENV PATH $PATH:/opt/ndk-standalone/bin

# Compiling libudev for Android
# This is the most hacky part of the process, as we need to apply a patch and pass specific
# options that the compiler environment doesn't define.
RUN cd /root && \
    git clone https://github.com/gentoo/eudev.git
ADD libudev.patch /root
RUN cd /root/eudev && \
    git checkout 83d918449f22720d84a341a05e24b6d109e6d3ae && \
    ./autogen.sh && \
    ./configure --disable-introspection --disable-programs --disable-hwdb \
                --host=arm-linux-androideabi --prefix=/opt/ndk-standalone/sysroot/usr/ \
                --enable-shared=false CC=arm-linux-androideabi-clang \
                CFLAGS="-D LINE_MAX=2048 -D RLIMIT_NLIMITS=15 -D IPTOS_LOWCOST=2 -std=gnu99" \
                CXX=arm-linux-androideabi-clang++ && \
    git apply - < /root/libudev.patch && \
    make && \
    make install
RUN rm -rf /root/eudev
RUN rm /root/libudev.patch

# Rust-related configuration
ADD cargo-config.toml /root/.cargo/config
ENV ARM_LINUX_ANDROIDEABI_OPENSSL_DIR /opt/ndk-standalone/sysroot/usr
ENV ARMV7_LINUX_ANDROIDEABI_OPENSSL_DIR /opt/ndk-standalone/sysroot/usr
ENV CC_arm_linux_androideabi arm-linux-androideabi-clang
ENV CC_armv7_linux_androideabi arm-linux-androideabi-clang
ENV CXX_arm_linux_androideabi arm-linux-androideabi-clang++
ENV CXX_armv7_linux_androideabi arm-linux-androideabi-clang++
ENV AR_arm_linux_androideabi arm-linux-androideabi-ar
ENV AR_armv7_linux_androideabi arm-linux-androideabi-ar
ENV CFLAGS_arm_linux_androideabi -std=gnu11 -fPIC -D OS_ANDROID
ENV CFLAGS_armv7_linux_androideabi -std=gnu11 -fPIC -D OS_ANDROID
ENV CXXFLAGS_arm_linux_androideabi -std=gnu++11 -fPIC -fexceptions -frtti -static-libstdc++ -D OS_ANDROID
ENV CXXFLAGS_armv7_linux_androideabi -std=gnu++11 -fPIC -fexceptions -frtti -static-libstdc++ -D OS_ANDROID
ENV CXXSTDLIB_arm_linux_androideabi ""
ENV CXXSTDLIB_armv7_linux_androideabi ""
