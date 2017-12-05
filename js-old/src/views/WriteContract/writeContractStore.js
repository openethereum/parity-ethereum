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

import { debounce } from 'lodash';
import { action, computed, observable, transaction } from 'mobx';
import React from 'react';
import { FormattedMessage } from 'react-intl';
import store from 'store';

import { sha3 } from '@parity/api/lib/util/sha3';
import SolidityUtils from '~/util/solidity';

const SOLIDITY_LIST_URL = 'https://rawgit.com/ethereum/solc-bin/gh-pages/bin/list.json';
const WRITE_CONTRACT_STORE_KEY = '_parity::writeContractStore';

const SNIPPETS = {
  snippet0: {
    name: 'Token.sol',
    description: (
      <FormattedMessage
        id='writeContract.type.standardErc20'
        defaultMessage='Standard ERC20 Token Contract'
      />
    ),
    id: 'snippet0',
    sourcecode: require('raw-loader!../../contracts/snippets/token.sol')
  },
  snippet1: {
    name: 'StandardToken.sol',
    description: (
      <FormattedMessage
        id='writeContract.type.implementErc20'
        defaultMessage='Implementation of ERC20 Token Contract'
      />
    ),
    id: 'snippet1',
    sourcecode: require('raw-loader!../../contracts/snippets/standard-token.sol')
  },
  snippet2: {
    name: 'HumanStandardToken.sol',
    description: (
      <FormattedMessage
        id='writeContract.type.humanErc20'
        defaultMessage='Implementation of the Human Token Contract'
      />
    ),
    id: 'snippet2',
    sourcecode: require('raw-loader!../../contracts/snippets/human-standard-token.sol')
  },
  snippet3: {
    name: 'Wallet.sol',
    description: (
      <FormattedMessage
        id='writeContract.type.multisig'
        defaultMessage='Implementation of a multisig Wallet'
      />
    ),
    id: 'snippet3',
    sourcecode: require('raw-loader!../../contracts/snippets/wallet.sol')
  }
};

let instance = null;

export default class WriteContractStore {
  @observable sourcecode = '';

  @observable compiled = false;
  @observable compiling = false;
  @observable loading = true;

  @observable contractIndex = -1;
  @observable contract = null;
  @observable contracts = {};

  @observable errors = [];
  @observable annotations = [];

  @observable builds = [];
  @observable selectedBuild = -1;

  @observable autocompile = false;
  @observable optimize = false;

  @observable showDeployModal = false;
  @observable showSaveModal = false;
  @observable showLoadModal = false;

  @observable savedContracts = {};
  @observable selectedContract = {};

  @observable workerError = null;

  loadingSolidity = false;
  lastCompilation = {};
  snippets = SNIPPETS;
  worker = undefined;

  useWorker = true;
  solc = {};

  constructor () {
    this.debouncedCompile = debounce(this.handleCompile, 1000);
  }

  static get () {
    if (!instance) {
      instance = new WriteContractStore();
    }

    return instance;
  }

  @action setWorkerError (error) {
    this.workerError = error;
  }

  @action setEditor (editor) {
    this.editor = editor;
  }

  @action setWorker (worker) {
    if (this.worker !== undefined) {
      return;
    }

    this.worker = worker;

    return Promise.all([
      this.fetchSolidityVersions().then(() => this.handleCompile()),
      this.reloadContracts(undefined, undefined, false)
    ]);
  }

  fetchSolidityVersions () {
    return fetch(SOLIDITY_LIST_URL)
      .then((response) => response.json())
      .then((data) => {
        const { builds, releases, latestRelease } = data;
        let latestIndex = -1;
        let promise = Promise.resolve();

        this.builds = builds.reverse().map((build, index) => {
          if (releases[build.version] === build.path) {
            build.release = true;

            if (build.version === latestRelease) {
              build.latest = true;
              promise = promise.then(() => this.loadSolidityVersion(build));
              latestIndex = index;
            }
          }

          return build;
        });

        this.selectedBuild = latestIndex;
        return promise;
      })
      .catch((error) => {
        this.setWorkerError(error);
      });
  }

  @action handleImport = (sourcecode) => {
    this.reloadContracts(-1, sourcecode);
  }

  @action handleSelectBuild = (_, index, value) => {
    this.selectedBuild = value;
    return this
      .loadSolidityVersion(this.builds[value])
      .then(() => this.handleCompile());
  }

  getCompiler (build) {
    const { longVersion } = build;

    if (!this.solc[longVersion]) {
      this.solc[longVersion] = SolidityUtils
        .getCompiler(build)
        .then((compiler) => {
          this.solc[longVersion] = compiler;
          return compiler;
        })
        .catch((error) => {
          this.setWorkerError(error);
          throw error;
        });
    }

    return Promise.resolve(this.solc[longVersion]);
  }

  @action loadSolidityVersion = (build) => {
    if (this.worker === undefined) {
      return;
    } else if (this.worker === null) {
      this.useWorker = false;
    }

    if (this.loadingSolidity) {
      return this.loadingSolidity;
    }

    this.loading = true;

    if (this.useWorker) {
      this.loadingSolidity = this.worker
        .postMessage({
          action: 'load',
          data: build
        })
        .then((result) => {
          if (result !== 'ok') {
            throw new Error('error while loading solidity: ' + result);
          }

          this.loadingSolidity = false;
          this.loading = false;
        })
        .catch((error) => {
          console.warn('error while loading solidity', error);
          this.useWorker = false;
          this.loadingSolidity = null;

          return this.loadSolidityVersion(build);
        });
    } else {
      this.loadingSolidity = this
        .getCompiler(build)
        .then(() => {
          this.loadingSolidity = false;
          this.loading = false;

          return 'ok';
        })
        .catch((error) => {
          this.setWorkerError(error);
          this.loadingSolidity = false;
          this.loading = false;
        });
    }

    return this.loadingSolidity;
  }

  @action handleOpenDeployModal = () => {
    this.showDeployModal = true;
  }

  @action handleCloseDeployModal = () => {
    this.showDeployModal = false;
  }

  @action handleOpenLoadModal = () => {
    this.showLoadModal = true;
  }

  @action handleCloseLoadModal = () => {
    this.showLoadModal = false;
  }

  @action handleOpenSaveModal = () => {
    this.showSaveModal = true;
  }

  @action handleCloseSaveModal = () => {
    this.showSaveModal = false;
  }

  @action handleSelectContract = (_, index, value) => {
    this.contractIndex = value;
    this.contract = this.contracts[Object.keys(this.contracts)[value]];
  }

  compile = (data) => {
    const { name = '' } = this.selectedContract;

    if (this.useWorker) {
      return this.worker.postMessage({
        action: 'compile',
        data: {
          ...data,
          name
        }
      });
    }

    return new Promise((resolve, reject) => {
      window.setTimeout(() => {
        this
          .getCompiler(data.build)
          .then((compiler) => {
            return SolidityUtils.compile({
              ...data,
              name
            }, compiler);
          })
          .then(resolve)
          .catch(reject);
      }, 0);
    });
  }

  @computed get isPristine () {
    return this.getHash() === this.lastCompilation.hash;
  }

  getHash () {
    const build = this.builds[this.selectedBuild];
    const version = build.longVersion;
    const sourcecode = this.sourcecode.replace(/\s+/g, ' ');

    return sha3(JSON.stringify({ version, sourcecode, optimize: this.optimize }));
  }

  @action handleCompile = () => {
    transaction(() => {
      this.compiled = false;
      this.compiling = true;
      this.setWorkerError(null);
    });

    const build = this.builds[this.selectedBuild];
    const hash = this.getHash();

    let promise = Promise.resolve(null);

    if (hash === this.lastCompilation.hash) {
      promise = new Promise((resolve) => {
        window.setTimeout(() => {
          resolve(this.lastCompilation);
        }, 500);
      });
    } else {
      promise = this
        .compile({
          sourcecode: this.sourcecode,
          build: build,
          optimize: this.optimize,
          files: this.files
        })
        .then((data) => {
          const result = this.parseCompiled(data);

          this.lastCompilation = {
            result: result,
            date: new Date(),
            version: data.version,
            hash
          };

          return this.lastCompilation;
        })
        .catch((error) => {
          this.setWorkerError(error);
        });
    }

    return promise.then((data = null) => {
      if (data) {
        const {
          contract, contractIndex,
          annotations, contracts, errors
        } = data.result;

        if (!contract && errors && errors.length > 0) {
          this.setWorkerError(errors[0]);
        } else {
          this.contract = contract;
          this.contractIndex = contractIndex;

          this.annotations = annotations;
          this.contracts = contracts;
          this.errors = errors;
        }
      }

      this.compiled = true;
      this.compiling = false;
    });
  }

  @action handleAutocompileToggle = () => {
    this.autocompile = !this.autocompile;
  }

  @action handleOptimizeToggle = () => {
    this.optimize = !this.optimize;
  }

  @action parseCompiled = (data) => {
    const { contracts } = data;

    const { errors = [] } = data;
    const errorAnnotations = this.parseErrors(errors);
    // const formalAnnotations = this.parseErrors(data.formal && data.formal.errors, true);

    const annotations = [].concat(
      errorAnnotations
    );

    const contractKeys = Object.keys(contracts || {});

    const contract = contractKeys.length ? contracts[contractKeys[0]] : null;
    const contractIndex = contractKeys.length ? 0 : -1;

    return {
      contract, contractIndex,
      contracts, errors, annotations
    };
  }

  parseErrors = (data, formal = false) => {
    const regex = /^(.*):(\d+):(\d+):\s*([a-z]+):\s*((.|[\r\n])+)$/i;

    return (data || [])
      .filter((e) => regex.test(e))
      .map((error, index) => {
        const match = regex.exec(error);

        const contract = match[1];
        const row = parseInt(match[2]) - 1;
        const column = parseInt(match[3]);

        const type = formal ? 'warning' : match[4].toLowerCase();
        const text = match[5];

        return {
          contract,
          row, column,
          type, text,
          formal
        };
      });
  }

  @action handleEditSourcecode = (value, compile = false) => {
    this.sourcecode = value;

    const localStore = store.get(WRITE_CONTRACT_STORE_KEY) || {};

    store.set(WRITE_CONTRACT_STORE_KEY, {
      ...localStore,
      current: value
    });

    if (compile) {
      this.handleCompile();
    } else if (this.autocompile) {
      this.debouncedCompile();
    }
  }

  @action handleSaveContract = () => {
    if (this.selectedContract && this.selectedContract.id !== undefined) {
      return this.handleSaveNewContract({
        ...this.selectedContract,
        sourcecode: this.sourcecode
      });
    }

    return this.handleOpenSaveModal();
  }

  getId (contracts) {
    return Object.values(contracts)
      .map((c) => c.id)
      .reduce((max, id) => Math.max(max, id), 0) + 1;
  }

  @action handleSaveNewContract = (data) => {
    const { name, sourcecode, id } = data;

    const localStore = store.get(WRITE_CONTRACT_STORE_KEY) || {};
    const savedContracts = localStore.saved || {};
    const cId = (id !== undefined)
      ? id
      : this.getId(savedContracts);

    store.set(WRITE_CONTRACT_STORE_KEY, {
      ...localStore,
      saved: {
        ...savedContracts,
        [ cId ]: { sourcecode, id: cId, name, timestamp: Date.now() }
      }
    });

    this.reloadContracts(cId);
  }

  @action reloadContracts = (id, sourcecode, recompile = true) => {
    const localStore = store.get(WRITE_CONTRACT_STORE_KEY) || {};

    this.savedContracts = localStore.saved || {};

    const cId = id !== undefined ? id : localStore.currentId;

    this.selectedContract = this.savedContracts[cId] || {};
    this.sourcecode = sourcecode !== undefined
      ? sourcecode
      : this.selectedContract.sourcecode || localStore.current || '';

    store.set(WRITE_CONTRACT_STORE_KEY, {
      ...localStore,
      currentId: this.selectedContract ? cId : null,
      current: this.sourcecode
    });

    this.resizeEditor();

    if (recompile) {
      return this.handleCompile();
    }
  }

  @action handleLoadContract = (contract) => {
    const { sourcecode, id } = contract;

    this.reloadContracts(id, sourcecode);
  }

  @action handleDeleteContract = (id) => {
    const localStore = store.get(WRITE_CONTRACT_STORE_KEY) || {};

    const savedContracts = Object.assign({}, localStore.saved || {});

    if (savedContracts[id]) {
      delete savedContracts[id];
    }

    store.set(WRITE_CONTRACT_STORE_KEY, {
      ...localStore,
      saved: savedContracts
    });

    this.reloadContracts();
  }

  @action handleNewContract = () => {
    this.reloadContracts(-1, '');
  }

  @action resizeEditor = () => {
    try {
      this.editor.refs.brace.editor.resize();
    } catch (e) {}
  }

  get files () {
    const files = [].concat(
      Object.values(this.snippets),
      Object.values(this.savedContracts)
    );

    return files;
  }
}
