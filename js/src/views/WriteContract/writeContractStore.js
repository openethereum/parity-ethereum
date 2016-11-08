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

import CompilerWorker from 'worker-loader!./compilerWorker.js';

const WRITE_CONTRACT_SAVED_KEY = 'WRITE_CONTRACT_SAVED';

export default class WriteContractStore {

  @observable sourcecode = '';

  @observable compiled = false;
  @observable compiling = false;
  @observable loading = false;

  @observable contractIndex = -1;
  @observable contract = null;
  @observable contracts = {};

  @observable errors = [];
  @observable annotations = [];

  @observable builds = [];
  @observable selectedBuild = -1;

  @observable showDeployModal = false;

  constructor () {
    const compiler = new CompilerWorker();
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
      }
    };

    const saveSourcecode = store.get(WRITE_CONTRACT_SAVED_KEY);
    this.sourcecode = saveSourcecode || '';

    this.fetchSolidityVersions();
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

  @action handleSelectContract = (_, index, value) => {
    this.contractIndex = value;
    this.contract = this.contracts[Object.keys(this.contracts)[value]];
  }

  @action handleCompile = () => {
    this.compiled = false;
    this.compiling = true;

    const build = this.builds[this.selectedBuild];

    this.compiler.postMessage(JSON.stringify({
      action: 'compile',
      data: {
        sourcecode: this.sourcecode,
        build: build
      }
    }));
  }

  @action parseCompiled = (data) => {
    this.compiled = true;
    this.compiling = false;

    const { contracts } = data;
    const regex = /^:(\d+):(\d+):\s*([a-z]+):\s*((.|[\r\n])+)$/i;

    const errors = data.errors || data.formal && data.formal.errors || [];

    const annotations = errors
      .map((error, index) => {
        const match = regex.exec(error);

        const row = parseInt(match[1]) - 1;
        const column = parseInt(match[2]);

        const type = match[3].toLowerCase();
        const text = match[4];

        return {
          row, column,
          type, text
        };
      });

    const contractKeys = Object.keys(contracts || {});

    this.contract = contractKeys.length ? contracts[contractKeys[0]] : null;
    this.contractIndex = contractKeys.length ? 0 : -1;

    this.contracts = contracts;
    this.errors = errors;
    this.annotations = annotations;
  }

  @action parseLoading = (isLoading) => {
    this.loading = isLoading;
  }

  @action handleEditSourcecode = (value) => {
    this.sourcecode = value;
    store.set(WRITE_CONTRACT_SAVED_KEY, value);
  }

}
