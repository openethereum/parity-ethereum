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
import keycode, { codes } from 'keycode';

import CloseIcon from 'material-ui/svg-icons/navigation/close';
import TextFieldUnderline from 'material-ui/TextField/TextFieldUnderline';

import ParityBackground from '~/ui/ParityBackground';
import IdentityIcon from '~/ui/IdentityIcon';
import InputAddress from '~/ui/Form/InputAddress';
import Tags from '~/ui/Tags';
import { fromWei } from '~/api/util/wei';

import styles from './addressSelect.css';

class AddressSelect extends Component {
  static contextTypes = {
    muiTheme: PropTypes.object.isRequired
  };

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

  nameRefs = {};

  state = {
    copied: null,
    expanded: false,
    focused: false,
    focusedCat: null,
    focusedItem: null,
    inputFocused: false,
    inputValue: '',
    left: 0,
    top: 0,
    values: []
  };

  componentWillMount () {
    this.setValues();

    window.addEventListener('resize', this.handleTagsOpacity);
  }

  componentWillReceiveProps (nextProps) {
    if (this.values && this.values.length > 0) {
      return;
    }

    this.setValues(nextProps);
  }

  componentWillUnmount () {
    window.removeEventListener('resize', this.handleTagsOpacity);
  }

  setValues (props = this.props) {
    const { accounts = {}, contracts = {}, contacts = {}, wallets = {} } = props;

    const accountsN = Object.keys(accounts).length;
    const contractsN = Object.keys(contracts).length;
    const contactsN = Object.keys(contacts).length;
    const walletsN = Object.keys(wallets).length;

    if (accountsN + contractsN + contactsN + walletsN === 0) {
      return;
    }

    this.values = [
      {
        label: 'accounts',
        values: [].concat(
          Object.values(wallets),
          Object.values(accounts)
        )
      },
      {
        label: 'contacts',
        values: Object.values(contacts)
      },
      {
        label: 'contracts',
        values: Object.values(contracts)
      }
    ];

    this.handleChange();
  }

  render () {
    const input = this.renderInput();
    const content = this.renderContent();

    const classes = [ styles.main ];

    return (
      <div
        className={ classes.join(' ') }
        onBlur={ this.handleMainBlur }
        onClick={ this.handleFocus }
        onFocus={ this.handleMainFocus }
        onKeyDown={ this.handleInputAddresKeydown }
        ref='inputAddress'
        tabIndex={ 0 }
      >
        { input }
        { content }
      </div>
    );
  }

  renderInput () {
    const { focused } = this.state;
    const { accountsInfo, disabled, error, hint, label, value } = this.props;

    const input = (
      <InputAddress
        accountsInfo={ accountsInfo }
        error={ error }
        focused={ focused }
        label={ label }
        hint={ hint }
        tabIndex={ -1 }
        value={ value }

        allowCopy={ false }
        disabled
        text
      />
    );

    if (disabled) {
      return input;
    }

    return (
      <div
        className={ styles.inputAddress }
      >
        { input }
      </div>
    );
  }

  renderContent () {
    const { muiTheme } = this.context;
    const { hint, disabled, label } = this.props;
    const { expanded, top, left, inputFocused } = this.state;

    if (disabled) {
      return null;
    }

    const classes = [ styles.overlay ];

    if (expanded) {
      classes.push(styles.expanded);
    }

    const id = `${label}_${hint}`;

    return (
      <Portal isOpened onClose={ this.handleClose }>
        <div
          className={ classes.join(' ') }
          style={ { top, left } }
          onKeyDown={ this.handleKeyDown }
        >
          <ParityBackground muiTheme={ muiTheme } className={ styles.parityBackground } />
          <div className={ styles.inputContainer }>
            <label className={ styles.label } htmlFor={ id }>
              { label }
            </label>
            <input
              id={ id }
              className={ styles.input }
              placeholder={ hint }

              onBlur={ this.handleInputBlur }
              onFocus={ this.handleInputFocus }
              onChange={ this.handleChange }

              ref={ this.setInputRef }
            />

            <div className={ styles.underline }>
              <TextFieldUnderline
                muiTheme={ muiTheme }
                focus={ inputFocused }
                style={ {
                  borderBottom: 'solid 3px'
                } }
                focusStyle={ {
                  borderBottom: 'solid 3px'
                } }
              />
            </div>

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
      <div

      >
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
    const { values } = this.state;

    if (values.length === 0) {
      return (
        <div className={ styles.categories }>
          <div className={ styles.empty }>
            No account matches this query...
          </div>
        </div>
      );
    }

    const categories = values.map((category) => {
      return this.renderCategory(category.label, category.values);
    });

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
    const { copied } = this.state;
    const { address, index = null } = _account;

    const account = this.props.accountsInfo[address];
    const name = (account && account.name && account.name.toUpperCase()) || address;
    const { tags = [] } = account && account.meta || {};

    const balance = this.renderBalance(address);

    const onClick = () => {
      this.handleClick(address);
    };

    const onFocus = () => {
      this.setState({ focusedItem: index });
    };

    const classes = [ styles.account ];

    if (index && copied === index) {
      classes.push(styles.copied);
    }

    const addressElements = name !== address
      ? (
        <div
          className={ styles.address }
          onClick={ this.preventEvent }
          ref={ `address_${index}` }
        >
          { address }
        </div>
      )
      : null;

    const setRef = (ref) => {
      const { tagsRef = [] } = this.nameRefs[address] || {};
      this.nameRefs[address] = { ref, tagsRef };
    };

    return (
      <div
        key={ address }
        ref={ `account_${index}` }
        tabIndex={ 0 }
        className={ classes.join(' ') }
        onClick={ onClick }
        onFocus={ onFocus }
      >
        <IdentityIcon address={ address } />
        <div className={ styles.accountInfo }>
          <div className={ styles.accountName }>
            <span ref={ setRef }>{ name }</span>
          </div>

          { this.renderTags(tags, address) }
          { addressElements }
          { balance }
        </div>
      </div>
    );
  }

  renderTags (tags = [], address) {
    if (tags.length === 0) {
      return null;
    }

    const setRefs = (tagRef) => {
      const nameRef = this.nameRefs[address];

      if (!nameRef) {
        return;
      }

      nameRef.tagsRef.push(tagRef);
    };

    return (
      <Tags tags={ tags } setRefs={ setRefs } />
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

  handleCustomInput = () => {
    const { allowInput } = this.props;
    const { inputValue, values } = this.state;

    // If input is HEX and allowInput === true, send it
    if (allowInput && inputValue && /^(0x)?([0-9a-f])+$/i.test(inputValue)) {
      return this.handleClick(inputValue);
    }

    // If only one value, select it
    if (values.length === 1 && values[0].values.length === 1) {
      const value = values[0].values[0];
      return this.handleClick(value.address);
    }
  }

  handleInputAddresKeydown = (event) => {
    const code = keycode(event);

    // Simulate click on input address if enter is pressed
    if (code === 'enter') {
      return this.handleDOMAction('inputAddress', 'click');
    }
  }

  handleKeyDown = (event) => {
    const codeName = keycode(event);

    if (event.ctrlKey) {
      const { focusedItem } = this.state;

      // Copy the selected address if nothing selected and there is
      // a focused item
      const isSelection = !window.getSelection || window.getSelection().type === 'Range';
      if (codeName === 'c' && focusedItem && focusedItem > 0 && !isSelection) {
        const element = ReactDOM.findDOMNode(this.refs[`address_${focusedItem}`]);

        if (!element) {
          return event;
        }

        // Copy the address from the right element
        // @see https://developers.google.com/web/updates/2015/04/cut-and-copy-commands
        try {
          const range = document.createRange();
          range.selectNode(element);
          window.getSelection().addRange(range);
          document.execCommand('copy');

          try {
            window.getSelection().removeRange(range);
          } catch (e) {
            window.getSelection().removeAllRanges();
          }

          this.setState({ copied: focusedItem }, () => {
            window.setTimeout(() => {
              this.setState({ copied: null });
            }, 250);
          });
        } catch (e) {
          console.warn('could not copy', focusedItem, e);
        }
      }

      return event;
    }

    switch (codeName) {
      case 'esc':
        event.preventDefault();
        return this.handleClose();

      case 'enter':
        const index = this.state.focusedItem;
        if (!index) {
          return this.handleCustomInput();
        }

        return this.handleDOMAction(`account_${index}`, 'click');

      case 'left':
      case 'right':
      case 'up':
      case 'down':
        event.preventDefault();
        return this.handleNavigation(codeName);

      default:
        const code = codes[codeName];

        // @see https://github.com/timoxley/keycode/blob/master/index.js
        // lower case chars
        if (code >= (97 - 32) && code <= (122 - 32)) {
          return this.handleDOMAction(this.inputRef, 'focus');
        }

        // numbers
        if (code >= 48 && code <= 57) {
          return this.handleDOMAction(this.inputRef, 'focus');
        }

        return event;
    }
  }

  handleDOMAction = (ref, method) => {
    const refItem = typeof ref === 'string' ? this.refs[ref] : ref;
    const element = ReactDOM.findDOMNode(refItem);

    if (!element || typeof element[method] !== 'function') {
      console.warn('could not find', ref, 'or method', method);
      return;
    }

    return element[method]();
  }

  focusItem = (index) => {
    this.setState({ focusedItem: index });
    return this.handleDOMAction(`account_${index}`, 'focus');
  }

  handleNavigation = (direction) => {
    const { focusedItem, focusedCat, values } = this.state;

    // Don't do anything if no values
    if (values.length === 0) {
      return;
    }

    // Focus on the first element if none selected yet if going down
    if (!focusedItem) {
      if (direction !== 'down') {
        return;
      }

      const nextValues = values[focusedCat || 0];
      const nextFocus = nextValues ? nextValues.values[0] : null;
      return this.focusItem(nextFocus && nextFocus.index || 1);
    }

    // Find the previous focused category
    const prevCategoryIndex = values.findIndex((category) => {
      return category.values.find((value) => value.index === focusedItem);
    });
    const prevFocusIndex = values[prevCategoryIndex].values.findIndex((a) => a.index === focusedItem);

    let nextCategory = prevCategoryIndex;
    let nextFocusIndex;

    // If down: increase index if possible
    if (direction === 'down') {
      const prevN = values[prevCategoryIndex].values.length;
      nextFocusIndex = Math.min(prevFocusIndex + 1, prevN - 1);
    }

    // If up: decrease index if possible
    if (direction === 'up') {
      // Focus on search if at the top
      if (prevFocusIndex === 0) {
        return this.handleDOMAction(this.inputRef, 'focus');
      }

      nextFocusIndex = prevFocusIndex - 1;
    }

    // If right: next category
    if (direction === 'right') {
      nextCategory = Math.min(prevCategoryIndex + 1, values.length - 1);
    }

    // If right: previous category
    if (direction === 'left') {
      nextCategory = Math.max(prevCategoryIndex - 1, 0);
    }

    // If left or right: try to keep the horizontal index
    if (direction === 'left' || direction === 'right') {
      this.setState({ focusedCat: nextCategory });
      nextFocusIndex = Math.min(prevFocusIndex, values[nextCategory].values.length - 1);
    }

    const nextFocus = values[nextCategory].values[nextFocusIndex].index;
    return this.focusItem(nextFocus);
  }

  handleClick = (address) => {
    // Don't do anything if it's only text-selection
    if (window.getSelection && window.getSelection().type === 'Range') {
      return;
    }

    this.props.onChange(null, address);
    this.handleClose();
  }

  handleMainBlur = () => {
    if (window.document.hasFocus() && !this.state.expanded) {
      this.closing = false;
      this.setState({ focused: false });
    }
  }

  handleMainFocus = () => {
    if (this.state.focused) {
      return;
    }

    this.setState({ focused: true }, () => {
      if (this.closing) {
        this.closing = false;
        return;
      }

      this.handleFocus();
    });
  }

  handleFocus = () => {
    const { top, left } = this.refs.inputAddress.getBoundingClientRect();

    this.setState({ top, left }, () => {
      this.setState({ expanded: true, focusedItem: null, focusedCat: null }, () => {
        this.setState({ top: 0, left: 0 }, () => {
          window.setTimeout(() => {
            this.handleDOMAction(this.inputRef, 'focus');
          }, 250);
        });
      });
    });
  }

  handleClose = () => {
    if (!this.refs.inputAddress) {
      return null;
    }

    const { top, left } = this.refs.inputAddress.getBoundingClientRect();
    this.setState({ top, left, expanded: false });
    this.closing = true;

    return this.handleDOMAction('inputAddress', 'focus');
  }

  /**
   * Filter the given values based on the given
   * filter
   */
  filterValues = (values = [], _filter = '') => {
    const filter = _filter.toLowerCase();

    return values
      // Remove empty accounts
      .filter((a) => a)
      .filter((account) => {
        const address = account.address.toLowerCase();
        const inAddress = address.includes(filter);

        if (!account.name || inAddress) {
          return inAddress;
        }

        const name = account.name.toLowerCase();
        const inName = name.includes(filter);
        const { meta = {} } = account;

        if (!meta.tags || inName) {
          return inName;
        }

        const tags = (meta.tags || []).join('');
        return tags.includes(filter);
      })
      .sort((accA, accB) => {
        const nameA = accA.name || accA.address;
        const nameB = accB.name || accB.address;

        return nameA.localeCompare(nameB);
      });
  }

  handleInputBlur = () => {
    this.setState({ inputFocused: false });
  }

  handleInputFocus = () => {
    this.setState({ focusedItem: null, inputFocused: true });
  }

  handleChange = (event = { target: {} }) => {
    const { value = '' } = event.target;
    let index = 0;

    window.setTimeout(() => { this.handleTagsOpacity(); });

    const values = this.values
      .map((category) => {
        const filteredValues = this
          .filterValues(category.values, value)
          .map((value) => {
            index++;
            return { ...value, index: parseInt(index) };
          });

        return {
          label: category.label,
          values: filteredValues
        };
      })
      .filter((category) => category.values.length > 0);

    this.setState({
      values,
      focusedItem: null,
      inputValue: value
    });
  }

  handleTagsOpacity = () => {
    Object.values(this.nameRefs).forEach((data) => {
      if (data.tagsRef.length === 0) {
        return;
      }

      const nameEl = ReactDOM.findDOMNode(data.ref);

      if (!nameEl) {
        return;
      }

      const nameBounds = nameEl.getBoundingClientRect();

      data.tagsRef.forEach((tagRef) => {
        const tagEl = ReactDOM.findDOMNode(tagRef);

        if (!tagEl) {
          return;
        }

        const tagBounds = tagEl.getBoundingClientRect();

        // Hide if haven't at least a 10px margin
        tagEl.style.opacity = (tagBounds.left > nameBounds.right + 10)
          ? 1
          : 0;
      });
    });
  }

  preventEvent = (e) => {
    e.preventDefault();
    e.stopPropagation();
  }
}

function mapStateToProps (state) {
  const { accountsInfo } = state.personal;
  const { balances } = state.balances;

  return {
    accountsInfo,
    balances
  };
}

export default connect(
  mapStateToProps
)(AddressSelect);

