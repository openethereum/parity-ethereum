import React, { Component, PropTypes } from 'react';
import { Toolbar, ToolbarGroup, ToolbarTitle } from 'material-ui/Toolbar';

import styles from './actionbar.css';

export default class Actionbar extends Component {
  static propTypes = {
    title: PropTypes.string,
    buttons: PropTypes.array,
    children: PropTypes.node
  };

  render () {
    const { children } = this.props;

    return (
      <Toolbar>
        { this.renderTitle() }
        { this.renderButtons() }
        { children }
      </Toolbar>
    );
  }

  renderButtons () {
    const { buttons } = this.props;

    if (!buttons || !buttons.length) {
      return null;
    }

    return (
      <ToolbarGroup
        className={ styles.toolbuttons }>
        { buttons }
      </ToolbarGroup>
    );
  }

  renderTitle () {
    const { title } = this.props;

    return (
      <ToolbarTitle
        className={ styles.tooltitle }
        text={ title } />
    );
  }
}
