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

import React, { PropTypes, Component } from 'react';
import { observer } from 'mobx-react';
import { MenuItem } from 'material-ui';
import { connect } from 'react-redux';
import CircularProgress from 'material-ui/CircularProgress';

import { Actionbar, Button, Editor, Page, Select, Input } from '../../ui';
import { DeployContract } from '../../modals';

import WriteContractStore from './writeContractStore';
import styles from './writeContract.css';

@observer
class WriteContract extends Component {

  static propTypes = {
    accounts: PropTypes.object.isRequired
  };

  store = new WriteContractStore();

  componentWillUnmount () {
    this.store.closeWorker();
  }

  render () {
    const { sourcecode, annotations } = this.store;

    return (
      <div className={ styles.outer }>
        { this.renderDeployModal() }
        <Actionbar
          title='Write a Contract'
        />
        <Page className={ styles.page }>
          <div className={ styles.container }>
            <div className={ styles.editor }>
              <h2>Solidity Source Code</h2>
              <Editor
                onChange={ this.store.handleEditSourcecode }
                onExecute={ this.store.handleCompile }
                annotations={ annotations.slice() }
                value={ sourcecode }
              />
            </div>
            <div className={ styles.parameters }>
              <h2>Parameters</h2>
              { this.renderParameters() }
            </div>
          </div>
        </Page>
      </div>
    );
  }

  renderParameters () {
    const { compiling, contract, selectedBuild, loading } = this.store;

    if (selectedBuild < 0) {
      return (
        <div className={ `${styles.panel} ${styles.centeredMessage}` }>
          <CircularProgress size={ 80 } thickness={ 5 } />
          <p>Loading...</p>
        </div>
      );
    }

    if (loading) {
      const { longVersion } = this.store.builds[selectedBuild];

      return (
        <div className={ styles.panel }>
          <div className={ styles.centeredMessage }>
            <CircularProgress size={ 80 } thickness={ 5 } />
            <p>Loading Solidity { longVersion }</p>
          </div>
        </div>
      );
    }

    return (
      <div className={ styles.panel }>
        <Button
          label='Compile'
          onClick={ this.store.handleCompile }
          primary={ false }
          disabled={ compiling }
        />
        {
          contract
          ? <Button
            label='Deploy'
            onClick={ this.store.handleOpenDeployModal }
            primary={ false }
          />
          : null
        }
        { this.renderSolidityVersions() }
        { this.renderCompilation() }
      </div>
    );
  }

  renderSolidityVersions () {
    const { builds, selectedBuild } = this.store;

    const buildsList = builds.map((build, index) => (
      <MenuItem
        key={ index }
        value={ index }
        label={ build.release ? build.version : build.longVersion }
      >
        {
          build.release
          ? (<span className={ styles.big }>{ build.version }</span>)
          : build.longVersion
        }
      </MenuItem>
    ));

    return (
      <div>
        <Select
          label='Select a Solidity version'
          value={ selectedBuild }
          onChange={ this.store.handleSelectBuild }
        >
          { buildsList }
        </Select>
      </div>
    );
  }

  renderDeployModal () {
    const { showDeployModal, contract, sourcecode } = this.store;

    if (!showDeployModal) {
      return null;
    }

    return (
      <DeployContract
        abi={ contract.interface }
        code={ `0x${contract.bytecode}` }
        source={ sourcecode }
        accounts={ this.props.accounts }
        onClose={ this.store.handleCloseDeployModal }
        readOnly
      />
    );
  }

  renderCompilation () {
    const { compiled, contracts, compiling, contractIndex, contract } = this.store;

    if (compiling) {
      return (
        <div className={ styles.centeredMessage }>
          <CircularProgress size={ 80 } thickness={ 5 } />
          <p>Compiling...</p>
        </div>
      );
    }

    if (!compiled) {
      return (
        <div className={ styles.centeredMessage }>
          <p>Please compile the source code.</p>
        </div>
      );
    }

    if (!contracts) {
      return this.renderErrors();
    }

    const contractKeys = Object.keys(contracts);

    if (contractKeys.length === 0) {
      return (
        <div className={ styles.centeredMessage }>
          <p>No contract has been found.</p>
        </div>
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

    return (
      <div>
        <Select
          label='Select a contract'
          value={ contractIndex }
          onChange={ this.store.handleSelectContract }
        >
          { contractsList }
        </Select>
        { this.renderContract(contract) }
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
          value={ `0x${bytecode}` }
          label='Bytecode'
        />
      </div>
    );
  }

  renderErrors () {
    const { errors } = this.store;

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
        <h4 className={ styles.messagesHeader }>Compiler messages</h4>
        { body }
      </div>
    );
  }

}

function mapStateToProps (state) {
  const { accounts } = state.personal;
  return { accounts };
}

export default connect(
  mapStateToProps
)(WriteContract);
