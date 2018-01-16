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

import { isEqual } from 'lodash';
import React, { Component, PropTypes } from 'react';
import ReactDOM from 'react-dom';
import { Toolbar, ToolbarGroup } from 'material-ui/Toolbar';

import { nodeOrStringProptype } from '~/util/proptypes';

import styles from './actionbar.css';

export default class Actionbar extends Component {
  buttons = {};
  buttonsTooltip = {};

  static propTypes = {
    title: nodeOrStringProptype(),
    buttons: PropTypes.array,
    children: PropTypes.node,
    className: PropTypes.string,
    health: PropTypes.node
  };

  static defaultProps = {
    buttons: []
  };

  state = {
    buttons: []
  };

  componentWillMount () {
    this.setButtons(this.props);

    window.addEventListener('resize', this.checkButtonsTooltip);
  }

  componentWillUnmount () {
    window.removeEventListener('resize', this.checkButtonsTooltip);
  }

  componentWillReceiveProps (nextProps) {
    if (!isEqual(this.props.buttons, nextProps.buttons)) {
      this.setButtons(nextProps);
    }
  }

  render () {
    const { children, className } = this.props;
    const classes = `${styles.actionbar} ${className}`;

    return (
      <Toolbar className={ classes }>
        { this.renderTitle() }
        { this.renderButtons() }
        { children }
      </Toolbar>
    );
  }

  renderButtons () {
    const { buttons } = this.state;

    if (buttons.length === 0) {
      return null;
    }

    return (
      <ToolbarGroup className={ styles.toolbuttons }>
        { buttons }
      </ToolbarGroup>
    );
  }

  renderTitle () {
    const { title } = this.props;

    return (
      <h3 className={ styles.tooltitle }>
        { title }
      </h3>
    );
  }

  checkButtonsTooltip = () => {
    const buttonsTooltip = Object.keys(this.buttons)
      .reduce((buttonsTooltip, index) => {
        buttonsTooltip[index] = this.checkButtonTooltip(this.buttons[index]);
        return buttonsTooltip;
      }, {});

    if (isEqual(buttonsTooltip, this.buttonsTooltip)) {
      return;
    }

    this.buttonsTooltip = buttonsTooltip;
    this.setButtons(this.props);
  }

  checkButtonTooltip = (button) => {
    const { icon, text } = button;
    const iconBoundings = icon.getBoundingClientRect();
    const textBoundings = text.getBoundingClientRect();

    // Visible if the bottom  of the text is above the bottom of the
    // button (text is v-aligned on top)
    const isTextVisible = textBoundings.top + textBoundings.height < iconBoundings.top + iconBoundings.height;

    return !isTextVisible;
  }

  /**
   * Return the icon and text nodes of a Button
   * (and SVG/IMG for the icon next to a span node)
   */
  getIconAndTextNodes (element) {
    if (!element || !element.children || element.children.length === 0) {
      return null;
    }

    const children = Array.slice(element.children);
    const text = children.find((child) => child.nodeName.toLowerCase() === 'span');
    const icon = children.find((child) => {
      const nodeName = child.nodeName.toLowerCase();

      return nodeName === 'svg' || nodeName === 'img';
    });

    if (icon && text) {
      return { icon, text };
    }

    const result = children
      .map((child) => {
        return this.getIconAndTextNodes(child);
      })
      .filter((result) => result);

    return result.length > 0
      ? result[0]
      : null;
  }

  /**
   * Add tooltip to all Buttons
   */
  patchButton (element, extraProps) {
    if (element.type.displayName !== 'Button') {
      if (!element.props.children) {
        return element;
      }

      const children = this.patchButton(element.props.children);

      return React.cloneElement(element, {}, children);
    }

    return React.cloneElement(element, extraProps);
  }

  setButtons (props) {
    const buttons = props.buttons
      .filter((button) => button)
      .map((button, index) => {
        const ref = this.setButtonRef.bind(this, index);
        const showTooltip = this.buttonsTooltip[index];

        return this.patchButton(button, { tooltip: showTooltip, ref });
      });

    this.setState({ buttons });
  }

  setButtonRef = (index, element) => {
    const node = ReactDOM.findDOMNode(element);
    const iconAndText = this.getIconAndTextNodes(node);

    if (!iconAndText) {
      return;
    }

    if (!this.buttons[index]) {
      this.buttonsTooltip[index] = this.checkButtonTooltip(iconAndText);
      this.setButtons(this.props);
    }

    this.buttons[index] = iconAndText;
  };
}
