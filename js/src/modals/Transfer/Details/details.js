import React, { Component, PropTypes } from 'react';
import { Checkbox, FloatingActionButton } from 'material-ui';

import CommunicationContacts from 'material-ui/svg-icons/communication/contacts';

import AddressSelector from '../../AddressSelector';
import Form, { Input } from '../../../ui/Form';

import styles from '../style.css';

const CHECK_STYLE = {
  position: 'absolute',
  top: '38px',
  left: '1em'
};

export default class Details extends Component {
  static contextTypes = {
    api: PropTypes.object.isRequired
  }

  static propTypes = {
    all: PropTypes.bool,
    extras: PropTypes.bool,
    recipient: PropTypes.string,
    recipientError: PropTypes.string,
    total: PropTypes.string,
    totalError: PropTypes.string,
    value: PropTypes.string,
    valueError: PropTypes.string,
    onChange: PropTypes.func.isRequired
  }

  state = {
    showAddresses: false
  }

  render () {
    return (
      <Form>
        <AddressSelector
          onSelect={ this.onSelectRecipient }
          visible={ this.state.showAddresses } />
        <div>
          <Input
            label='recipient address'
            hint='the recipient address'
            error={ this.props.recipientError }
            value={ this.props.recipient }
            onChange={ this.onEditRecipient } />
          <div className={ styles.floatbutton }>
            <FloatingActionButton
              primary mini
              onTouchTap={ this.onContacts }>
              <CommunicationContacts />
            </FloatingActionButton>
          </div>
        </div>
        <div className={ styles.columns }>
          <div>
            <Input
              disabled={ this.props.all }
              label='amount to transfer (in ΞTH)'
              hint='the amount to transfer to the recipient'
              value={ this.props.value }
              onChange={ this.onEditValue } />
          </div>
          <div>
            <Checkbox
              checked={ this.props.all }
              label='full account balance'
              onCheck={ this.onCheckAll }
              style={ CHECK_STYLE } />
          </div>
        </div>
        <div className={ styles.columns }>
          <div>
            <Input
              disabled
              label='total amount'
              hint='the total amount of the transaction'
              error={ this.props.totalError }
              value={ `${this.props.total} ΞTH` } />
          </div>
          <div>
            <Checkbox
              checked={ this.props.extras }
              label='advanced sending options'
              onCheck={ this.onCheckExtras }
              style={ CHECK_STYLE } />
          </div>
        </div>
      </Form>
    );
  }

  onSelectRecipient = (recipient) => {
    this.setState({ showAddresses: false });
    this.props.onChange('recipient', recipient);
  }

  onEditRecipient = (event) => {
    this.onSelectRecipient(event.target.value);
  }

  onEditValue = (event) => {
    this.props.onChange('value', event.target.value);
  }

  onCheckAll = () => {
    this.props.onChange('all', !this.props.all);
  }

  onCheckExtras = () => {
    this.props.onChange('extras', !this.props.extras);
  }

  onContacts = () => {
    this.setState({
      showAddresses: true
    });
  }
}
