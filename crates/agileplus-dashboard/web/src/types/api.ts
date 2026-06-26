/**
 * API response types matching agileplus-api dashboard JSON endpoints.
 */

export interface ApiEpic {
  id: number;
  title: string;
  status: string;
  requirement_id: string | null;
}

export interface ApiStory {
  id: number;
  epic_id: number | null;
  title: string;
  status: string;
  requirement_id: string | null;
}

export interface EpicsStoriesResponse {
  epics: ApiEpic[];
  stories: ApiStory[];
  epic_count?: number;
  story_count?: number;
  timestamp?: string;
  error?: string;
}

export interface ApiWorkPackage {
  id: string;
  feature_id?: number;
  title: string;
  status: string;
  priority: string;
  assignee?: string | null;
}

export interface WorkPackagesResponse {
  work_packages: ApiWorkPackage[];
  count?: number;
  timestamp?: string;
}
