import React, { Component, PropTypes } from 'react';
import { Link } from 'react-router';

import Container, { Title } from '../../../ui/Container';
import IdentityIcon from '../../../ui/IdentityIcon';

export default class Summary extends Component {
  static contextTypes = {
    api: React.PropTypes.object.isRequired
  }

  static propTypes = {
    contract: PropTypes.object.isRequired,
    children: PropTypes.node
  }

  render () {
    const contract = this.props.contract;

    if (!contract) {
      return null;
    }

    const viewLink = `/app/${contract.address}`;

    return (
      <Container>
        <IdentityIcon
          address={ contract.address } />
        <Title
          title={ <Link to={ viewLink }>{ contract.name || 'Unnamed' }</Link> }
          byline={ contract.address } />
        { this.props.children }
      </Container>
    );
  }
}
