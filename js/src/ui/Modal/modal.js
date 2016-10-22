// Copyright 2015, 2016 Ethcore (UK) Ltd.
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
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';
import { Dialog } from 'material-ui';

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
    waiting: PropTypes.array,
    scroll: PropTypes.bool,
    steps: PropTypes.array,
    title: React.PropTypes.oneOfType([
      PropTypes.node, PropTypes.string
    ]),
    visible: PropTypes.bool.isRequired,
    settings: PropTypes.object.isRequired
  }

  render () {
    const { muiTheme } = this.context;
    const { actions, busy, className, current, children, compact, scroll, steps, waiting, title, visible, settings } = this.props;
    const contentStyle = muiTheme.parity.getBackgroundStyle(null, settings.backgroundSeed);
    const header = (
      <Title
        current={ current }
        busy={ busy }
        waiting={ waiting }
        steps={ steps }
        title={ title } />
    );
    const classes = `${styles.dialog} ${className}`;

    return (
      <Dialog
        className={ classes }
        actions={ actions }
        actionsContainerStyle={ ACTIONS_STYLE }
        autoDetectWindowHeight={ false }
        autoScrollBodyContent={ !!scroll }
        actionsContainerClassName={ styles.actions }
        bodyClassName={ styles.body }
        contentClassName={ styles.content }
        contentStyle={ contentStyle }
        modal
        open={ visible }
        overlayClassName={ styles.overlay }
        overlayStyle={ { transition: 'none' } }
        repositionOnUpdate={ false }
        style={ DIALOG_STYLE }
        title={ header }
        titleStyle={ TITLE_STYLE }>
        <Container light compact={ compact } style={ { transition: 'none' } }>
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

function mapDispatchToProps (dispatch) {
  return bindActionCreators({}, dispatch);
}

export default connect(
  mapStateToProps,
  mapDispatchToProps
)(Modal);
