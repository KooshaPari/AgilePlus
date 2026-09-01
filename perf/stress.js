import { listFeatures, coreThresholds } from './lib/core.js';

export const options = {
  scenarios: {
    bounded_stress: {
      executor: 'ramping-vus',
      startVUs: 1,
      stages: [
        { duration: '10s', target: 5 },
        { duration: '20s', target: 10 },
        { duration: '10s', target: 0 },
      ],
      gracefulRampDown: '5s',
      gracefulStop: '5s',
    },
  },
  thresholds: coreThresholds,
};

export default function () {
  listFeatures();
}
