import React, { Component, PropTypes } from 'react';
import AvPause from 'material-ui/svg-icons/av/pause';
import AvPlay from 'material-ui/svg-icons/av/play-arrow';
import AvReplay from 'material-ui/svg-icons/av/replay';

import { Container, ContainerTitle } from '../../../../ui';

import styles from './Debug.css';

export default class Debug extends Component {
  static propTypes = {
    actions: PropTypes.shape({
      clearStatusLogs: PropTypes.func.isRequired,
      toggleStatusLogs: PropTypes.func.isRequired
    }).isRequired,
    nodeStatus: PropTypes.object.isRequired
  }

  render () {
    const { nodeStatus } = this.props;
    const { devLogsLevels } = nodeStatus;

    return (
      <Container>
        <ContainerTitle
          title='Node Logs' />
        { this.renderActions() }
        <h2 className={ styles.subheader }>
          { devLogsLevels || '-' }
        </h2>
        <div className={ styles.logs }>
          { this.renderLogs() }
        </div>
      </Container>
    );
  }

  renderLogs () {
    const { nodeStatus } = this.props;
    const { devLogs } = nodeStatus;

    if (!devLogs) {
      return null;
    }

    return devLogs.map((log, idx) => (
      <pre className={ styles.log } key={ idx }>
        { log }
      </pre>
    ));
  }

  renderActions () {
    const { devLogsEnabled } = this.props.nodeStatus;
    const toggleButton = devLogsEnabled
      ? <AvPause />
      : <AvPlay />;

    return (
      <div className={ styles.actions }>
        <a onClick={ this.toggle }>{ toggleButton }</a>
        <a onClick={ this.clear }><AvReplay /></a>
      </div>
    );
  }

  clear = () => {
    const { clearStatusLogs } = this.props.actions;

    clearStatusLogs();
  }

  toggle = () => {
    const { devLogsEnabled } = this.props.nodeStatus;
    const { toggleStatusLogs } = this.props.actions;

    toggleStatusLogs(!devLogsEnabled);
  }
}
