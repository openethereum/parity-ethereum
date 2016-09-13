import blockies from 'blockies';

export function createIdentityImg (address, scale = 7) {
  return blockies({
    seed: (address || '').toLowerCase(),
    size: 8,
    scale
  }).toDataURL();
}
