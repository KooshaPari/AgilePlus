import { check } from 'k6';
import grpc from 'k6/net/grpc';
import { Rate } from 'k6/metrics';

const CORE_ADDRESS = (__ENV.AGILEPLUS_GRPC_ADDRESS || '').trim();
if (!CORE_ADDRESS) {
  throw new Error('AGILEPLUS_GRPC_ADDRESS must be set to the harness-owned core endpoint');
}
const LIST_FEATURES = 'agileplus.v1.AgilePlusCoreService/ListFeatures';

const client = new grpc.Client();
client.load(['../proto'], 'agileplus/v1/core.proto');

export const requestErrors = new Rate('request_errors');

export const coreThresholds = {
  request_errors: ['rate<=0.01'],
  grpc_req_duration: ['p(95)<250'],
};

export function listFeatures() {
  let connected = false;
  let response;

  try {
    client.connect(CORE_ADDRESS, { plaintext: true });
    connected = true;
    response = client.invoke(LIST_FEATURES, { stateFilter: '' });
  } catch (_error) {
    requestErrors.add(true);
    check(null, {
      'ListFeatures completes without a transport error': () => false,
    });
    return;
  } finally {
    if (connected) {
      client.close();
    }
  }

  const passed = check(response, {
    'ListFeatures returns gRPC OK': (result) => result.status === grpc.StatusOK,
    'ListFeatures returns a feature list': (result) =>
      result.message !== null && Array.isArray(result.message.features),
  });
  requestErrors.add(!passed);
}
