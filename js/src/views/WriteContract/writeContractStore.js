// Copyright 2015, 2016 Ethcore (UK) Ltd.
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

import { action, observable } from 'mobx';
import store from 'store';
import { debounce } from 'lodash';

const WRITE_CONTRACT_STORE_KEY = '_parity::writeContractStore';

const SNIPPETS = {
  snippet0: {
    name: 'Token.sol',
    description: 'Standard ERP20 Token Contract',
    id: 'snippet0', sourcecode: require('raw!../../contracts/snippets/token.sol')
  },
  snippet1: {
    name: 'StandardToken.sol',
    description: 'Implementation of ERP20 Token Contract',
    id: 'snippet1', sourcecode: require('raw!../../contracts/snippets/standard-token.sol')
  },
  snippet2: {
    name: 'HumanStandardToken.sol',
    description: 'Implementation of the Human Token Contract',
    id: 'snippet2', sourcecode: require('raw!../../contracts/snippets/human-standard-token.sol')
  }
};

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

  @observable showDeployModal = false;
  @observable showSaveModal = false;
  @observable showLoadModal = false;

  @observable savedContracts = {};
  @observable selectedContract = {};

  snippets = SNIPPETS;

  constructor () {
    this.reloadContracts();
    this.fetchSolidityVersions();

    this.debouncedCompile = debounce(this.handleCompile, 1000);
  }

  @action setEditor (editor) {
    this.editor = editor;
  }

  @action setCompiler (compiler) {
    this.compiler = compiler;

    this.compiler.onmessage = (event) => {
      const message = JSON.parse(event.data);

      switch (message.event) {
        case 'compiled':
          this.parseCompiled(message.data);
          break;
        case 'loading':
          this.parseLoading(message.data);
          break;
        case 'try-again':
          this.handleCompile();
          break;
      }
    };
  }

  fetchSolidityVersions () {
    fetch('https://raw.githubusercontent.com/ethereum/solc-bin/gh-pages/bin/list.json')
      .then((r) => r.json())
      .then((data) => {
        const { builds, releases, latestRelease } = data;
        let latestIndex = -1;

        this.builds = builds.reverse().map((build, index) => {
          if (releases[build.version] === build.path) {
            build.release = true;

            if (build.version === latestRelease) {
              build.latest = true;
              this.loadSolidityVersion(build);
              latestIndex = index;
            }
          }

          return build;
        });

        this.selectedBuild = latestIndex;
      });
  }

  @action closeWorker = () => {
    this.compiler.postMessage(JSON.stringify({
      action: 'close'
    }));
  }

  @action handleImport = (sourcecode) => {
    this.reloadContracts(-1, sourcecode);
  }

  @action handleSelectBuild = (_, index, value) => {
    this.selectedBuild = value;
    this.loadSolidityVersion(this.builds[value]);
  }

  @action loadSolidityVersion = (build) => {
    this.compiler.postMessage(JSON.stringify({
      action: 'load',
      data: build
    }));
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

  @action handleCompile = () => {
    this.compiled = false;
    this.compiling = true;

    const build = this.builds[this.selectedBuild];

    if (this.compiler && typeof this.compiler.postMessage === 'function') {
      this.sendFilesToWorker();

      this.compiler.postMessage(JSON.stringify({
        action: 'compile',
        data: {
          sourcecode: this.sourcecode,
          build: build
        }
      }));
    }
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

  @action parseCompiled = (data) => {
    const { contracts } = data;

    const { errors = [] } = data;
    const errorAnnotations = this.parseErrors(errors);
    const formalAnnotations = this.parseErrors(data.formal && data.formal.errors, true);

    const annotations = [].concat(
      errorAnnotations,
      formalAnnotations
    );

    if (annotations.findIndex((a) => /__parity_tryAgain/.test(a.text)) > -1) {
      return;
    }

    const contractKeys = Object.keys(contracts || {});

    this.contract = contractKeys.length ? contracts[contractKeys[0]] : null;
    this.contractIndex = contractKeys.length ? 0 : -1;

    this.contracts = contracts;
    this.errors = errors;
    this.annotations = annotations;

    this.compiled = true;
    this.compiling = false;
  }

  @action parseLoading = (isLoading) => {
    this.loading = isLoading;

    if (!isLoading) {
      this.handleCompile();
    }
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
    } else {
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

  @action reloadContracts = (id, sourcecode) => {
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

    this.handleCompile();
    this.resizeEditor();
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

  sendFilesToWorker = () => {
    const files = [].concat(
      Object.values(this.snippets),
      Object.values(this.savedContracts)
    );

    this.compiler.postMessage(JSON.stringify({
      action: 'setFiles',
      data: files
    }));
  }

}
