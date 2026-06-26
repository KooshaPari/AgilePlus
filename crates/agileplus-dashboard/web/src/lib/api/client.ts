import axios, { type AxiosInstance } from 'axios';
import type {
  EpicsStoriesResponse,
  WorkPackagesResponse,
} from '../../types/api';
import { API_TIMEOUT_MS, getApiBase } from './config';

let client: AxiosInstance | null = null;

/** Shared axios instance targeting agileplus-api. */
export function getApiClient(): AxiosInstance {
  if (!client) {
    client = axios.create({
      baseURL: getApiBase(),
      timeout: API_TIMEOUT_MS,
      headers: { Accept: 'application/json' },
    });
  }
  return client;
}

/** Dashboard epics + stories (GET /api/dashboard/epics-stories.json). */
export async function fetchDashboardEpicsStories(): Promise<EpicsStoriesResponse> {
  const { data } = await getApiClient().get<EpicsStoriesResponse>(
    '/api/dashboard/epics-stories.json',
  );
  return data;
}

/** All work packages (GET /api/dashboard/work-packages.json). */
export async function fetchDashboardWorkPackages(): Promise<WorkPackagesResponse> {
  const { data } = await getApiClient().get<WorkPackagesResponse>(
    '/api/dashboard/work-packages.json',
  );
  return data;
}
