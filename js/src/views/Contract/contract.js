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
import { attachContract, detachContract } from '../../redux/providers/blockchainActions';
import { EditMeta, ExecuteContract } from '../../modals';
import { Actionbar, Button, Page } from '../../ui';

import Header from '../Account/Header';
import Delete from '../Address/Delete';

import Events from './Events';
import Queries from './Queries';

import styles from './contract.css';

class Contract extends Component {
  static contextTypes = {
    api: React.PropTypes.object.isRequired
  }

  static propTypes = {
    accounts: PropTypes.object,
    balances: PropTypes.object,
    contract: PropTypes.object,
    ready: PropTypes.bool,
    isTest: PropTypes.bool,
    params: PropTypes.object
  }

  state = {
    fromAddress: '',
    showDeleteDialog: false,
    showEditDialog: false,
    showExecuteDialog: false
  }

  attachContract (props = this.props) {
    const { attachContract, params } = props;
    attachContract(params.address);
  }

  detachContract (props = this.props) {
    const { detachContract, params } = props;
    detachContract(params.address);
  }

  componentDidMount () {
    this.setBaseAccount(this.props);

    if (this.props.ready) {
      this.attachContract();
    }
  }

  componentWillReceiveProps (newProps) {
    const { accounts, params } = newProps;

    if (!this.props.ready && newProps.ready) {
      this.attachContract();
    }

    // New contract address
    if (params.address !== this.props.params.address) {
      this.attachContract(newProps);
    }

    if (Object.keys(accounts).length !== Object.keys(this.props.accounts).length) {
      this.setBaseAccount(newProps);
    }
  }

  componentWillUnmount () {
    this.detachContract();
  }

  render () {
    const { balances, contract, params, isTest } = this.props;
    const { address } = params;
    const balance = balances[address];

    if (!contract) {
      return null;
    }

    return (
      <div className={ styles.contract }>
        { this.renderActionbar() }
        { this.renderDeleteDialog() }
        { this.renderEditDialog() }
        { this.renderExecuteDialog() }
        <Page>
          <Header
            isTest={ isTest }
            account={ contract }
            balance={ balance } />

          <Queries address={ address } />
          <Events address={ address } />
        </Page>
      </div>
    );
  }

  renderActionbar () {
    const { contract } = this.props;

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
    const { contract } = this.props;
    const { showDeleteDialog } = this.state;

    return (
      <Delete
        account={ contract }
        visible={ showDeleteDialog }
        route='/contracts'
        onClose={ this.closeDeleteDialog } />
    );
  }

  renderEditDialog () {
    const { contract } = this.props;
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
    const { fromAddress, showExecuteDialog } = this.state;
    const { accounts, contract } = this.props;

    if (!showExecuteDialog) {
      return null;
    }

    return (
      <ExecuteContract
        accounts={ accounts }
        contract={ contract.instance }
        fromAddress={ fromAddress }
        onClose={ this.closeExecuteDialog }
        onFromAddressChange={ this.onFromAddressChange } />
    );
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
    this.setState({ fromAddress });
  }

}

function mapStateToProps (_, initProps) {
  const { address } = initProps.params;

  return (state) => {
    const { accounts } = state.personal;
    const { balances } = state.balances;
    const { isTest } = state.nodeStatus;
    const { contracts } = state.blockchain;

    const contract = contracts[address];
    const ready = Object.keys(state.personal.contracts).length > 0;

    return {
      ready,
      isTest,
      accounts,
      contract,
      balances
    };
  };
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    newError,
    attachContract, detachContract
  }, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Contract);
