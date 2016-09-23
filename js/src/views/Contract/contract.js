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
import AvPlayArrow from 'material-ui/svg-icons/av/play-arrow';
import ContentCreate from 'material-ui/svg-icons/content/create';

import { newError } from '../../redux/actions';
import { EditMeta, ExecuteContract } from '../../modals';
import { Actionbar, Button, Container, ContainerTitle, Page } from '../../ui';

import Header from '../Account/Header';
import Delete from '../Address/Delete';

import styles from './contract.css';

class Contract extends Component {
  static contextTypes = {
    api: React.PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.object,
    balances: PropTypes.object,
    contracts: PropTypes.object,
    isTest: PropTypes.bool,
    params: PropTypes.object
  }

  state = {
    contract: null,
    fromAddress: '',
    showDeleteDialog: false,
    showEditDialog: false,
    showExecuteDialog: false,
    subscriptionId: -1,
    allEvents: [],
    minedEvents: [],
    pendingEvents: []
  }

  componentDidMount () {
    this.attachContract(this.props);
    this.setBaseAccount(this.props);
    this.queryContract(this.props);
  }

  componentWillReceiveProps (newProps) {
    const { accounts, contracts } = newProps;

    if (Object.keys(contracts).length !== Object.keys(this.props.contracts).length) {
      this.attachContract(newProps);
    }

    if (Object.keys(accounts).length !== Object.keys(this.props.accounts).length) {
      this.setBaseAccount(newProps);
    }
  }

  render () {
    const { balances, contracts, params, isTest } = this.props;
    const contract = contracts[params.address];
    const balance = balances[params.address];

    if (!contract) {
      return null;
    }

    return (
      <div className={ styles.contract }>
        { this.renderActionbar(contract) }
        { this.renderDeleteDialog() }
        { this.renderEditDialog(contract) }
        { this.renderExecuteDialog() }
        <Page>
          <Header
            isTest={ isTest }
            account={ contract }
            balance={ balance } />
          { this.renderQueries() }
          { this.renderEvents() }
        </Page>
      </div>
    );
  }

  renderActionbar (contract) {
    const buttons = [
      <Button
        key='execute'
        icon={ <AvPlayArrow /> }
        label='execute'
        onClick={ this.showExecuteDialog } />,
      <Button
        key='editmeta'
        icon={ <ContentCreate /> }
        label='edit'
        onClick={ this.onEditClick } />,
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

  renderDeleteDialog () {
    const { contracts, params } = this.props;
    const { showDeleteDialog } = this.state;
    const contract = contracts[params.address];

    return (
      <Delete
        account={ contract }
        visible={ showDeleteDialog }
        route='/contracts'
        onClose={ this.closeDeleteDialog } />
    );
  }

  renderEditDialog (contract) {
    const { showEditDialog } = this.state;

    if (!showEditDialog) {
      return null;
    }

    return (
      <EditMeta
        account={ contract }
        keys={ ['description'] }
        onClose={ this.onEditClick } />
    );
  }

  renderExecuteDialog () {
    const { contract, fromAddress, showExecuteDialog } = this.state;
    const { accounts } = this.props;

    if (!showExecuteDialog) {
      return null;
    }

    return (
      <ExecuteContract
        accounts={ accounts }
        contract={ contract }
        fromAddress={ fromAddress }
        onClose={ this.closeExecuteDialog }
        onFromAddressChange={ this.onFromAddressChange } />
    );
  }

  renderEvents () {
    const { isTest } = this.props;
    const { allEvents, contract } = this.state;

    if (!contract || !allEvents || !allEvents.length) {
      return null;
    }

    const rows = allEvents.map((event) => {
      const classes = `${styles.event} ${styles[event.state]}`;
      const url = `https://${isTest ? 'testnet.' : ''}etherscan.io/tx/${event.transactionHash}`;
      const keys = Object.keys(event.params).map((key, index) => {
        return <div className={ styles.key } key={ `${event.key}_key_${index}` }>{ key }</div>;
      });
      const values = Object.values(event.params).map((value, index) => {
        return <div className={ styles.value } key={ `${event.key}_val_${index}` }>{ value.toString() }</div>;
      });

      return (
        <tr className={ classes } key={ event.key }>
          <td>{ event.state === 'pending' ? 'pending' : event.blockNumber.toFormat(0) }</td>
          <td className={ styles.txhash }>
            <div>{ event.address }</div>
            <a href={ url } target='_blank'>{ event.transactionHash }</a>
          </td>
          <td>
            <div>{ event.type } =></div>
            { keys }
          </td>
          <td>
            <div>&nbsp;</div>
            { values }
          </td>
        </tr>
      );
    });

    return (
      <Container>
        <ContainerTitle title='events' />
        <table className={ styles.events }>
          <tbody>{ rows }</tbody>
        </table>
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
    const nextTimeout = (delay = 5000) => setTimeout(this.queryContract, delay);

    if (!contract) {
      nextTimeout(500);
      return;
    }

    const queries = contract.functions
      .filter((fn) => fn.constant)
      .filter((fn) => !fn.inputs.length);

    Promise
      .all(queries.map((query) => query.call()))
      .then((returns) => {
        // console.log(returns.map((value, index) => {
        //   return [queries[index].name, index];
        // }));
        nextTimeout();
      })
      .catch((error) => {
        console.error('queryContract', error);
        nextTimeout();
      });
  }

  onEditClick = () => {
    this.setState({
      showEditDialog: !this.state.showEditDialog
    });
  }

  closeDeleteDialog = () => {
    this.setState({ showDeleteDialog: false });
  }

  showDeleteDialog = () => {
    this.setState({ showDeleteDialog: true });
  }

  closeExecuteDialog = () => {
    this.setState({ showExecuteDialog: false });
  }

  showExecuteDialog = () => {
    this.setState({ showExecuteDialog: true });
  }

  _sortEvents = (a, b) => {
    return b.blockNumber.cmp(a.blockNumber) || b.logIndex.cmp(a.logIndex);
  }

  _logToEvent = (log) => {
    const { api } = this.context;
    const key = api.util.sha3(JSON.stringify(log));
    const { address, blockNumber, logIndex, transactionHash, transactionIndex, params, type } = log;

    return {
      type: log.event,
      state: type,
      address,
      blockNumber,
      logIndex,
      transactionHash,
      transactionIndex,
      params,
      key
    };
  }

  _receiveEvents = (error, logs) => {
    if (error) {
      console.error('_receiveEvents', error);
      return;
    }

    const events = logs.map(this._logToEvent);
    const minedEvents = events
      .filter((event) => event.state === 'mined')
      .reverse()
      .concat(this.state.minedEvents)
      .sort(this._sortEvents);
    const pendingEvents = events
      .filter((event) => event.state === 'pending')
      .reverse()
      .concat(this.state.pendingEvents.filter((pending) => {
        return !events.find((event) => {
          const isMined = (event.state === 'mined') && (event.transactionHash === pending.transactionHash);
          const isPending = (event.state === 'pending') && (event.key === pending.key);

          return isMined || isPending;
        });
      }))
      .sort(this._sortEvents);
    const allEvents = pendingEvents.concat(minedEvents);

    this.setState({
      allEvents,
      minedEvents,
      pendingEvents
    });
  }

  attachContract (props) {
    if (!props) {
      return;
    }

    const { api } = this.context;
    const { contracts, params } = props;
    const account = contracts[params.address];

    if (!account) {
      return;
    }

    const contract = api.newContract(account.meta.abi, params.address);
    contract
      .subscribe(null, { limit: 50, fromBlock: 0, toBlock: 'pending' }, this._receiveEvents)
      .then((subscriptionId) => {
        this.setState({ subscriptionId });
      });

    this.setState({ contract });
  }

  setBaseAccount (props) {
    const { fromAccount } = this.state;

    if (!props || fromAccount) {
      return;
    }

    const { accounts } = props;

    this.setState({
      fromAddress: Object.keys(accounts)[0]
    });
  }

  onFromAddressChange = (event, fromAddress) => {
    this.setState({
      fromAddress
    });
  }
}

function mapStateToProps (state) {
  const { accounts, contracts } = state.personal;
  const { balances } = state.balances;
  const { isTest } = state.nodeStatus;

  return {
    isTest,
    accounts,
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
