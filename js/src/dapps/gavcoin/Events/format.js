export function formatBlockNumber (event) {
  return event.state === 'pending'
    ? 'Pending'
    : `#${event.blockNumber.toFormat()}`;
}
