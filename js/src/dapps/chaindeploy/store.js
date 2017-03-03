// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

import { action, computed, observable } from 'mobx';

import { contracts, registry } from './contracts';
import { apps } from './dapps';
import { api } from './parity';
import { isValidNumber, trackRequest, validateCode } from './utils';

export default class ContractsStore {
  @observable apps = null;
  @observable contracts = null;
  @observable error = null;
  @observable registry = null;

  constructor () {
    this.apps = apps;
    this.contracts = contracts;
    this.registry = registry;

    api.subscribe('eth_blockNumber', this.onNewBlockNumber);
  }

  @computed get contractBadgereg () {
    return this.contracts.find((contract) => contract.id === 'badgereg');
  }

  @computed get contractDappreg () {
    return this.contracts.find((contract) => contract.id === 'dappreg');
  }

  @computed get contractGithubhint () {
    return this.contracts.find((contract) => contract.id === 'githubhint');
  }

  @computed get contractTokenreg () {
    return this.contracts.find((contract) => contract.id === 'tokenreg');
  }

  @computed get isContractDeploying () {
    return this.contracts
      .filter((contract) => contract.isDeploying)
      .length !== 0;
  }

  @computed get isDappDeploying () {
    return this.apps
      .filter((app) => app.isDeploying)
      .length !== 0;
  }

  @computed get haveAllContracts () {
    return this.contracts
      .filter((contract) => !contract.instance || !contract.hasLatestCode)
      .length === 0;
  }

  @computed get haveAllDapps () {
    return this.apps
      .filter((app) => !app.imageHash || !app.imageMatch)
      .length === 0;
  }

  @action refreshApps = () => {
    this.apps = [].concat(this.apps.peek());
  }

  @action refreshContracts = () => {
    this.contracts = [].concat(this.contracts.peek());
  }

  @action setError = (error) => {
    console.error(error);

    this.error = error.message
      ? error.message
      : error;
  }

  @action setRegistryAddress = (address, isOnChain = false) => {
    if (this.registry.address !== address || !this.registry.instance) {
      console.log(`registry found at ${address}`);

      this.registry = Object.assign({}, this.registry, {
        address,
        instance: api.newContract(registry.abi, address).instance,
        isOnChain
      });
    }
  }

  @action setRegistryCode (byteCode) {
    this.registry.hasLatestCode = validateCode(this.registry.byteCode, byteCode);
  }

  @action setRegistryDeploying = (isDeploying = false) => {
    this.registry = Object.assign({}, this.registry, {
      isDeploying,
      status: isDeploying
        ? 'Deploying contract'
        : null
    });
  }

  @action setContractAddress = (contract, address, isOnChain = false) => {
    if (contract.address !== address || !contract.instance || contract.isOnChain !== isOnChain) {
      console.log(`${contract.id} found at ${address}`);

      contract.address = address;
      contract.instance = api.newContract(contract.abi, address).instance;
      contract.isOnChain = isOnChain;

      this.refreshContracts();
    }
  }

  @action setContractCode (contract, byteCode) {
    contract.hasLatestCode = validateCode(contract.byteCode, byteCode);

    this.refreshContracts();
  }

  @action setContractDeploying = (contract, isDeploying = false) => {
    contract.isDeploying = isDeploying;
    contract.status = isDeploying
      ? 'Deploying contract'
      : null;

    this.refreshContracts();
  }

  @action setContractStatus = (contract, status) => {
    contract.status = status;

    this.refreshContracts();
  }

  @action setAppDeploying = (app, isDeploying = false) => {
    app.isDeploying = isDeploying;
    app.status = isDeploying
      ? 'Registering app'
      : null;

    this.refreshApps();
  }

  @action setAppFound = (app, isOnChain = false) => {
    if (app.isOnChain !== isOnChain) {
      console.log(`${app.name} found on dappreg`);

      app.isOnChain = isOnChain;

      this.refreshApps();
    }
  }

  @action setAppImageHash = (app, imageHash) => {
    if (app.imageHash !== imageHash) {
      console.log(`${app.name} has imageHash ${imageHash}`);

      app.imageHash = imageHash;
      app.imageMatch = imageHash === app.source.imageHash;

      this.refreshApps();
    }
  }

  @action setAppImageUrl = (app, imageUrl) => {
    if (app.imageUrl !== imageUrl) {
      console.log(`${app.name} has imageUrl ${imageUrl}`);

      app.imageUrl = imageUrl;

      this.refreshApps();
    }
  }

  @action setAppStatus = (app, status) => {
    app.status = status;

    this.refreshApps();
  }

  deployApp = (app) => {
    console.log(`Registering application ${app.id}`);

    this.setAppDeploying(app, true);

    const options = {};
    const values = [app.hashId];

    return api.parity
      .defaultAccount()
      .then((defaultAccount) => {
        options.from = defaultAccount;

        if (app.isOnChain) {
          return true;
        }

        return this.contractDappreg.instance
          .fee.call({}, [])
          .then((fee) => {
            options.value = fee;

            return this.contractDappreg.instance
              .register.estimateGas(options, values)
              .then((gasEst) => {
                options.gas = gasEst.mul(1.2);

                return trackRequest(
                  this.contractDappreg.instance.register.postTransaction(options, values),
                  (error, data) => {
                    if (error) {
                      console.error(app.id, error);
                    } else {
                      console.log(app.id, data);
                    }
                  }
                );
              });
          });
      })
      .then(() => {
        this.setAppStatus(app, 'Registering image url');

        return this.registerHash(app.source.imageHash, app.source.imageUrl, options.from);
      })
      .then(() => {
        values.push('IMG');
        values.push(app.source.imageHash);

        delete options.gas;
        delete options.value;

        this.setAppStatus(app, 'Setting Image meta');

        return this.contractDappreg.instance
          .setMeta.estimateGas(options, values)
          .then((gasEst) => {
            options.gas = gasEst.mul(1.2);

            return trackRequest(
              this.contractDappreg.instance.setMeta.postTransaction(options, values),
              (error, data) => {
                if (error) {
                  console.error(app.id, error);
                } else {
                  console.log(app.id, data);
                }
              }
            );
          });
      })
      .catch(() => {
        return null;
      })
      .then(() => {
        this.setAppDeploying(app, false);
      });
  }

  deployApps = () => {
    this.apps
      .filter((app) => !app.isDeploying && (!app.imageHash || !app.imageMatch))
      .forEach(this.deployApp);
  }

  _deployContract = (contract) => {
    console.log(`Deploying contract ${contract.id}`);

    const options = {
      data: contract.byteCode
    };

    return api.parity
      .defaultAccount()
      .then((defaultAccount) => {
        options.from = defaultAccount;

        return api
          .newContract(contract.abi)
          .deploy(options, contract.deployParams, (error, data) => {
            if (error) {
              console.error(contract.id, error);
            } else {
              console.log(contract.id, data);
            }
          })
          .then((contractAddress) => {
            return [contractAddress, defaultAccount];
          });
      });
  }

  deployContract = (contract) => {
    let defaultAccount = '0x0';

    this.setContractDeploying(contract, true);

    return this
      ._deployContract(contract)
      .then(([address, _defaultAccount]) => {
        const isOnChain = contract.isOnChain;

        defaultAccount = _defaultAccount;

        this.setContractAddress(contract, address);

        return isOnChain
          ? true
          : this.reserveAddress(contract, defaultAccount);
      })
      .then(() => {
        return this.registerAddress(contract, defaultAccount);
      })
      .catch(() => {
        return null;
      })
      .then(() => {
        this.setContractDeploying(contract, false);
      });
  }

  deployContracts = () => {
    this.contracts
      .filter((contract) => !contract.isDeploying && (!contract.instance || !contract.hasLatestCode))
      .forEach(this.deployContract);
  }

  deployRegistry = () => {
    this.setRegistryDeploying(true);

    return this
      ._deployContract(this.registry)
      .then(([address]) => {
        this.setRegistryDeploying(false);
        this.setRegistryAddress(address);
      });
  }

  reserveAddress = (contract, fromAddress) => {
    const options = { from: fromAddress };
    const values = [api.util.sha3(contract.id)];

    this.setContractStatus(contract, 'Reserving name');

    return this.registry.instance
      .fee.call({}, [])
      .then((value) => {
        options.value = value;

        return this.registry.instance
          .reserve.estimateGas(options, values)
          .then((gasEst) => {
            options.gas = gasEst.mul(1.2);

            return trackRequest(
              this.registry.instance.reserve.postTransaction(options, values).catch(() => true),
              (error, data) => {
                if (error) {
                  console.error(contract.id, error);
                } else {
                  console.log(contract.id, data);
                }
              }
            );
          });
      });
  }

  registerAddress = (contract, fromAddress) => {
    const options = { from: fromAddress };
    const values = [api.util.sha3(contract.id), 'A', contract.address];

    this.setContractStatus(contract, 'Setting lookup address');

    return this.registry.instance
      .setAddress.estimateGas(options, values)
      .then((gasEst) => {
        options.gas = gasEst.mul(1.2);

        return trackRequest(
          this.registry.instance.setAddress.postTransaction(options, values),
          (error, data) => {
            if (error) {
              console.error(contract.id, error);
            } else {
              console.log(contract.id, data);
            }
          }
        );
      });
  }

  registerHash = (hash, url, fromAddress) => {
    const options = {
      from: fromAddress
    };
    const values = [hash, url];

    return this.contractGithubhint.instance
      .entries.call({}, [hash])
      .then(([imageUrl, commit, owner]) => {
        if (isValidNumber(owner)) {
          return true;
        }

        return this.contractGithubhint.instance
          .hintURL.estimateGas(options, values)
          .then((gasEst) => {
            options.gas = gasEst.mul(1.2);

            return trackRequest(
              this.contractGithubhint.instance.hintURL.postTransaction(options, values),
              (error, data) => {
                if (error) {
                  console.error(hash, error);
                } else {
                  console.log(hash, data);
                }
              }
            );
          });
      });
  }

  findRegistry = () => {
    if (this.registry.address && this.registry.hasLatestCode) {
      return Promise.resolve(this.registry);
    }

    return api.parity
      .registryAddress()
      .then((address) => {
        if (isValidNumber(address)) {
          this.setRegistryAddress(address, true);
        }

        return api.eth.getCode(address);
      })
      .then((byteCode) => {
        this.setRegistryCode(byteCode);
      });
  }

  findApps = () => {
    if (!this.contractDappreg.instance) {
      return Promise.resolve(false);
    }

    return Promise
      .all(
        this.apps.map((app) => {
          return app.isOnChain
            ? Promise.resolve([[0]])
            : this.contractDappreg.instance.get.call({}, [app.hashId]);
        })
      )
      .then((apps) => {
        console.log(apps);

        apps.forEach(([_id], index) => {
          const id = api.util.bytesToHex(_id);

          if (isValidNumber(id)) {
            this.setAppFound(this.apps[index], true);
          }
        });

        return Promise.all(
          this.apps.map((app) => {
            return app.imageHash && app.imageMatch
              ? Promise.resolve([0])
              : this.contractDappreg.instance.meta.call({}, [app.hashId, 'IMG']);
          })
        );
      })
      .then((imageHashes) => {
        imageHashes.forEach((image, index) => {
          const imageHash = api.util.bytesToHex(image);

          if (isValidNumber(imageHash)) {
            this.setAppImageHash(this.apps[index], imageHash);
          }
        });

        return Promise.all(
          this.apps.map((app) => {
            return app.imageUrl || !app.imageHash
              ? Promise.resolve([null, null, null])
              : this.contractGithubhint.instance.entries.call({}, [app.imageHash]);
          })
        );
      })
      .then((imageUrls) => {
        imageUrls.forEach(([imageUrl, commit, owner], index) => {
          if (isValidNumber(owner)) {
            this.setAppImageUrl(this.apps[index], imageUrl);
          }
        });
      });
  }

  findContracts = () => {
    if (!this.registry.instance) {
      return Promise.resolve(false);
    }

    return Promise
      .all(
        this.contracts.map((contract) => {
          return contract.isOnChain
            ? Promise.resolve(0)
            : this.registry.instance.getAddress.call({}, [api.util.sha3(contract.id), 'A']);
        })
      )
      .then((addresses) => {
        addresses.forEach((address, index) => {
          if (isValidNumber(address)) {
            this.setContractAddress(this.contracts[index], address, true);
          }
        });

        return Promise.all(
          this.contracts.map((contract) => {
            return !contract.address || contract.hasLatestCode
              ? Promise.resolve(null)
              : api.eth.getCode(contract.address);
          })
        );
      })
      .then((codes) => {
        codes.forEach((byteCode, index) => {
          if (byteCode) {
            this.setContractCode(this.contracts[index], byteCode);
          }
        });
      });
  }

  onNewBlockNumber = (error, blockNumber) => {
    if (error) {
      return;
    }

    return this
      .findRegistry()
      .then(this.findContracts)
      .then(this.findApps)
      .catch(this.setError);
  }
}
