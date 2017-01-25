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

import { Dialog } from 'material-ui';
import React, { Component, PropTypes } from 'react';
import ReactDOM from 'react-dom';
import { connect } from 'react-redux';

import { nodeOrStringProptype } from '~/util/proptypes';

import Container from '../Container';
import Title from './Title';

const ACTIONS_STYLE = { borderStyle: 'none' };
const TITLE_STYLE = { borderStyle: 'none' };
const DIALOG_STYLE = { paddingTop: '1px' };

import styles from './modal.css';

class Modal extends Component {
  static contextTypes = {
    muiTheme: PropTypes.object.isRequired
  }

  static propTypes = {
    actions: PropTypes.node,
    busy: PropTypes.bool,
    children: PropTypes.node,
    className: PropTypes.string,
    compact: PropTypes.bool,
    current: PropTypes.number,
    settings: PropTypes.object.isRequired,
    steps: PropTypes.array,
    title: nodeOrStringProptype(),
    visible: PropTypes.bool.isRequired,
    waiting: PropTypes.array
  }

  componentDidMount () {
    const element = ReactDOM.findDOMNode(this.refs.dialog);

    if (element) {
      element.focus();
    }
  }

  render () {
    const { muiTheme } = this.context;
    const { actions, busy, children, className, current, compact, settings, steps, title, visible, waiting } = this.props;
    const contentStyle = muiTheme.parity.getBackgroundStyle(null, settings.backgroundSeed);
    const header = (
      <Title
        busy={ busy }
        current={ current }
        steps={ steps }
        title={ title }
        waiting={ waiting }
      />
    );
    const classes = `${styles.dialog} ${className}`;

    return (
      <Dialog
        actions={ actions }
        actionsContainerClassName={ styles.actions }
        actionsContainerStyle={ ACTIONS_STYLE }
        autoDetectWindowHeight={ false }
        autoScrollBodyContent
        bodyClassName={ styles.body }
        className={ classes }
        contentClassName={ styles.content }
        contentStyle={ contentStyle }
        modal
        open={ visible }
        overlayClassName={ styles.overlay }
        overlayStyle={ { transition: 'none' } }
        repositionOnUpdate={ false }
        style={ DIALOG_STYLE }
        title={ header }
        titleStyle={ TITLE_STYLE }
      >
        <Container
          compact={ compact }
          light
          ref='dialog'
          style={
            { transition: 'none' }
          }
          tabIndex={ 0 }
        >
          { children }
        </Container>
      </Dialog>
    );
  }
}

function mapStateToProps (state) {
  const { settings } = state;

  return { settings };
}

export default connect(
  mapStateToProps,
  null
)(Modal);
