// Copyright 2015, 2016 Parity Technologies (UK) Ltd.
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
import ReactDOM from 'react-dom';
import { connect } from 'react-redux';
import Portal from 'react-portal';
import keycode from 'keycode';

import CloseIcon from 'material-ui/svg-icons/navigation/close';

import IdentityIcon from '~/ui/IdentityIcon';
import InputAddress from '~/ui/Form/InputAddress';
import { fromWei } from '~/api/util/wei';

import styles from './addressSelector.css';

class AddressSelector extends Component {
  static propTypes = {
    // Required props
    onChange: PropTypes.func.isRequired,

    // Redux props
    accountsInfo: PropTypes.object,
    accounts: PropTypes.object,
    balances: PropTypes.object,
    contacts: PropTypes.object,
    contracts: PropTypes.object,
    tokens: PropTypes.object,
    wallets: PropTypes.object,

    // Optional props
    allowInput: PropTypes.bool,
    disabled: PropTypes.bool,
    error: PropTypes.string,
    hint: PropTypes.string,
    label: PropTypes.string,
    value: PropTypes.string
  };

  static defaultProps = {
    value: ''
  };

  state = {
    inputValue: '',
    indexes: [],
    values: [],
    expanded: false,
    top: 0,
    left: 0,
    focusedItem: null,
    focusedCat: null
  };

  componentWillMount () {
    this.setValues();
  }

  componentWillReceiveProps (nextProps) {
    if (this.values && this.values.length > 0) {
      return;
    }

    this.setValues(nextProps);
  }

  setValues (props = this.props) {
    const { accounts = {}, contracts = {}, contacts = {} } = props;

    this.values = [].concat(
      Object.values(accounts)
        .map((a, idx) => { a.type = 'account'; a.cat = 0; a.index = idx; return a; }),
      Object.values(contacts)
        .map((a, idx) => { a.type = 'contact'; a.cat = 1; a.index = idx; return a; }),
      Object.values(contracts)
        .map((a, idx) => { a.type = 'contract'; a.cat = 2; a.index = idx; return a; })
    );

    this.setState({ values: this.values }, () => {
      this.handleChange(null);
    });
  }

  render () {
    const input = this.renderInput();
    const content = this.renderContent();

    return (
      <div className={ styles.main } ref='main'>
        { input }
        { content }
      </div>
    );
  }

  renderInput () {
    const { accountsInfo, disabled, error, hint, label, value } = this.props;

    return (
      <div className={ styles.inputAddress }>
        <InputAddress
          accountsInfo={ accountsInfo }
          disabled={ disabled }
          error={ error }
          label={ label }
          hint={ hint }
          onClick={ this.handleFocus }
          value={ value }
          text
        />
      </div>
    );
  }

  renderContent () {
    const { hint } = this.props;
    const { expanded, top, left } = this.state;

    const classes = [ styles.overlay ];

    if (expanded) {
      classes.push(styles.expanded);
    }

    return (
      <Portal isOpened onClose={ this.handleClose }>
        <div
          className={ classes.join(' ') }
          style={ { top, left } }
          onKeyDown={ this.handleKeyDown }
        >
          <div className={ styles.inputContainer }>
            <input
              className={ styles.input }
              placeholder={ hint }

              onBlur={ this.handleBlur }
              onChange={ this.handleChange }

              ref={ this.setInputRef }
            />

            { this.renderCurrentInput() }
            { this.renderAccounts() }
            { this.renderCloseIcon() }
          </div>
        </div>
      </Portal>
    );
  }

  renderCurrentInput () {
    if (!this.props.allowInput) {
      return null;
    }

    const { inputValue } = this.state;

    if (inputValue.length === 0 || !/^(0x)?[a-f0-9]*$/i.test(inputValue)) {
      return null;
    }

    return (
      <div>
        { this.renderAccountCard({ address: inputValue }) }
      </div>
    );
  }

  renderCloseIcon () {
    const { expanded } = this.state;

    if (!expanded) {
      return null;
    }

    return (
      <div className={ styles.closeIcon } onClick={ this.handleClose }>
        <CloseIcon style={ { width: 48, height: 48 } } />
      </div>
    );
  }

  renderAccounts () {
    const { expanded, values } = this.state;

    if (!expanded) {
      // return null;
    }

    if (values.length === 0) {
      return (
        <div className={ styles.categories }>
          <div className={ styles.empty }>
            No account matches this query...
          </div>
        </div>
      );
    }

    const accounts = values.filter((a) => a.type === 'account');
    const contacts = values.filter((a) => a.type === 'contact');
    const contracts = values.filter((a) => a.type === 'contract');

    const categories = [
      this.renderCategory('accounts', accounts),
      this.renderCategory('contacts', contacts),
      this.renderCategory('contracts', contracts)
    ];

    return (
      <div className={ styles.categories }>
        { categories }
      </div>
    );
  }

  renderCategory (name, values = []) {
    if (values.length === 0) {
      return null;
    }

    const cards = values
      .map((account) => this.renderAccountCard(account));

    return (
      <div className={ styles.category } key={ name }>
        <div className={ styles.title }>{ name }</div>
        <div className={ styles.cards }>
          <div>{ cards }</div>
        </div>
      </div>
    );
  }

  renderAccountCard (_account) {
    const { address, index = null, cat = null } = _account;

    const account = this.props.accountsInfo[address];
    const name = (account && account.name && account.name.toUpperCase()) || address;
    const balance = this.renderBalance(address);

    const onClick = () => {
      this.handleClick(address);
    };

    const classes = [ styles.account ];

    const addressElements = name !== address
      ? (
        <div className={ styles.address }>{ address }</div>
      )
      : null;

    return (
      <div
        key={ address }
        ref={ `account_${cat}_${index}` }
        tabIndex={ index }
        className={ classes.join(' ') }
        onClick={ onClick }
      >
        <IdentityIcon address={ address } />
        <div className={ styles.accountInfo }>
          <div className={ styles.accountName }>{ name }</div>
          { addressElements }
          { balance }
        </div>
      </div>
    );
  }

  renderBalance (address) {
    const { balances = {} } = this.props;

    const balance = balances[address];

    if (!balance || !balance.tokens) {
      return null;
    }

    const ethToken = balance.tokens
      .find((tok) => tok.token && (tok.token.tag || '').toLowerCase() === 'eth');

    if (!ethToken) {
      return null;
    }

    const value = fromWei(ethToken.value).toFormat(3);

    return (
      <div className={ styles.balance }>
        <span className={ styles.value }>{ value }</span>
        <span className={ styles.tag }>ETH</span>
      </div>
    );
  }

  setInputRef = (refId) => {
    this.inputRef = refId;
  }

  handleKeyDown = (event) => {
    const code = keycode(event);
    const { focusedItem, indexes } = this.state;

    const firstCat = indexes.findIndex((a) => a && a.length > 0);
    let focusedCat = this.state.focusedCat === null ? firstCat : this.state.focusedCat;
    if (!indexes[focusedCat].find((a) => a)) {
      focusedCat = firstCat;
    }

    const catIndexes = indexes[focusedCat];

    let nextIndex;
    let nextCat;

    switch (code) {
      case 'esc':
        return this.handleClose();

      case 'right':
        if (focusedCat >= indexes.length - 1) {
          return;
        }

        nextCat = focusedCat + 1;
        nextIndex = Math.min(indexes[nextCat].length - 1, focusedItem);
        return this.setState({ focusedItem: nextIndex, focusedCat: nextCat }, () => {
          ReactDOM.findDOMNode(this.refs[`account_${nextCat}_${nextIndex}`]).focus();
        });

      case 'left':
        if (focusedCat <= 0) {
          return;
        }

        nextCat = focusedCat - 1;
        nextIndex = Math.min(indexes[nextCat].length - 1, focusedItem);
        return this.setState({ focusedItem: nextIndex, focusedCat: nextCat }, () => {
          ReactDOM.findDOMNode(this.refs[`account_${nextCat}_${nextIndex}`]).focus();
        });

      case 'down':
        if (focusedItem === null) {
          nextIndex = catIndexes[focusedCat];
        } else {
          const lastIndex = catIndexes.indexOf(focusedItem);
          nextIndex = lastIndex >= catIndexes.length - 1 ? focusedItem : catIndexes[lastIndex + 1];
        }

        return this.setState({ focusedItem: nextIndex }, () => {
          ReactDOM.findDOMNode(this.refs[`account_${focusedCat}_${nextIndex}`]).focus();
        });

      case 'up':
        if (focusedItem === null && focusedCat === null) {
          return;
        }

        const lastIndex = catIndexes.indexOf(focusedItem);

        if (lastIndex <= 0) {
          return this.setState({ focusedItem: null, focusedCat: null }, () => {
            return ReactDOM.findDOMNode(this.inputRef).focus();
          });
        }

        nextIndex = catIndexes[lastIndex - 1];

        return this.setState({ focusedItem: nextIndex }, () => {
          ReactDOM.findDOMNode(this.refs[`account_${focusedCat}_${nextIndex}`]).focus();
        });

      case 'enter':
        const account = this.values.find((a) => a.index === focusedItem && a.cat === focusedCat);
        return this.handleClick(account && account.address);
    }
  }

  handleClick = (address) => {
    this.props.onChange(null, address);
    this.handleClose();
  }

  handleFocus = () => {
    const { top, left } = this.refs.main.getBoundingClientRect();

    this.setState({ top, left, expanded: true, focusedItem: null, focusedCat: null }, () => {
      this.setState({ top: 0, left: 0 }, () => {
        window.setTimeout(() => {
          ReactDOM.findDOMNode(this.inputRef).focus();
        }, 250);
      });
    });
  }

  handleClose = () => {
    if (!this.refs.main) {
      return null;
    }

    const { top, left } = this.refs.main.getBoundingClientRect();
    this.setState({ top, left, expanded: false });
  }

  handleChange = (event) => {
    const { value = '' } = event && event.target || {};
    const indexes = [];

    const values = this.values
      .filter((account) => {
        const address = account.address.toLowerCase();
        const name = (account.name || address).toLowerCase();

        return address.includes(value) || name.includes(value);
      })
      .map((a, idx) => {
        if (!indexes[a.cat]) {
          indexes[a.cat] = [];
        }

        indexes[a.cat].push(a.index);
        return a;
      });

    this.setState({ values, inputValue: value, indexes, focusedItem: null, focusedCat: null });
  }
}

function mapStateToProps (state) {
  // const { accounts, contacts, contracts } = state.personal;
  const { accountsInfo } = state.personal;
  const { balances } = state.balances;

  return {
    // accounts,
    // contacts,
    // contracts,
    accountsInfo,
    balances
  };
}

export default connect(
  mapStateToProps
)(AddressSelector);

