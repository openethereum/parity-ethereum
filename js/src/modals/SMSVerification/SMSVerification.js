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
import ActionDoneAll from 'material-ui/svg-icons/action/done-all';
import ContentClear from 'material-ui/svg-icons/content/clear';

import { BusyStep, CompletedStep, Button, IdentityIcon, Modal } from '../../ui';
import { validateAddress, validateUint } from '../../util/validation';

import ABI from '../../contracts/abi/sms-verification.json';
const contract = '0x7B3F58965439b22ef1dA4BB78f16191d11ab80B0';

// import DetailsStep from './DetailsStep';

export default class SMSVerification extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired,
    // store: PropTypes.object.isRequired
  }

  static propTypes = {
    isTest: PropTypes.bool,
    account: PropTypes.string,
    onClose: PropTypes.func.isRequired
  }

  state = {
    contract: null,
    step: 0,
    number: null,
    numberError: null
  }

  componentDidMount () {
    const { api } = this.context;

    this.setState({
      contract: api.newContract(ABI, contract)
    });
  }

  render () {
    return (
      <Modal
        actions={ this.renderDialogActions() }
        title='verify your account via SMS'
        waiting={ [4] }
        visible scroll
      >
        <span>foo</span>
      </Modal>
    );
  }

  renderDialogActions () {
    const { onClose, account } = this.props;

    return (
      <div>
        <Button
          key='cancel'
          label='Cancel'
          icon={ <ContentClear /> }
          onClick={ onClose }
        />
        <Button
          key='close'
          label='Done'
          icon={ <ActionDoneAll /> }
          onClick={ onClose }
        />
      </div>
    );
  }
}
