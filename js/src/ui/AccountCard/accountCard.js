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
import keycode, { codes } from 'keycode';

import IdentityIcon from '~/ui/IdentityIcon';
import Tags from '~/ui/Tags';

import { fromWei } from '~/api/util/wei';

import styles from './accountCard.css';

export default class AccountCard extends Component {

  static propTypes = {
    account: PropTypes.object.isRequired,
    onClick: PropTypes.func.isRequired,
    onFocus: PropTypes.func.isRequired,

    balance: PropTypes.object
  };

  state = {
    copied: false
  };

  tagRefs = [];

  componentWillMount () {
    window.addEventListener('resize', this.handleTagsOpacity);
  }

  componentWillUnmount () {
    window.removeEventListener('resize', this.handleTagsOpacity);
  }

  render () {
    const { account } = this.props;
    const { copied } = this.state;

    const { address, name, meta = {} } = account;

    const displayName = (name && name.toUpperCase()) || address;
    const { tags = [] } = meta;

    const classes = [ styles.account ];

    if (copied) {
      classes.push(styles.copied);
    }

    this.tagRefs = [];

    return (
      <div
        key={ address }
        tabIndex={ 0 }
        className={ classes.join(' ') }
        onClick={ this.onClick }
        onFocus={ this.onFocus }
        onKeyDown={ this.handleKeyDown }
      >
        <IdentityIcon address={ address } />
        <div className={ styles.accountInfo }>
          <div className={ styles.accountName }>
            <span ref='name'>{ displayName }</span>
          </div>

          { this.renderTags(tags, address) }
          { this.renderAddress(displayName, address) }
          { this.renderBalance(address) }
        </div>
      </div>
    );
  }

  renderAddress (name, address) {
    if (name === address) {
      return null;
    }

    return (
      <div className={ styles.addressContainer }>
        <span
          className={ styles.address }
          onClick={ this.preventEvent }
          ref={ `address` }
          title={ address }
        >
          { address }
        </span>
      </div>
    );
  }

  renderTags (tags = [], address) {
    if (tags.length === 0) {
      return null;
    }

    return (
      <Tags tags={ tags } setRefs={ this.setTagRef } />
    );
  }

  renderBalance (address) {
    const { balance = {} } = this.props;

    if (!balance.tokens) {
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
        <span>{ value }</span>
        <span className={ styles.tag }>ETH</span>
      </div>
    );
  }

  handleKeyDown = (event) => {
    const codeName = keycode(event);

    if (event.ctrlKey) {
      // Copy the selected address if nothing selected and there is
      // a focused item
      const isSelection = !window.getSelection || window.getSelection().type === 'Range';

      if (codeName === 'c' && !isSelection) {
        const element = ReactDOM.findDOMNode(this.refs.address);

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

          this.setState({ copied: true }, () => {
            window.setTimeout(() => {
              this.setState({ copied: false });
            }, 250);
          });
        } catch (e) {
          console.warn('could not copy');
        }
      }

      return event;
    }
  }

  handleTagsOpacity = () => {
    const nameEl = ReactDOM.findDOMNode(this.refs.name);

    if (!nameEl) {
      return;
    }

    const nameBounds = nameEl.getBoundingClientRect();

    this.tagRefs.forEach((tagRef) => {
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
  }

  onClick = () => {
    const { account, onClick } = this.props;
    onClick(account);
  }

  onFocus = () => {
    const { account, onFocus } = this.props;
    onFocus(account);
  }

  preventEvent = (e) => {
    e.preventDefault();
    e.stopPropagation();
  }

  setTagRef = (tagRef) => {
    this.tagRefs.push(tagRef);
  }
}
