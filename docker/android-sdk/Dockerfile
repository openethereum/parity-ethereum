FROM parity/parity-android 
MAINTAINER Parity Technologies <admin@parity.io>

WORKDIR /build

RUN apt-get install -y nodejs 
RUN mkdir -p /opt/android-sdk && cd /opt/android-sdk && wget -q https://dl.google.com/android/repository/sdk-tools-linux-4333796.zip && unzip -q *tools*linux*.zip && rm *tools*linux*.zip 
RUN  echo "deb http://ppa.launchpad.net/webupd8team/java/ubuntu xenial main" | tee /etc/apt/sources.list.d/webupd8team-java.list
RUN  echo "deb-src http://ppa.launchpad.net/webupd8team/java/ubuntu xenial main" | tee -a /etc/apt/sources.list.d/webupd8team-java.list
RUN  apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys EEA14886
RUN  apt-get update
RUN echo "oracle-java8-installer shared/accepted-oracle-license-v1-1 select true" | sudo debconf-set-selections
RUN apt-get install -y oracle-java8-installer
RUN yes | /opt/android-sdk/tools/bin/sdkmanager --licenses || true

# cleanup
RUN apt-get autoremove -y
RUN apt-get clean -y
RUN rm -rf /tmp/* /var/tmp/*
