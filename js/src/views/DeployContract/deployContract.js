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

import React, { Component } from 'react';
import ContentAdd from 'material-ui/svg-icons/content/add';
import { MenuItem } from 'material-ui';

import 'brace';
import AceEditor from 'react-ace';

import 'brace/theme/solarized_dark';
import 'brace/mode/javascript';

import { Actionbar, Button, Page, Select, Input } from '../../ui';

import styles from './deployContract.css';

import CompilerWorker from 'worker-loader!./compilerWorker.js';

export default class DeployContract extends Component {

  state = {
    sourceCode: '',
    worker: null,
    compiled: false,
    compiling: false,
    contracts: {},
    errors: [],
    selectedContract: 0
  };

  render () {
    const { sourceCode } = this.state;

    return (
      <div>
        { this.renderActionbar() }
        <Page>
          <div className={ styles.container }>
            <div className={ styles.editor }>
              <h2>Solidity Source Code</h2>
              <AceEditor
                mode='javascript'
                theme='solarized_dark'
                width='100%'
                onChange={ this.onEditSource }
                name='PARITY_EDITOR'
                editorProps={ { $blockScrolling: true } }
                setOptions={ {
                  showPrintMargin: false
                } }
                value={ sourceCode }
              />
            </div>
            <div className={ styles.parameters }>
              <h2>Parameters</h2>
              <div className={ styles.panel }>
                <Button
                  label='Compile'
                  onClick={ this.compile }
                  primary={ false }
                />
                { this.renderCompilation() }
              </div>
            </div>
          </div>
        </Page>
      </div>
    );
  }

  renderActionbar () {
    const buttons = [
      <Button
        key='deployContract'
        icon={ <ContentAdd /> }
        label='deploy'
        onClick={ this.onDeployContract }
      />
    ];

    return (
      <Actionbar
        title='Write a Contract'
        buttons={ buttons }
      />
    );
  }

  renderCompilation () {
    const { compiled, contracts, compiling, selectedContract } = this.state;

    if (compiling) {
      return (
        <p>Compiling...</p>
      );
    }

    if (!compiled) {
      return (
        <p>Please compile the source code.</p>
      );
    }

    if (!contracts) {
      return this.renderErrors();
    }

    const contractKeys = Object.keys(contracts);

    if (contractKeys.length === 0) {
      return (
        <p>No contract has been found.</p>
      );
    }

    const contractsList = contractKeys.map((name, index) => (
      <MenuItem
        key={ index }
        value={ index }
        label={ name }
      >
        { name }
      </MenuItem>
    ));

    const selected = contracts[contractKeys[selectedContract]];

    return (
      <div>
        <Select
          label='Select a contract'
          value={ selectedContract }
          onChange={ this.onSelectContract }
        >
          { contractsList }
        </Select>
        { this.renderContract(selected) }
        { this.renderErrors() }
      </div>
    );
  }

  renderContract (contract) {
    const { bytecode } = contract;
    const abi = contract.interface;

    return (
      <div>
        <Input
          readOnly
          value={ abi }
          label='ABI Interface'
        />

        <Input
          readOnly
          value={ bytecode }
          label='Bytecode'
        />
      </div>
    );
  }

  renderErrors () {
    const { errors } = this.state;

    const body = errors.map((error, index) => {
      const regex = /^:(\d+):(\d+):\s*([a-z]+):\s*((.|[\r\n])+)$/gi;
      const match = regex.exec(error);

      const line = parseInt(match[1]);
      const column = parseInt(match[2]);

      const type = match[3].toLowerCase();
      const message = match[4];

      const classes = [ styles.message, styles[type] ];

      return (
        <div key={ index } className={ styles.messageContainer }>
          <div className={ classes.join(' ') }>{ message }</div>
          <span className={ styles.errorPosition }>
            L{ line } C{ column }
          </span>
        </div>
      );
    });

    return (
      <div>
        { body }
      </div>
    );
  }

  onSelectContract = (_, index, value) => {
    this.setState({ selectedContract: value });
  }

  onEditSource = (sourceCode) => {
    this.setState({ sourceCode });
  }

  compile = () => {
    this.setState({ compiling: true });

    const { sourceCode } = this.state;
    const worker = this.getWorker();

    worker.postMessage(JSON.stringify({
      action: 'compile',
      data: sourceCode
    }));

    worker.onmessage = (event) => {
      const message = JSON.parse(event.data);

      switch (message.event) {
        case 'compiled':
          this.setCompiledCode(message.data);
          break;
      }
    };
  }

  setCompiledCode = (data) => {
    const { contracts, errors } = data;

    this.setState({
      compiled: true,
      compiling: false,
      contracts, errors
    });
  }

  getWorker = () => {
    const { worker } = this.state;

    if (worker) {
      return worker;
    }

    const compiler = new CompilerWorker();
    this.setState({ worker: compiler });

    return compiler;
  }

}
