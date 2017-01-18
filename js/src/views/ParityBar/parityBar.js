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
import { Link } from 'react-router';
import { connect } from 'react-redux';
import { throttle } from 'lodash';
import store from 'store';

import { CancelIcon, FingerprintIcon } from '~/ui/Icons';
import { Badge, Button, ContainerTitle, ParityBackground } from '~/ui';
import { Embedded as Signer } from '../Signer';

import imagesEthcoreBlock from '../../../assets/images/parity-logo-white-no-text.svg';
import styles from './parityBar.css';

const LS_STORE_KEY = '_parity::parityBar';
const DEFAULT_POSITION = { right: '1em', bottom: 0 };

class ParityBar extends Component {
  measures = null;
  moving = false;

  static propTypes = {
    dapp: PropTypes.bool,
    hash: PropTypes.string,
    pending: PropTypes.array,
    root: PropTypes.string
  };

  state = {
    moving: false,
    opened: false,
    position: DEFAULT_POSITION
  };

  getAppId (props = this.props) {
    const { hash = '-1' } = props;

    return hash;
  }

  constructor (props) {
    super(props);

    this.debouncedMouseMove = throttle(
      this._onMouseMove,
      40,
      { leading: true, trailing: true }
    );
  }

  componentWillMount () {
    // Load the saved position of the Parity Bar
    this.loadPosition();
  }

  componentWillReceiveProps (nextProps) {
    const count = this.props.pending.length;
    const newCount = nextProps.pending.length;

    // Reload the Bar position when changing dapps
    if (nextProps.dapp && nextProps.hash !== this.props.hash) {
      this.loadPosition(nextProps);
    }

    if (count === newCount) {
      return;
    }

    if (count < newCount) {
      this.setState({ opened: true });
    } else if (newCount === 0 && count === 1) {
      this.setState({ opened: false });
    }
  }

  render () {
    const { moving, opened, position } = this.state;

    const content = opened
      ? this.renderExpanded()
      : this.renderBar();

    const containerClassNames = opened
      ? [ styles.overlay ]
      : [ styles.bar ];

    if (!opened && moving) {
      containerClassNames.push(styles.moving);
    }

    const parityBgClassName = opened
      ? styles.expanded
      : styles.corner;

    const parityBgClassNames = [ parityBgClassName, styles.parityBg ];

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
        parityBgStyle.bottom = 0;
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
      >
        <ParityBackground
          className={ parityBgClassNames.join(' ') }
          ref='container'
          style={ parityBgStyle }
        >
          { content }
        </ParityBackground>
      </div>
    );
  }

  renderBar () {
    const { dapp } = this.props;

    if (!dapp) {
      return null;
    }

    const parityIcon = (
      <img
        src={ imagesEthcoreBlock }
        className={ styles.parityIcon } />
    );

    const dragButtonClasses = [ styles.dragButton ];

    if (this.state.moving) {
      dragButtonClasses.push(styles.moving);
    }

    return (
      <div className={ styles.cornercolor }>
        <Link to='/apps'>
          <Button
            className={ styles.parityButton }
            icon={ parityIcon }
            label={ this.renderLabel('Parity') } />
        </Link>
        <Button
          className={ styles.button }
          icon={ <FingerprintIcon /> }
          label={ this.renderSignerLabel() }
          onClick={ this.toggleDisplay } />

        <div
          className={ styles.moveIcon }
          onMouseDown={ this.onMouseDown }
        >
          <div
            className={ dragButtonClasses.join(' ') }
            ref='dragButton'
          />
        </div>
      </div>
    );
  }

  renderExpanded () {
    return (
      <div>
        <div className={ styles.header }>
          <div className={ styles.title }>
            <ContainerTitle title='Parity Signer: Pending' />
          </div>
          <div className={ styles.actions }>
            <Button
              icon={ <CancelIcon /> }
              label='Close'
              onClick={ this.toggleDisplay } />
          </div>
        </div>
        <div className={ styles.content }>
          <Signer />
        </div>
      </div>
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
          value={ pending.length } />
      );
    }

    return this.renderLabel('Signer', bubble);
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

  onMouseDown = (event) => {
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

    this.setState({ moving: true });
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

    this.moving = false;
    this.setState({ moving: false, position });
    this.savePosition(position);
  }

  toggleDisplay = () => {
    const { opened } = this.state;

    this.setState({
      opened: !opened
    });
  }

  get config () {
    let config;

    try {
      config = JSON.parse(store.get(LS_STORE_KEY));
    } catch (error) {
      config = {};
    }

    return config;
  }

  loadPosition (props = this.props) {
    const { config } = this;
    const appId = this.getAppId(props);

    const position = config[appId] || { ...DEFAULT_POSITION };
    this.setState({ position });
  }

  savePosition (position) {
    const { config } = this;
    const appId = this.getAppId();

    config[appId] = position;

    store.set(LS_STORE_KEY, JSON.stringify(config));
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
