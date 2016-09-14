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

// no need for react since not using JSX
import { Component, PropTypes } from 'react';

export default class Web3Provider extends Component {

  static childContextTypes = {
    web3: PropTypes.object.isRequired
  };

  static propTypes = {
    web3: PropTypes.object.isRequired,
    children: PropTypes.element
  };

  getChildContext () {
    return {
      web3: this.props.web3
    };
  }

  render () {
    return this.props.children;
  }

}
