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

class Requests extends Component {
  static propTypes = {
    requests: PropTypes.object.isRequired
  };

  render () {
    const { requests } = this.props;

    return (
      <div>
        { Object.keys(requests).map((requestId) => this.renderRequest(requestId, requests[requestId])) }
      </div>
    );
  }

  renderRequest (requestId, requestData) {
    return (
      <div key={ requestId }>
        <pre>{ JSON.stringify(requestData, null, 2) }</pre>
      </div>
    );
  }
}

const mapStateToProps = (state) => {
  return { requests: state.requests };
};

export default connect(mapStateToProps, null)(Requests);
