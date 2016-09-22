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

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import ActionDelete from 'material-ui/svg-icons/action/delete';

import { newError } from '../../redux/actions';
import { Actionbar, Button, Container, ContainerTitle, Page } from '../../ui';

import Header from '../Account/Header';
import Delete from '../Address/Delete';

import styles from './contract.css';

export default class Contract extends Component {
  static contextTypes = {
    api: React.PropTypes.object.isRequired
  }

  static propTypes = {
    balances: PropTypes.object,
    contracts: PropTypes.object,
    isTest: PropTypes.bool,
    params: PropTypes.object
  }

  state = {
    contract: null,
    showDeleteDialog: false
  }

  componentDidMount () {
    this._attachContract();
  }

  componentDidReceiveProps (newProps) {
    const { contracts } = newProps;

    if (contracts === this.props.contracts) {
      return;
    }

    this._attachContract();
  }

  render () {
    const { balances, contracts, params, isTest } = this.props;
    const { showDeleteDialog } = this.state;
    const contract = contracts[params.address];
    const balance = balances[params.address];

    if (!contract) {
      return null;
    }

    return (
      <div className={ styles.contract }>
        { this.renderActionbar(contract) }
        <Delete
          account={ contract }
          visible={ showDeleteDialog }
          route='/contracts'
          onClose={ this.closeDeleteDialog } />
        <Page>
          <Header
            isTest={ isTest }
            account={ contract }
            balance={ balance } />
          { this.renderQueries() }
          { this.renderFunctions() }
          { this.renderEvents() }
        </Page>
      </div>
    );
  }

  renderActionbar (contract) {
    const buttons = [
      <Button
        key='delete'
        icon={ <ActionDelete /> }
        label='delete contract'
        onClick={ this.showDeleteDialog } />
    ];

    return (
      <Actionbar
        title='Contract Information'
        buttons={ !contract || contract.meta.deleted ? [] : buttons } />
    );
  }

  renderEvents () {
    const { contract } = this.state;

    if (!contract) {
      return null;
    }

    const events = contract.events
      .sort(this._sortEntries)
      .map((fn) => {
        return (
          <div key={ fn.signature } className={ styles.method }>
            { fn.name }
          </div>
        );
      });

    return (
      <Container>
        <ContainerTitle title='events' />
        <div className={ styles.methods }>
          { events }
        </div>
      </Container>
    );
  }

  renderFunctions () {
    const { contract } = this.state;

    if (!contract) {
      return null;
    }

    const functions = contract.functions
      .filter((fn) => !fn.constant)
      .sort(this._sortEntries).map((fn) => {
        return (
          <div
            key={ fn.signature }
            className={ styles.method }>
            { fn.name }
          </div>
        );
      });

    return (
      <Container>
        <ContainerTitle title='functions' />
        <div className={ styles.methods }>
          { functions }
        </div>
      </Container>
    );
  }

  renderQueries () {
    const { contract } = this.state;

    if (!contract) {
      return null;
    }

    const queries = contract.functions
      .filter((fn) => fn.constant)
      .sort(this._sortEntries)
      .map((fn) => {
        return (
          <div
            key={ fn.signature }
            className={ styles.method }>
            { fn.name }
          </div>
        );
      });

    return (
      <Container>
        <ContainerTitle title='queries' />
        <div className={ styles.methods }>
          { queries }
        </div>
      </Container>
    );
  }

  _sortEntries (a, b) {
    return a.name.localeCompare(b.name);
  }

  queryContract = () => {
    const { contract } = this.state;
    const queries = contract.functions.filter((fn) => fn.constant).sort(this._sortEntries);

    console.log(queries);

    Promise
      .all(
        queries
          .filter((query) => !query.inputs.length)
          .map((query) => query.call())
        )
      .then((returns) => {
        console.log(returns);
        setTimeout(this.queryContract, 5000);
      });
  }

  closeDeleteDialog = () => {
    this.setState({ showDeleteDialog: false });
  }

  showDeleteDialog = () => {
    this.setState({ showDeleteDialog: true });
  }

  _attachContract () {
    const { api } = this.context;
    const { contracts, params } = this.props;
    const account = contracts[params.address];

    if (!account) {
      return;
    }

    const contract = api.newContract(account.meta.abi);

    this.setState({ contract }, this.queryContract);
  }
}

function mapStateToProps (state) {
  const { contracts } = state.personal;
  const { balances } = state.balances;
  const { isTest } = state.nodeStatus;

  return {
    isTest,
    contracts,
    balances
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({ newError }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Contract);
