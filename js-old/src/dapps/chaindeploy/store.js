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

import { contracts as contractsInfo, registry as registryInfo } from './contracts';
import { apps } from './dapps';
import { api } from './parity';
import { executeContract, isValidNumber, validateCode } from './utils';

export default class ContractsStore {
  @observable apps = null;
  @observable badges = null;
  @observable contracts = null;
  @observable error = null;
  @observable registry = null;

  constructor () {
    this.apps = apps;
    this.badges = contractsInfo.filter((contract) => contract.isBadge);
    this.contracts = contractsInfo.filter((contract) => !contract.isBadge);
    this.registry = registryInfo;

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

  @computed get isBadgeDeploying () {
    return this.badges
      .filter((contract) => contract.isDeploying)
      .length !== 0;
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

  @computed get haveAllBadges () {
    return this.badges
      .filter((contract) => !contract.instance || !contract.hasLatestCode || !contract.badgeImageHash || !contract.badgeImageMatch || !contract.isBadgeRegistered)
      .length === 0;
  }

  @computed get haveAllContracts () {
    return this.contracts
      .filter((contract) => !contract.instance || !contract.hasLatestCode)
      .length === 0;
  }

  @computed get haveAllDapps () {
    return this.apps
      .filter((app) => {
        return !app.isOnChain ||
          !app.imageHash || !app.imageMatch ||
          (app.source.contentHash && !app.contentMatch) ||
          (app.source.manifestHash && !app.manifestMatch);
      })
      .length === 0;
  }

  @action refreshApps = () => {
    this.apps = [].concat(this.apps.peek());
  }

  @action refreshContracts = () => {
    this.badges = [].concat(this.badges.peek());
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
        instance: api.newContract(this.registry.abi, address).instance,
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

  @action setBadgeId = (badge, badgeId) => {
    badge.badgeId = badgeId;
    badge.isBadgeRegistered = true;

    this.refreshContracts();
  }

  @action setBadgeImageHash = (badge, imageHash) => {
    badge.badgeImageHash = imageHash;
    badge.badgeImageMatch = badge.badgeSource.imageHash === imageHash;

    this.refreshContracts();
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

  @action setAppContentHash = (app, contentHash) => {
    if (app.contentHash !== contentHash) {
      console.log(`${app.name} has contentHash ${contentHash}`);

      app.contentHash = contentHash;
      app.contentMatch = contentHash === app.source.contentHash;

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

  @action setAppManifestHash = (app, manifestHash) => {
    if (app.manifestHash !== manifestHash) {
      console.log(`${app.name} has manifestHash ${manifestHash}`);

      app.manifestHash = manifestHash;
      app.manifestMatch = manifestHash === app.source.manifestHash;

      this.refreshApps();
    }
  }

  @action setAppStatus = (app, status) => {
    console.log(app.id, status);

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

            return executeContract(app.id, this.contractDappreg, 'register', options, values);
          });
      })
      .then(() => {
        if (app.imageHash && app.imageMatch) {
          return true;
        }

        this.setAppStatus(app, 'Registering image url');

        return this
          .registerHash(app.source.imageHash, app.source.imageUrl, options.from)
          .then(() => this.setAppMeta(app, 'IMG', app.source.imageHash, options.from));
      })
      .then(() => {
        if (!app.source.manifestHash || app.manifestMatch) {
          return true;
        }

        this.setAppStatus(app, 'Registering manifest url');

        return this
          .registerHash(app.source.manifestHash, app.source.manifestUrl, options.from)
          .then(() => this.setAppMeta(app, 'MANIFEST', app.source.manifestHash, options.from));
      })
      .then(() => {
        if (!app.source.contentHash || app.contentMatch) {
          return true;
        }

        this.setAppStatus(app, 'Registering content url');

        return this
          .registerRepo(app.source.contentHash, app.source.contentUrl, options.from)
          .then(() => this.setAppMeta(app, 'CONTENT', app.source.contentHash, options.from));
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
      .filter((app) => {
        return !app.isDeploying &&
          (
            !app.isOnChain ||
            (!app.imageHash || !app.imageMatch) ||
            (app.source.contentHash && !app.contentMatch) ||
            (app.source.manifestHash && !app.manifestMatch)
          );
      })
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
    if (contract.hasLatestCode) {
      return Promise.resolve(false);
    }

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

  deployBadge = (badge) => {
    let defaultAccount;

    return this
      .deployContract(badge)
      .then(() => {
        this.setContractDeploying(badge, true);

        return api.parity.defaultAccount();
      })
      .then((_defaultAccount) => {
        defaultAccount = _defaultAccount;

        if (badge.isBadgeRegistered) {
          return true;
        }

        this.setContractStatus(badge, 'Registering with badgereg');

        return this.registerBadge(badge, defaultAccount);
      })
      .then(() => {
        if (badge.badgeImageMatch) {
          return true;
        }

        this.setContractStatus(badge, 'Registering image url');

        return this
          .registerHash(badge.badgeSource.imageHash, badge.badgeSource.imageUrl, defaultAccount)
          .then(() => this.registerBadgeImage(badge, badge.badgeSource.imageHash, defaultAccount));
      })
      .then(() => {
        this.setContractDeploying(badge, false);
      });
  }

  deployContracts = () => {
    this.contracts
      .filter((contract) => !contract.isDeploying && (!contract.instance || !contract.hasLatestCode))
      .forEach(this.deployContract);
  }

  deployBadges = () => {
    this.badges
      .filter((contract) => !contract.isDeploying && (!contract.instance || !contract.hasLatestCode || !contract.badgeImageHash || !contract.badgeImageMatch || !contract.isBadgeRegistered))
      .forEach(this.deployBadge);
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

  registerBadge = (badge, fromAddress) => {
    const options = {
      from: fromAddress
    };
    const values = [badge.address, api.util.sha3.text(badge.id.toLowerCase())];

    return this.contractBadgereg.instance
      .fee.call({}, [])
      .then((fee) => {
        options.value = fee;

        return executeContract(badge.id, this.contractBadgereg, 'register', options, values);
      });
  }

  registerBadgeImage = (badge, hash, fromAddress) => {
    const options = {
      from: fromAddress
    };
    const values = [badge.badgeId, 'IMG', hash];

    this.setContractStatus(badge, 'Setting meta IMG');

    return executeContract(badge.id, this.contractBadgereg, 'setMeta', options, values);
  }

  setAppMeta = (app, key, meta, fromAddress) => {
    const options = {
      from: fromAddress
    };
    const values = [app.hashId, key, meta];

    this.setAppStatus(app, `Setting meta ${key}`);

    return executeContract(app.id, this.contractDappreg, 'setMeta', options, values);
  }

  reserveAddress = (contract, fromAddress) => {
    const options = { from: fromAddress };
    const values = [api.util.sha3.text(contract.id.toLowerCase())];

    this.setContractStatus(contract, 'Reserving name');

    return this.registry.instance
      .fee.call({}, [])
      .then((value) => {
        options.value = value;

        return executeContract(contract.id, this.registry, 'reserve', options, values);
      });
  }

  registerAddress = (contract, fromAddress) => {
    const options = { from: fromAddress };
    const values = [api.util.sha3.text(contract.id.toLowerCase()), 'A', contract.address];

    this.setContractStatus(contract, 'Setting lookup address');

    return executeContract(contract.id, this.registry, 'setAddress', options, values);
  }

  registerRepo = (hash, content, fromAddress) => {
    const options = {
      from: fromAddress
    };
    const values = [hash, content.repo || content, content.commit || 0];

    return this.contractGithubhint.instance
      .entries.call({}, [hash])
      .then(([imageUrl, commit, owner]) => {
        if (isValidNumber(owner)) {
          return true;
        }

        return executeContract(hash, this.contractGithubhint, 'hint', options, values);
      })
      .catch(() => false);
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

        return executeContract(hash, this.contractGithubhint, 'hintURL', options, values);
      })
      .catch(() => false);
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
        apps.forEach(([_id, owner], index) => {
          const id = api.util.bytesToHex(_id);

          if (isValidNumber(id)) {
            this.setAppFound(this.apps[index], true);
          }
        });

        return Promise.all(
          this.apps.map((app) => {
            return !app.isOnChain || (app.imageHash && app.imageMatch)
              ? Promise.resolve([[0], [0], [0]])
              : Promise.all([
                this.contractDappreg.instance.meta.call({}, [app.hashId, 'CONTENT']),
                this.contractDappreg.instance.meta.call({}, [app.hashId, 'IMG']),
                this.contractDappreg.instance.meta.call({}, [app.hashId, 'MANIFEST'])
              ]);
          })
        );
      })
      .then((hashes) => {
        hashes.forEach(([content, image, manifest], index) => {
          const contentHash = api.util.bytesToHex(content);
          const imageHash = api.util.bytesToHex(image);
          const manifestHash = api.util.bytesToHex(manifest);

          if (isValidNumber(contentHash)) {
            this.setAppContentHash(this.apps[index], contentHash);
          }

          if (isValidNumber(imageHash)) {
            this.setAppImageHash(this.apps[index], imageHash);
          }

          if (isValidNumber(manifestHash)) {
            this.setAppManifestHash(this.apps[index], manifestHash);
          }
        });
      });
  }

  findBadges = () => {
    if (!this.contractBadgereg.instance) {
      return Promise.resolve(false);
    }

    return this
      .findContracts(this.badges)
      .then(() => {
        return Promise.all(
          this.badges.map((badge) => {
            return badge.isBadgeRegistered
              ? Promise.resolve([0, 0, 0])
              : this.contractBadgereg.instance.fromAddress.call({}, [badge.address]);
          })
        );
      })
      .then((badgeInfos) => {
        badgeInfos.forEach(([id, name, owner], index) => {
          if (isValidNumber(owner)) {
            this.setBadgeId(this.badges[index], id);
          }
        });

        return Promise
          .all(
            this.badges.map((badge) => {
              return !badge.isBadgeRegistered
                ? Promise.resolve([0])
                : this.contractBadgereg.instance.meta.call({}, [badge.badgeId, 'IMG']);
            })
          );
      })
      .then((images) => {
        images.forEach((imageBytes, index) => {
          const imageHash = api.util.bytesToHex(imageBytes);

          if (isValidNumber(imageHash)) {
            this.setBadgeImageHash(this.badges[index], imageHash);
          }
        });
      });
  }

  findContracts = (contracts = this.contracts) => {
    if (!this.registry.instance) {
      return Promise.resolve(false);
    }

    return Promise
      .all(
        contracts.map((contract) => {
          const hashId = api.util.sha3.text(contract.id.toLowerCase());

          return contract.isOnChain
            ? Promise.resolve([0, 0])
            : Promise.all([
              this.registry.instance.getAddress.call({}, [hashId, 'A']),
              this.registry.instance.getOwner.call({}, [hashId])
            ]);
        })
      )
      .then((addresses) => {
        addresses.forEach(([address, owner], index) => {
          if (isValidNumber(owner) && isValidNumber(address)) {
            this.setContractAddress(contracts[index], address, true);
          }
        });

        return Promise.all(
          contracts.map((contract) => {
            return !contract.address || contract.hasLatestCode
              ? Promise.resolve(null)
              : api.eth.getCode(contract.address);
          })
        );
      })
      .then((codes) => {
        codes.forEach((byteCode, index) => {
          if (byteCode) {
            this.setContractCode(contracts[index], byteCode);
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
      .then(this.findBadges)
      .catch(this.setError);
  }
}
