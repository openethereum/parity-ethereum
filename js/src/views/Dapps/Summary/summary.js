import React, { Component, PropTypes } from 'react';
import { Link } from 'react-router';

import Container, { Title } from '../../../ui/Container';
import IdentityIcon from '../../../ui/IdentityIcon';

export default class Summary extends Component {
  static contextTypes = {
    api: React.PropTypes.object
  }

  static propTypes = {
    app: PropTypes.object.isRequired,
    children: PropTypes.node
  }

  render () {
    const { app } = this.props;

    if (!app) {
      return null;
    }

    const url = `/app/${app.url}`;

    return (
      <Container>
        <IdentityIcon
          address={ app.address } />
        <Title
          title={ <Link to={ url }>{ app.name }</Link> }
          byline={ app.description } />
        { this.props.children }
      </Container>
    );
  }
}
