import React, { Component, PropTypes } from 'react';
import { FlatButton } from 'material-ui';
import ContentClear from 'material-ui/svg-icons/content/clear';

import IdentityIcon from '../../ui/IdentityIcon';
import Modal from '../../ui/Modal';

import styles from './style.css';

export default class AddressSelector extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  static propTypes = {
    onSelect: PropTypes.func.isRequired
  }

  state = {
    accounts: [],
    contacts: []
  }

  componentDidMount () {
    this.retrieveAccounts();
  }

  render () {
    return (
      <Modal
        scroll
        visible
        actions={ this.renderDialogActions() }>
        { this.renderAccounts('accounts') }
        { this.renderAccounts('contacts') }
      </Modal>
    );
  }

  renderAccounts (type) {
    const nothing = (<div className={ styles.nothing }>There are no addresses available</div>);
    const list = this.state[type].map((acc) => {
      return (
        <div
          key={ acc.address }
          className={ styles.account }>
          <IdentityIcon
            center inline
            address={ acc.address } />
          <div>
            <div
              className={ styles.name }
              data-address={ acc.address }
              onTouchTap={ this.onSelect }>
              { acc.name }
            </div>
            <div className={ styles.address }>
              { acc.address }
            </div>
          </div>
        </div>
      );
    });

    return (
      <div>
        <div className={ styles.header }>
          <h3>{ type }</h3>
        </div>
        <div className={ styles.body }>
          { list.length ? list : nothing }
        </div>
      </div>
    );
  }

  renderDialogActions () {
    return (
      <FlatButton
        icon={ <ContentClear /> }
        label='Cancel'
        primary
        onTouchTap={ this.onClose } />
    );
  }

  onSelect = (event) => {
    const address = event.target.getAttribute('data-address');

    if (address) {
      this.props.onSelect(address);
    }
  }

  onClose = () => {
    this.props.onSelect(null);
  }

  retrieveAccounts () {
    const api = this.context.api;

    api.personal
      .accountsInfo()
      .then((infos) => {
        const all = Object.keys(infos).map((address) => {
          const info = infos[address];

          return {
            address: address,
            name: info.name,
            uuid: info.uuid,
            meta: info.meta
          };
        });

        this.setState({
          accounts: all.filter((acc) => acc.uuid),
          contacts: all.filter((acc) => !acc.uuid)
        });
      });
  }
}
