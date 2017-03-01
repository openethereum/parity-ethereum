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
import { builtins } from './dapps';
import { api } from './parity';
import { isValidNumber, trackRequest } from './utils';

export default class ContractsStore {
  @observable builtins = null;
  @observable contracts = null;
  @observable error = null;
  @observable registry = null;

  constructor () {
    this.builtins = builtins
      .map((app) => {
        app.deploy = () => this.deployApp(app);

        return app;
      });

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
    return this.contracts.filter((contract) => contract.isDeploying).length !== 0;
  }

  @computed get isDappDeploying () {
    return this.builtins.filter((app) => app.isDeploying).length !== 0;
  }

  @computed get haveAllContracts () {
    return this.contracts.filter((contract) => !contract.instance).length === 0;
  }

  @computed get haveAllDapps () {
    return this.builtins.filter((app) => !app.imageHash).length === 0;
  }

  @action refreshBuiltins = () => {
    this.builtins = [].concat(this.builtins.peek());
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
        isDeploying: false,
        isOnChain
      });
    }
  }

  @action setRegistryDeploying = (isDeploying = false) => {
    this.registry = Object.assign({}, this.registry, { isDeploying });
  }

  @action setContractAddress = (contract, address, isOnChain = false) => {
    if (contract.address !== address || !contract.instance || contract.isOnChain !== isOnChain) {
      console.log(`${contract.id} found at ${address}`);

      contract.address = address;
      contract.instance = api.newContract(contract.abi, address).instance;
      contract.isDeploying = false;
      contract.isOnChain = isOnChain;
    }
  }

  @action setContractDeploying = (contract, isDeploying = false) => {
    contract.isDeploying = isDeploying;

    this.refreshContracts();
  }

  @action setBuiltinFound = (builtin, isOnChain = false) => {
    if (builtin.isOnChain !== isOnChain) {
      console.log(`${builtin.name} found on dappreg`);

      builtin.isOnChain = isOnChain;
    }
  }

  @action setBuiltinImageHash = (builtin, imageHash) => {
    if (builtin.imageHash !== imageHash) {
      console.log(`${builtin.name} has imageHash ${imageHash}`);

      builtin.imageHash = imageHash;
    }
  }

  @action setBuiltinImageUrl = (builtin, imageUrl) => {
    if (builtin.imageUrl !== imageUrl) {
      console.log(`${builtin.name} has imageUrl ${imageUrl}`);

      builtin.imageUrl = imageUrl;
    }
  }

  deployApp = (app) => {

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
    this.setContractDeploying(contract, true);

    return this
      ._deployContract(contract)
      .then(([address, defaultAccount]) => {
        this.setContractAddress(contract, address);

        return this.registerAddress(contract, defaultAccount);
      })
      .then(() => {
        this.refreshContracts();
      });
  }

  deployContracts = () => {
    this.contracts
      .filter((contract) => !contract.isDeploying && !contract.instance && contract.byteCode)
      .forEach(this.deployContract);
  }

  deployRegistry = () => {
    this.setRegistryDeploying(true);

    return this
      ._deployContract(this.registry)
      .then(([address]) => {
        this.setRegistryAddress(address);
      });
  }

  registerAddress = (contract, fromAddress) => {
    const options = { from: fromAddress };
    const values = [api.util.sha3(contract.id)];

    return this.registry.instance
      .fee.call({}, [])
      .then((value) => {
        options.value = value;

        return this.registry.instance
          .reserve.estimateGas(options, values)
          .then((gasEst) => {
            options.gas = gasEst.mul(1.2);

            return trackRequest(
              this.registry.instance.reserve.postTransaction(options, values),
              (error, data) => {
                if (error) {
                  console.error(contract.id, error);
                } else {
                  console.log(contract.id, data);
                }
              }
            );
          });
      })
      .then(() => {
        values.push('A');
        values.push(contract.address);

        delete options.gas;
        delete options.value;

        return this.registry.instance.setAddress.estimateGas(options, values);
      })
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

  findRegistry = () => {
    if (this.registry.address) {
      return Promise.resolve(this.registry);
    }

    return api.parity
      .registryAddress()
      .then((address) => {
        if (isValidNumber(address)) {
          this.setRegistryAddress(address, true);
        }
      });
  }

  findBuiltins = () => {
    if (!this.contractDappreg.instance) {
      return Promise.resolve(false);
    }

    return Promise
      .all(
        this.builtins.map((builtin) => {
          return builtin.isOnChain
            ? Promise.resolve(0)
            : this.contractDappreg.instance.get.call({}, [builtin.id]);
        })
      )
      .then((apps) => {
        apps.forEach(([_id], index) => {
          const id = api.util.bytesToHex(_id);

          if (isValidNumber(id)) {
            this.setBuiltinFound(this.builtins[index], true);
          }
        });

        return Promise.all(
          this.builtins.map((builtin) => {
            return builtin.imageHash
              ? Promise.resolve([0])
              : this.contractDappreg.instance.meta.call({}, [builtin.id, 'IMG']);
          })
        );
      })
      .then((imageHashes) => {
        imageHashes.forEach((image, index) => {
          const imageHash = api.util.bytesToHex(image);

          if (isValidNumber(imageHash)) {
            this.setBuiltinImageHash(this.builtins[index], imageHash);
          }
        });

        return Promise.all(
          this.builtins.map((builtin) => {
            return builtin.imageUrl || !builtin.imageHash
              ? Promise.resolve([null, null, null])
              : this.contractGithubhint.instance.entries.call({}, [builtin.imageHash]);
          })
        );
      })
      .then((imageUrls) => {
        imageUrls.forEach(([imageUrl, commit, owner], index) => {
          if (isValidNumber(owner)) {
            this.setBuiltinImageUrl(this.builtins[index], imageUrl);
          }
        });

        this.refreshBuiltins();
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
        console.log(addresses);
        addresses.forEach((address, index) => {
          if (isValidNumber(address)) {
            this.setContractAddress(this.contracts[index], address, true);
          }
        });

        this.refreshContracts();
      });
  }

  onNewBlockNumber = (error, blockNumber) => {
    if (error) {
      return;
    }

    return this
      .findRegistry()
      .then(this.findContracts)
      .then(this.findBuiltins)
      .catch(this.setError);
  }
}
