import { listFeatures, coreThresholds } from './lib/core.js';

export const options = {
  scenarios: {
    smoke: {
      executor: 'constant-vus',
      vus: 1,
      duration: '5s',
      gracefulStop: '2s',
    },
  },
  thresholds: coreThresholds,
};

export default function () {
  listFeatures();
}
