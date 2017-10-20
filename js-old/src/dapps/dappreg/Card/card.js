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

import keycode from 'keycode';
import React, { Component, PropTypes } from 'react';
import ReactDOM from 'react-dom';

import styles from './card.css';

export default class Card extends Component {
  static propTypes = {
    children: PropTypes.any,
    dashed: PropTypes.bool,
    focus: PropTypes.bool,
    icon: PropTypes.object,
    name: PropTypes.object,
    onClick: PropTypes.func.isRequired
  };

  static defaultProps = {
    dashed: false,
    focus: false,
    name: { value: '' }
  };

  componentWillReceiveProps (nextProps) {
    if (nextProps.focus && !this.props.focus) {
      this.handleFocus();
    }
  }

  render () {
    const { children, dashed, icon, name } = this.props;

    const cardClasses = [ styles.card ];

    if (dashed) {
      cardClasses.push(styles.dashed);
    }

    return (
      <div className={ styles.container }>
        <div
          className={ cardClasses.join(' ') }
          onClick={ this.handleClick }
          onKeyPress={ this.handleKeyPress }
          ref='card'
          tabIndex={ 0 }
        >
          <div className={ styles.icon }>
            { icon }
          </div>
          <span
            className={ styles.name }
            title={ name.title || name.value }
          >
            { name.value }
          </span>
          { children }
        </div>
      </div>
    );
  }

  handleKeyPress = (event) => {
    const codeName = keycode(event);

    if (codeName === 'enter') {
      return this.handleClick();
    }

    return event;
  }

  handleFocus = () => {
    setTimeout(() => {
      const element = ReactDOM.findDOMNode(this.refs.card);

      element && element.focus();
    }, 50);
  }

  handleClick = () => {
    this.props.onClick();
  }
}
