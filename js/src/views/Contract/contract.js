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

import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import BigNumber from 'bignumber.js';

import { EditMeta, ExecuteContract } from '~/modals';
import { newError } from '~/redux/actions';
import { setVisibleAccounts } from '~/redux/providers/personalActions';
import { Actionbar, Button, Page, Portal } from '~/ui';
import { CancelIcon, DeleteIcon, EditIcon, PlayIcon, VisibleIcon } from '~/ui/Icons';
import Editor from '~/ui/Editor';
import { getSender, loadSender } from '~/util/tx';

import Header from '../Account/Header';
import Delete from '../Address/Delete';

import Events from './Events';
import Queries from './Queries';

import styles from './contract.css';

class Contract extends Component {
  static contextTypes = {
    api: React.PropTypes.object.isRequired
  };

  static propTypes = {
    setVisibleAccounts: PropTypes.func.isRequired,

    accounts: PropTypes.object,
    contracts: PropTypes.object,
    netVersion: PropTypes.string.isRequired,
    params: PropTypes.object
  };

  state = {
    contract: null,
    fromAddress: getSender(),
    showDeleteDialog: false,
    showEditDialog: false,
    showExecuteDialog: false,
    showDetailsDialog: false,
    subscriptionId: -1,
    blockSubscriptionId: -1,
    allEvents: [],
    minedEvents: [],
    pendingEvents: [],
    queryValues: {},
    loadingEvents: true
  };

  componentDidMount () {
    const { api } = this.context;

    this.attachContract(this.props);
    this.setBaseAccount(this.props);
    this.setVisibleAccounts();

    api
      .subscribe('eth_blockNumber', this.queryContract)
      .then(blockSubscriptionId => this.setState({ blockSubscriptionId }));

    loadSender(api)
      .then((defaultAccount) => {
        if (defaultAccount !== this.state.fromAddress) {
          this.onFromAddressChange(null, defaultAccount);
        }
      });
  }

  componentWillReceiveProps (nextProps) {
    const { accounts, contracts } = nextProps;

    if (Object.keys(contracts).length !== Object.keys(this.props.contracts).length) {
      this.attachContract(nextProps);
    }

    if (Object.keys(accounts).length !== Object.keys(this.props.accounts).length) {
      this.setBaseAccount(nextProps);
    }

    const prevAddress = this.props.params.address;
    const nextAddress = nextProps.params.address;

    if (prevAddress !== nextAddress) {
      this.setVisibleAccounts(nextProps);
    }
  }

  componentWillUnmount () {
    const { api } = this.context;
    const { subscriptionId, blockSubscriptionId, contract } = this.state;

    if (blockSubscriptionId >= 0) {
      api.unsubscribe(blockSubscriptionId);
    }

    if (subscriptionId >= 0) {
      contract.unsubscribe(subscriptionId);
    }

    this.props.setVisibleAccounts([]);
  }

  setVisibleAccounts (props = this.props) {
    const { params, setVisibleAccounts } = props;
    const addresses = [ params.address ];

    setVisibleAccounts(addresses);
  }

  render () {
    const { contracts, netVersion, params } = this.props;
    const { allEvents, contract, queryValues, loadingEvents } = this.state;
    const account = contracts[params.address];

    if (!account) {
      return null;
    }

    return (
      <div>
        { this.renderActionbar(account) }
        { this.renderDeleteDialog(account) }
        { this.renderEditDialog(account) }
        { this.renderExecuteDialog() }
        <Page padded>
          <Header
            account={ account }
            isContract
          >
            { this.renderBlockNumber(account.meta) }
          </Header>
          <Queries
            contract={ contract }
            values={ queryValues }
          />
          <Events
            isLoading={ loadingEvents }
            events={ allEvents }
            netVersion={ netVersion }
          />
          { this.renderDetails(account) }
        </Page>
      </div>
    );
  }

  renderBlockNumber (meta = {}) {
    const { blockNumber } = meta;

    if (!blockNumber) {
      return null;
    }

    const formattedBlockNumber = (new BigNumber(blockNumber)).toFormat();

    return (
      <div className={ styles.blockNumber }>
        <FormattedMessage
          id='contract.minedBlock'
          defaultMessage='Mined at block #{blockNumber}'
          values={ {
            blockNumber: formattedBlockNumber
          } }
        />
      </div>
    );
  }

  renderDetails (contract) {
    const { showDetailsDialog } = this.state;

    if (!showDetailsDialog) {
      return null;
    }

    const cancelBtn = (
      <Button
        icon={ <CancelIcon /> }
        label={
          <FormattedMessage
            id='contract.buttons.close'
            defaultMessage='Close'
          />
        }
        onClick={ this.closeDetailsDialog }
      />
    );

    return (
      <Portal
        buttons={ [ cancelBtn ] }
        onClose={ this.closeDetailsDialog }
        open
        title={
          <FormattedMessage
            id='contract.details.title'
            defaultMessage='contract details'
          />
        }
      >
        <div className={ styles.details }>
          { this.renderSource(contract) }

          <div>
            <h4>Contract ABI</h4>
            <Editor
              value={ JSON.stringify(contract.meta.abi, null, 2) }
              mode='json'
              maxLines={ 20 }
              readOnly
            />
          </div>
        </div>
      </Portal>
    );
  }

  renderSource (contract) {
    const { source } = contract.meta;

    if (!source) {
      return null;
    }

    return (
      <div>
        <h4>Contract source code</h4>
        <Editor
          value={ source }
          readOnly
        />
      </div>
    );
  }

  renderActionbar (account) {
    const buttons = [
      <Button
        key='execute'
        icon={ <PlayIcon /> }
        label={
          <FormattedMessage
            id='contract.buttons.execute'
            defaultMessage='execute'
          />
        }
        onClick={ this.showExecuteDialog }
      />,
      <Button
        key='editmeta'
        icon={ <EditIcon /> }
        label={
          <FormattedMessage
            id='contract.buttons.edit'
            defaultMessage='edit'
          />
        }
        onClick={ this.showEditDialog }
      />,
      <Button
        key='delete'
        icon={ <DeleteIcon /> }
        label={
          <FormattedMessage
            id='contract.buttons.forget'
            defaultMessage='forget'
          />
        }
        onClick={ this.showDeleteDialog }
      />,
      <Button
        key='viewDetails'
        icon={ <VisibleIcon /> }
        label={
          <FormattedMessage
            id='contract.buttons.details'
            defaultMessage='details'
          />
        }
        onClick={ this.showDetailsDialog }
      />
    ];

    return (
      <Actionbar
        title={
          <FormattedMessage
            id='contract.title'
            defaultMessage='Contract Information'
          />
        }
        buttons={ !account ? [] : buttons }
      />
    );
  }

  renderDeleteDialog (account) {
    const { showDeleteDialog } = this.state;

    return (
      <Delete
        account={ account }
        visible={ showDeleteDialog }
        route='/contracts'
        onClose={ this.closeDeleteDialog }
      />
    );
  }

  renderEditDialog (account) {
    const { showEditDialog } = this.state;

    if (!showEditDialog) {
      return null;
    }

    return (
      <EditMeta
        account={ account }
        onClose={ this.closeEditDialog }
      />
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
        onFromAddressChange={ this.onFromAddressChange }
      />
    );
  }

  queryContract = () => {
    const { contract } = this.state;

    if (!contract) {
      return;
    }

    const queries = contract.functions
      .filter((fn) => fn.constant)
      .filter((fn) => !fn.inputs.length);

    Promise
      .all(queries.map((query) => query.call({ rawTokens: true })))
      .then(results => {
        const values = queries.reduce((object, fn, idx) => {
          const key = fn.name;

          object[key] = fn.outputs.length === 1
            ? [ results[idx] ]
            : results[idx];

          return object;
        }, {});

        this.setState({ queryValues: values });
      })
      .catch((error) => {
        console.error('queryContract', error);
      });
  }

  closeEditDialog = () => {
    this.setState({ showEditDialog: false });
  }

  showEditDialog = () => {
    this.setState({ showEditDialog: true });
  }

  closeDeleteDialog = () => {
    this.setState({ showDeleteDialog: false });
  }

  showDeleteDialog = () => {
    this.setState({ showDeleteDialog: true });
  }

  showDetailsDialog = () => {
    this.setState({ showDetailsDialog: true });
  }

  closeDetailsDialog = () => {
    this.setState({ showDetailsDialog: false });
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
    if (this.state.loadingEvents) {
      this.setState({ loadingEvents: false });
    }

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
      .subscribe(null, { limit: 25, fromBlock: 0, toBlock: 'pending' }, this._receiveEvents)
      .then((subscriptionId) => {
        this.setState({ subscriptionId });
      });

    this.setState({ contract }, this.queryContract);
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
  const { netVersion } = state.nodeStatus;

  return {
    accounts,
    contracts,
    netVersion
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    newError,
    setVisibleAccounts
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Contract);
