## Usage

```docker build -f docker/ubuntu/Dockerfile --tag ethcore/openethereum:branch_or_tag_name .```

## Usage - CentOS

Builds a lightweight non-root OpenEthereum docker image:
```
git clone https://github.com/openethereum/openethereum.git
cd openethereum
./scripts/docker/centos/build.sh
```

Fully customised build:
```
OPENETHEREUM_IMAGE_REPO=my-personal/openethereum \
OPENETHEREUM_BUILDER_IMAGE_TAG=build-latest \
OPENETHEREUM_RUNNER_IMAGE_TAG=centos-openethereum-experimental \
./scripts/docker/centos/build.sh
```

Default values:
```
# The image name
OPENETHEREUM_IMAGE_REPO - openethereum/openethereum

# The tag to be used for builder image, git commit sha will be appended
OPENETHEREUM_BUILDER_IMAGE_TAG - build

# The tag to be used for runner image
OPENETHEREUM_RUNNER_IMAGE_TAG - latest
```

All default ports you might use will be exposed:
```
#      secret
#      store     ui   rpc  ws   listener  discovery
#      ↓         ↓    ↓    ↓    ↓    ↓         ↓
EXPOSE 8082 8083 8180 8545 8546 30303/tcp 30303/udp
```
