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

import { IconButton } from 'material-ui';
import React, { Component, PropTypes } from 'react';
import { FormattedMessage } from 'react-intl';
import Clipboard from 'react-copy-to-clipboard';
import { connect } from 'react-redux';
import { bindActionCreators } from 'redux';

import { showSnackbar } from '~/redux/providers/snackbarActions';

import { CopyIcon } from '../Icons';
import Theme from '../Theme';

import styles from './copyToClipboard.css';

const { textColor, disabledTextColor } = Theme.flatButton;

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
      <Clipboard
        onCopy={ this.onCopy }
        text={ data }
      >
        <div
          className={ styles.wrapper }
          onClick={ this.onClick }
        >
          <IconButton
            disableTouchRipple
            iconStyle={ {
              height: size,
              width: size
            } }
            style={ {
              height: size,
              padding: '0',
              width: size
            } }
          >
            <CopyIcon
              color={
                copied
                  ? disabledTextColor
                  : textColor
              }
            />
          </IconButton>
        </div>
      </Clipboard>
    );
  }

  onCopy = () => {
    const { data, onCopy, cooldown, showSnackbar } = this.props;
    const message = (
      <div className={ styles.container }>
        <FormattedMessage
          id='ui.copyToClipboard.copied'
          defaultMessage='copied {data} to clipboard'
          values={ {
            data: <code className={ styles.data }> { data } </code>
          } }
        />
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

  onClick = (event) => {
    event.stopPropagation();
    event.preventDefault();
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
