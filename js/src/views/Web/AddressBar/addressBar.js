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
import Refresh from 'material-ui/svg-icons/navigation/refresh';
import Close from 'material-ui/svg-icons/navigation/close';
import Subdirectory from 'material-ui/svg-icons/navigation/subdirectory-arrow-left';

import { Button } from '~/ui';

const KEY_ESC = 27;
const KEY_ENTER = 13;

export default class AddressBar extends Component {
  static propTypes = {
    className: PropTypes.string,
    isLoading: PropTypes.bool.isRequired,
    onChange: PropTypes.func.isRequired,
    onRefresh: PropTypes.func.isRequired,
    url: PropTypes.string.isRequired
  };

  state = {
    currentUrl: this.props.url
  };

  componentWillReceiveProps (nextProps) {
    if (this.props.url === nextProps.url) {
      return;
    }

    this.setState({
      currentUrl: nextProps.url
    });
  }

  isPristine () {
    return this.state.currentUrl === this.props.url;
  }

  render () {
    const { isLoading } = this.props;
    const { currentUrl } = this.state;
    const isPristine = this.isPristine();

    return (
      <div className={ this.props.className }>
        <Button
          disabled={ isLoading }
          icon={
            isLoading
              ? <Close />
              : <Refresh />
          }
          onClick={ this.onGo }
        />
        <input
          onChange={ this.onUpdateUrl }
          onKeyDown={ this.onKey }
          type='text'
          value={ currentUrl }
        />
        <Button
          disabled={ isPristine }
          icon={ <Subdirectory /> }
          onClick={ this.onGo }
        />
      </div>
    );
  }

  onUpdateUrl = (ev) => {
    this.setState({
      currentUrl: ev.target.value
    });
  };

  onKey = (ev) => {
    const key = ev.which;

    if (key === KEY_ESC) {
      this.setState({
        currentUrl: this.props.url
      });
      return;
    }

    if (key === KEY_ENTER) {
      this.onGo();
      return;
    }
  };

  onGo = () => {
    if (this.isPristine()) {
      this.props.onRefresh();
    } else {
      this.props.onChange(this.state.currentUrl);
    }
  };
}
