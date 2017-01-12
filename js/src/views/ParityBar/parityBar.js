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
import { debounce } from 'lodash';

import { CancelIcon, FingerprintIcon, MoveIcon } from '~/ui/Icons';
import { Badge, Button, ContainerTitle, ParityBackground } from '~/ui';
import { Embedded as Signer } from '../Signer';

import imagesEthcoreBlock from '../../../assets/images/parity-logo-white-no-text.svg';
import styles from './parityBar.css';

class ParityBar extends Component {
  moving = false;
  offset = null;

  static propTypes = {
    pending: PropTypes.array,
    dapp: PropTypes.bool
  };

  state = {
    moving: false,
    opened: false,
    top: false,
    x: 0,
    y: 0
  };

  constructor (props) {
    super(props);

    this.debouncedMouseMove = debounce(this._onMouseMove, 40, { leading: true });
  }

  componentWillReceiveProps (nextProps) {
    const count = this.props.pending.length;
    const newCount = nextProps.pending.length;

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
    const { moving, opened, top, x, y } = this.state;
    const position = this.getPosition(x, y);

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

    const parityBgStyle = opened
      ? {
        top: top ? 0 : undefined,
        bottom: top ? undefined : 0
      }
      : {
        left: position.x,
        top: position.y
      };

    return (
      <div
        className={ containerClassNames.join(' ') }
        onMouseEnter={ this.onMouseEnter }
        onMouseMove={ this.onMouseMove }
        onMouseUp={ this.onMouseUp }
      >
        <ParityBackground
          className={ [ parityBgClassName, styles.parityBg ].join(' ') }
          ref='container'
          style={ parityBgStyle }
        >
          { content }
        </ParityBackground>
      </div>
    );
  }

  getPosition (_x, _y) {
    if (!this.moving || !this.offset) {
      return { x: _x, y: _y };
    }

    const { pageWidth = 0, pageHeight = 0, width = 0, height = 0 } = this.offset;

    const maxX = pageWidth - width;
    const maxY = pageHeight - height;

    const x = Math.min(maxX, Math.max(0, _x));
    const y = Math.min(maxY, Math.max(0, _y));

    if (y < 75) {
      return { x, y: 0 };
    }

    if (maxY - y < 75) {
      return { x, y: maxY };
    }

    return { x, y };
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
          <MoveIcon />
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

  onMouseDown = (event) => {
    const container = ReactDOM.findDOMNode(this.refs.container);

    if (!container) {
      return;
    }

    const { left, top, width, height } = container.getBoundingClientRect();
    const { clientX, clientY } = event;
    const bodyRect = document.body.getBoundingClientRect();

    const pageHeight = bodyRect.height;
    const pageWidth = bodyRect.width;

    this.moving = true;
    this.offset = { x: clientX - left, y: clientY - top, pageWidth, pageHeight, width, height };

    this.setState({ moving: true });
  }

  onMouseEnter = (event) => {
    if (!this.moving) {
      return;
    }

    const { buttons } = event;

    // If no left-click, stop move
    if (buttons !== 1) {
      this.onMouseUp();
    }
  }

  onMouseMove = (event) => {
    const { pageX, pageY } = event;
    this._onMouseMove({ pageX, pageY });
    // this.debouncedMouseMove({ pageX, pageY });

    event.stopPropagation();
    event.preventDefault();
  }

  _onMouseMove = (event) => {
    if (!this.moving) {
      return;
    }

    const { pageX, pageY } = event;
    const { x = 0, y = 0 } = this.offset;

    this.setState({ x: pageX - x, y: pageY - y });
  }

  onMouseUp = (event) => {
    if (!this.moving) {
      return;
    }

    const { height, width, pageHeight, pageWidth } = this.offset;
    const { x } = this.getPosition(this.state.x, this.state.y);
    const { y } = this.state;

    const top = y < (pageHeight / 2);

    const nextX = x < 25
      ? 25
      : x > (pageWidth - width - 25) ? x - 25 : x;

    const nextY = top
      ? 0
      : (pageHeight - height);

    this.moving = false;
    this.setState({ moving: false, top, x: nextX, y: nextY });
  }

  toggleDisplay = () => {
    const { opened } = this.state;

    this.setState({
      opened: !opened
    });
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
