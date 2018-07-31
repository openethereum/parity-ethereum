## Usage

```docker build -f docker/ubuntu/Dockerfile --tag ethcore/parity:branch_or_tag_name .```

## Usage - CentOS

Builds a lightweight non-root Parity docker image:

```
git clone https://github.com/paritytech/parity-ethereum.git
cd parity-ethereum
./docker/centos/build.sh
```

Fully customised build:
```
PARITY_IMAGE_REPO=my-personal/parity \
PARITY_BUILDER_IMAGE_TAG=build-latest \
PARITY_RUNNER_IMAGE_TAG=centos-parity-experimental \
./docker/centos/build.sh
```


Default values:
```
# The image name
PARITY_IMAGE_REPO - parity/parity

# The tag to be used for builder image, git commit sha will be appended
PARITY_BUILDER_IMAGE_TAG - build

# The tag to be used for runner image
PARITY_RUNNER_IMAGE_TAG - latest
```

All default ports you might use will be exposed:
```
#           secret
#      ipfs store     ui   rpc  ws   listener  discovery
#      ↓    ↓         ↓    ↓    ↓    ↓         ↓
EXPOSE 5001 8082 8083 8180 8545 8546 30303/tcp 30303/udp
```
