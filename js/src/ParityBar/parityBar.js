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

import throttle from 'lodash.throttle';
import { observer } from 'mobx-react';
import React, { Component } from 'react';
import PropTypes from 'prop-types';
import ReactDOM from 'react-dom';
import { FormattedMessage } from 'react-intl';
import { Link } from 'react-router';
import { connect } from 'react-redux';
import store from 'store';

import AccountCard from '@parity/ui/AccountCard';
import Badge from '@parity/ui/Badge';
import Button from '@parity/ui/Button';
import ContainerTitle from '@parity/ui/Container/Title';
import IdentityIcon from '@parity/ui/IdentityIcon';
import GradientBg from '@parity/ui/GradientBg';
import SelectionList from '@parity/ui/SectionList';
import { CancelIcon, FingerprintIcon } from '@parity/ui/Icons';

import imagesEthcoreBlock from '@parity/shared/assets/images/parity-logo-white-no-text.svg';

import DappsStore from '@parity/shared/mobx/dappsStore';
import Signer from '../Signer/Embedded';

import AccountStore from './accountStore';
import styles from './parityBar.css';

const LS_STORE_KEY = '_parity::parityBar';
const DEFAULT_POSITION = { right: '1em', bottom: '2.5em' };
const DISPLAY_ACCOUNTS = 'accounts';
const DISPLAY_SIGNER = 'signer';

@observer
class ParityBar extends Component {
  app = null;
  measures = null;
  moving = false;

  static contextTypes = {
    api: PropTypes.object.isRequired
  };

  static propTypes = {
    dapp: PropTypes.bool,
    externalLink: PropTypes.string,
    pending: PropTypes.array
  };

  state = {
    displayType: DISPLAY_SIGNER,
    moving: false,
    opened: false,
    position: DEFAULT_POSITION
  };

  constructor (props) {
    super(props);

    this.debouncedMouseMove = throttle(
      this._onMouseMove,
      40,
      { leading: true, trailing: true }
    );
  }

  componentWillMount () {
    const { api } = this.context;

    this.accountStore = new AccountStore(api);

    // Hook to the dapp loaded event to position the
    // Parity Bar accordingly
    const dappsStore = DappsStore.get(api);

    dappsStore
      .on('loaded', (app) => {
        this.app = app;

        if (this.props.dapp) {
          this.loadPosition();
        }
      });

    if (this.props.dapp) {
      this.loadPosition();
    }
  }

  componentWillReceiveProps (nextProps) {
    const count = this.props.pending.length;
    const newCount = nextProps.pending.length;

    // Replace to default position when leaving a dapp
    if (this.props.dapp && !nextProps.dapp) {
      this.loadPosition(true);
    }

    // Load position when dapp loads
    if (!this.props.dapp && nextProps.dapp) {
      this.loadPosition();
    }

    if (count === newCount) {
      return;
    }

    if (count < newCount) {
      this.setOpened(true, DISPLAY_SIGNER);
    } else if (newCount === 0 && count === 1) {
      this.setOpened(false);
    }
  }

  setOpened (opened, displayType = DISPLAY_SIGNER) {
    this.setState({ displayType, opened });
    this.dispatchOpenEvent(opened);
  }

  dispatchOpenEvent (opened) {
    if (!this.bar) {
      return;
    }

    // Fire up custom even to support having parity bar iframed.
    const event = new CustomEvent('parity.bar.visibility', {
      bubbles: true,
      detail: { opened }
    });

    this.bar.dispatchEvent(event);
  }

  onRef = (element) => {
    this.bar = element;
  }

  render () {
    const { moving, opened, position } = this.state;

    const containerClassNames = opened
      ? [ styles.overlay ]
      : [ styles.bar ];

    if (!opened && moving) {
      containerClassNames.push(styles.moving);
    }

    const parityBgClassNames = [
      opened
        ? styles.expanded
        : styles.corner,
      styles.parityBg
    ];

    if (moving) {
      parityBgClassNames.push(styles.moving);
    }

    const parityBgStyle = {
      ...position
    };

    // Open the Signer at one of the four corners
    // of the screen
    if (opened) {
      // Set at top or bottom of the screen
      if (position.top !== undefined) {
        parityBgStyle.top = 0;
      } else {
        parityBgStyle.bottom = '2.5em';
      }

      // Set at left or right of the screen
      if (position.left !== undefined) {
        parityBgStyle.left = '1em';
      } else {
        parityBgStyle.right = '1em';
      }
    }

    return (
      <div
        className={ containerClassNames.join(' ') }
        onMouseEnter={ this.onMouseEnter }
        onMouseLeave={ this.onMouseLeave }
        onMouseMove={ this.onMouseMove }
        onMouseUp={ this.onMouseUp }
        ref={ this.onRef }
      >
        <div
          className={ parityBgClassNames.join(' ') }
          ref='container'
          style={ parityBgStyle }
        >
          {
            opened
              ? this.renderExpanded()
              : this.renderBar()
          }
        </div>
      </div>
    );
  }

  renderBar () {
    const { dapp } = this.props;

    if (!dapp) {
      return null;
    }

    return (
      <GradientBg className={ styles.cornercolor }>
        <Button
          className={ styles.iconButton }
          icon={
            <IdentityIcon
              address={ this.accountStore.defaultAccount }
              center
              inline
            />
          }
          onClick={ this.toggleAccountsDisplay }
        />
        {
          this.renderLink(
            <Button
              className={ styles.parityButton }
              icon={
                <img
                  className={ styles.parityIcon }
                  src={ imagesEthcoreBlock }
                />
              }
              label={
                this.renderLabel(
                  <FormattedMessage
                    id='parityBar.label.parity'
                    defaultMessage='Parity'
                  />
                )
              }
            />
          )
        }
        <Button
          className={ styles.button }
          icon={ <FingerprintIcon /> }
          label={ this.renderSignerLabel() }
          onClick={ this.toggleSignerDisplay }
        />
        { this.renderDrag() }
      </GradientBg>
    );
  }

  renderDrag () {
    const dragButtonClasses = [ styles.dragButton ];

    if (this.state.moving) {
      dragButtonClasses.push(styles.moving);
    }

    return (
      <div
        className={ styles.moveIcon }
        onMouseDown={ this.onMouseDown }
      >
        <div
          className={ dragButtonClasses.join(' ') }
          ref='dragButton'
        />
      </div>
    );
  }

  renderLink (button) {
    const { externalLink } = this.props;

    if (!externalLink) {
      return (
        <Link to='/'>
          { button }
        </Link>
      );
    }

    return (
      <a
        href={ externalLink }
        target='_parent'
      >
        { button }
      </a>
    );
  }

  renderExpanded () {
    const { externalLink } = this.props;
    const { displayType } = this.state;

    return (
      <div className={ styles.container }>
        <GradientBg className={ styles.header }>
          <div className={ styles.title }>
            <ContainerTitle
              title={
                displayType === DISPLAY_ACCOUNTS
                  ? (
                    <FormattedMessage
                      id='parityBar.title.accounts'
                      defaultMessage='Default Account'
                    />
                  )
                  : (
                    <FormattedMessage
                      id='parityBar.title.signer'
                      defaultMessage='Parity Signer: Pending'
                    />
                  )
              }
            />
          </div>
          <div className={ styles.actions }>
            <Button
              icon={ <CancelIcon /> }
              label={
                <FormattedMessage
                  id='parityBar.button.close'
                  defaultMessage='Close'
                />
              }
              onClick={ this.toggleSignerDisplay }
            />
          </div>
        </GradientBg>
        <div className={ styles.content }>
          {
            displayType === DISPLAY_ACCOUNTS
              ? (
                <SelectionList
                  className={ styles.accountsSection }
                  items={ this.accountStore.accounts }
                  noStretch
                  onSelectClick={ this.onMakeDefault }
                  renderItem={ this.renderAccount }
                />
              )
              : (
                <Signer externalLink={ externalLink } />
              )
          }
        </div>
      </div>
    );
  }

  onMakeDefault = (account) => {
    this.toggleAccountsDisplay();

    return this.accountStore
      .makeDefaultAccount(account.address)
      .then(() => this.accountStore.loadAccounts());
  }

  renderAccount = (account) => {
    return (
      <AccountCard
        account={ account }
      />
    );
  }

  renderLabel (name, bubble) {
    return (
      <div className={ styles.label }>
        <div className={ styles.labelText }>
          { name }
        </div>
        { bubble }
      </div>
    );
  }

  renderSignerLabel () {
    const { pending } = this.props;
    let bubble = null;

    if (pending && pending.length) {
      bubble = (
        <Badge
          color='red'
          className={ styles.labelBubble }
          value={ pending.length }
        />
      );
    }

    return this.renderLabel(
      <FormattedMessage
        id='parityBar.label.signer'
        defaultMessage='Signer'
      />,
      bubble
    );
  }

  getHorizontal (x) {
    const { page, button, container } = this.measures;

    const left = x - button.offset.left;
    const centerX = left + container.width / 2;

    // left part of the screen
    if (centerX < page.width / 2) {
      return { left: Math.max(0, left) };
    }

    const right = page.width - x - button.offset.right;

    return { right: Math.max(0, right) };
  }

  getVertical (y) {
    const STICKY_SIZE = 75;
    const { page, button, container } = this.measures;

    const top = y - button.offset.top;
    const centerY = top + container.height / 2;

    // top part of the screen
    if (centerY < page.height / 2) {
      // Add Sticky edges
      const stickyTop = top < STICKY_SIZE
        ? 0
        : top;

      return { top: Math.max(0, stickyTop) };
    }

    const bottom = page.height - y - button.offset.bottom;
    // Add Sticky edges
    const stickyBottom = bottom < STICKY_SIZE
      ? 0
      : bottom;

    return { bottom: Math.max(0, stickyBottom) };
  }

  getPosition (x, y) {
    if (!this.moving || !this.measures) {
      return {};
    }

    const horizontal = this.getHorizontal(x);
    const vertical = this.getVertical(y);

    const position = {
      ...horizontal,
      ...vertical
    };

    return position;
  }

  onMouseDown = () => {
    // Dispatch an open event in case in an iframe (get full w and h)
    this.dispatchOpenEvent(true);

    window.setTimeout(() => {
      const containerElt = ReactDOM.findDOMNode(this.refs.container);
      const dragButtonElt = ReactDOM.findDOMNode(this.refs.dragButton);

      if (!containerElt || !dragButtonElt) {
        console.warn(containerElt ? 'drag button' : 'container', 'not found...');
        return;
      }

      const bodyRect = document.body.getBoundingClientRect();
      const containerRect = containerElt.getBoundingClientRect();
      const buttonRect = dragButtonElt.getBoundingClientRect();

      const buttonOffset = {
        top: (buttonRect.top + buttonRect.height / 2) - containerRect.top,
        left: (buttonRect.left + buttonRect.width / 2) - containerRect.left
      };

      buttonOffset.bottom = containerRect.height - buttonOffset.top;
      buttonOffset.right = containerRect.width - buttonOffset.left;

      const button = {
        offset: buttonOffset,
        height: buttonRect.height,
        width: buttonRect.width
      };

      const container = {
        height: containerRect.height,
        width: containerRect.width
      };

      const page = {
        height: bodyRect.height,
        width: bodyRect.width
      };

      this.moving = true;
      this.measures = {
        button,
        container,
        page
      };

      this.setMovingState(true);
    }, 50);
  }

  onMouseEnter = (event) => {
    if (!this.moving) {
      return;
    }

    const { buttons } = event;

    // If no left-click, stop move
    if (buttons !== 1) {
      this.onMouseUp(event);
    }
  }

  onMouseLeave = (event) => {
    if (!this.moving) {
      return;
    }

    event.stopPropagation();
    event.preventDefault();
  }

  onMouseMove = (event) => {
    const { pageX, pageY } = event;

    // this._onMouseMove({ pageX, pageY });
    this.debouncedMouseMove({ pageX, pageY });

    event.stopPropagation();
    event.preventDefault();
  }

  _onMouseMove = (event) => {
    if (!this.moving) {
      return;
    }

    const { pageX, pageY } = event;
    const position = this.getPosition(pageX, pageY);

    this.setState({ position });
  }

  onMouseUp = (event) => {
    if (!this.moving) {
      return;
    }

    const { pageX, pageY } = event;
    const position = this.getPosition(pageX, pageY);

    // Stick to bottom or top
    if (position.top !== undefined) {
      position.top = 0;
    } else {
      position.bottom = 0;
    }

    // Stick to bottom or top
    if (position.left !== undefined) {
      position.left = '1em';
    } else {
      position.right = '1em';
    }

    this.moving = false;
    this.setMovingState(false, { position });
    this.savePosition(position);
  }

  toggleAccountsDisplay = () => {
    const { opened } = this.state;

    this.setOpened(!opened, DISPLAY_ACCOUNTS);
  }

  toggleSignerDisplay = () => {
    const { opened } = this.state;

    this.setOpened(!opened, DISPLAY_SIGNER);
  }

  get config () {
    const config = store.get(LS_STORE_KEY);

    if (typeof config === 'string') {
      try {
        return JSON.parse(config);
      } catch (e) {
        return {};
      }
    }

    return config || {};
  }

  /**
   * Return the config key for the current view.
   * If inside a dapp, should be the dapp id.
   * Otherwise, try to get the current hostname.
   */
  getConfigKey () {
    const { app } = this;

    if (app && app.id) {
      return app.id;
    }

    return window.location.hostname;
  }

  loadPosition (loadDefault = false) {
    if (loadDefault) {
      return this.setState({ position: DEFAULT_POSITION });
    }

    const { app, config } = this;
    const configKey = this.getConfigKey();

    if (config[configKey]) {
      return this.setState({ position: config[configKey] });
    }

    if (app && app.position) {
      const position = this.stringToPosition(app.position);

      return this.setState({ position });
    }

    return this.setState({ position: DEFAULT_POSITION });
  }

  savePosition (position) {
    const { config } = this;
    const configKey = this.getConfigKey();

    config[configKey] = position;
    store.set(LS_STORE_KEY, config);
  }

  stringToPosition (value) {
    switch (value) {
      case 'top-left':
        return {
          left: '1em',
          top: 0
        };

      case 'top-right':
        return {
          right: '1em',
          top: 0
        };

      case 'bottom-left':
        return {
          bottom: 0,
          left: '1em'
        };

      case 'bottom-right':
      default:
        return DEFAULT_POSITION;
    }
  }

  setMovingState (moving, extras = {}) {
    this.setState({ moving, ...extras });

    // Trigger an open event if it's moving
    this.dispatchOpenEvent(moving);
  }
}

function mapStateToProps (state) {
  const { pending } = state.signer;

  return {
    pending
  };
}

export default connect(
  mapStateToProps,
  null
)(ParityBar);
