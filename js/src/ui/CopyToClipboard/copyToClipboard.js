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

import React, { Component, PropTypes } from 'react';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { IconButton } from 'material-ui';
import Clipboard from 'react-copy-to-clipboard';
import CopyIcon from 'material-ui/svg-icons/content/content-copy';
import Theme from '../Theme';

import { showSnackbar } from '~/redux/providers/snackbarActions';

const { textColor, disabledTextColor } = Theme.flatButton;

import styles from './copyToClipboard.css';

class CopyToClipboard extends Component {
  static propTypes = {
    showSnackbar: PropTypes.func.isRequired,
    data: PropTypes.string.isRequired,

    onCopy: PropTypes.func,
    size: PropTypes.number, // in px
    cooldown: PropTypes.number // in ms
  };

  static defaultProps = {
    className: '',
    onCopy: () => {},
    size: 16,
    cooldown: 1000
  };

  state = {
    copied: false,
    timeoutId: null
  };

  componentWillUnmount () {
    const { timeoutId } = this.state;

    if (timeoutId) {
      window.clearTimeout(timeoutId);
    }
  }

  render () {
    const { data, size } = this.props;
    const { copied } = this.state;

    return (
      <Clipboard onCopy={ this.onCopy } text={ data }>
        <div className={ styles.wrapper }>
          <IconButton
            disableTouchRipple
            style={ { width: size, height: size, padding: '0' } }
            iconStyle={ { width: size, height: size } }
          >
            <CopyIcon color={ copied ? disabledTextColor : textColor } />
          </IconButton>
        </div>
      </Clipboard>
    );
  }

  onCopy = () => {
    const { data, onCopy, cooldown, showSnackbar } = this.props;
    const message = (
      <div className={ styles.container }>
        <span>copied </span>
        <code className={ styles.data }> { data } </code>
        <span> to clipboard</span>
      </div>
    );

    this.setState({
      copied: true,
      timeoutId: setTimeout(() => {
        this.setState({ copied: false, timeoutId: null });
      }, cooldown)
    });

    showSnackbar(message, cooldown);
    onCopy();
  }
}

function mapDispatchToProps (dispatch) {
  return bindActionCreators({
    showSnackbar
  }, dispatch);
}

export default connect(
  null,
  mapDispatchToProps
)(CopyToClipboard);
