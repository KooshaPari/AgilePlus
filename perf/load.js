import { listFeatures, coreThresholds } from './lib/core.js';

export const options = {
  scenarios: {
    bounded_load: {
      executor: 'constant-vus',
      vus: 5,
      duration: '30s',
      gracefulStop: '5s',
    },
  },
  thresholds: coreThresholds,
};

export default function () {
  listFeatures();
}
