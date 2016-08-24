import React, { Component, PropTypes } from 'react';

import EventBuyin from './EventBuyin';

export default class Events extends Component {
  static contextTypes = {
    instance: PropTypes.object
  }

  state = {
    events: []
  }

  componentDidMount () {
    this.setupFilters();
  }

  render () {
    return (
      <div className='events'>
        { this.renderEvents() }
      </div>
    );
  }

  renderEvents () {
    const { events } = this.state;

    if (!events.length) {
      return null;
    }

    return events.map((event) => {
      switch (event.type) {
        case 'Buyin':
          return (
            <EventBuyin
              key={ event.key }
              event={ event } />
          );
      }
    });
  }

  setupFilters () {
    const { instance } = this.context;

    ['Approval', 'Buyin', 'Refund', 'Transfer', 'NewTranch'].forEach((eventName) => {
      const options = {
        fromBlock: 0,
        toBlock: 'latest' // 'pending'
      };

      instance[eventName].subscribe(options, (logs) => {
        if (!logs.length) {
          return;
        }

        console.log(logs);

        const mapped = logs.map((log) => {
          return {
            type: eventName,
            blockNumber: log.blockNumber,
            transactionHash: log.transactionHash,
            params: log.params,
            key: `${eventName}_${log.transactionHash}_${log.logIndex.toString()}`
          };
        });

        this.setState({
          events: this.state.events
            .concat(mapped)
            .sort((a, b) => {
              if (a.blockNumber.lt(b.blockNumber)) {
                return 1;
              } else if (a.blockNumber.gt(b.blockNumber)) {
                return -1;
              }

              return a.key.localeCompare(b.key);
            })
        });
      });
    });
  }
}
