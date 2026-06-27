export type AppState = {
  ring_path: string | null;
  alarm_enabled: boolean;
  source_path: string;
  save_path: string;
};

export type OpenAiConfig = {
  base_url: string;
  model: string;
  system_prompt: string;
  api_key: string;
  source_path: string;
  save_path: string;
};
export type OpenAiConfigText = {
  path: string;
  content: string;
};

export type LoginRecord = {
  email: string;
  password: string;
  updated_at: number;
};

export type JoinedClassInfo = {
  name: string;
  url: string;
};

export type OpenJudgeLoginResult = {
  personal_home_url: string;
  classes: JoinedClassInfo[];
};

export type ContestSetInfo = {
  url: string;
  title: string;
  item_class: string;
  problem_number?: string | null;
  end_time?: string | null;
  extra_text?: string | null;
};

export type GroupPageInfo = {
  group_page_url: string;
  contest_sets: ContestSetInfo[];
};

export type ClassPageInfo = {
  class_page_url: string;
  group_entry_url?: string | null;
  course_name?: string | null;
};

export type OpenClassResult = {
  class_info: ClassPageInfo;
  group_info: GroupPageInfo;
};

export type ContestProblemInfo = {
  problem_id: string;
  title: string;
  problem_url: string;
  accept_people: number;
  submission_people: number;
  solved: boolean;
};

export type ContestPageInfo = {
  contest_page_url: string;
  problems: ContestProblemInfo[];
  total_problems: number;
  solved_problems: number;
};

export type ProblemPageInfo = {
  problem_url: string;
  title?: string | null;
  submit_url?: string | null;
  time_limit?: string | null;
  memory_limit?: string | null;
  description?: string | null;
  starter_code?: string | null;
  input_spec?: string | null;
  output_spec?: string | null;
  sample_input?: string | null;
  sample_output?: string | null;
  hint?: string | null;
  tried_people: number;
  passed_people: number;
};

export type SubmitLanguageOption = {
  value: string;
  label: string;
  checked: boolean;
};

export type SubmitPageInfo = {
  page_url: string;
  submit_action_url?: string | null;
  contest_id?: string | null;
  problem_number?: string | null;
  source_encode?: string | null;
  languages: SubmitLanguageOption[];
};

export type SubmitResponse = {
  ok: boolean;
  status_code: number;
  final_url: string;
  inferred_result_url?: string | null;
};

export type ResultPageInfo = {
  page_url: string;
  solution_url?: string | null;
  submission_id?: string | null;
  status_text?: string | null;
  status_class?: string | null;
  detail_title?: string | null;
  detail_text?: string | null;
  has_result: boolean;
};

export type FavoriteFolderInfo = {
  id: number;
  name: string;
  item_count: number;
};

export type FavoriteProblemRow = {
  problem_url: string;
  title: string;
  saved_at: string;
};

export type DeadlineReminder = {
  course_name: string;
  contest_title: string;
  contest_url: string;
  deadline_text: string;
  deadline_epoch_ms: number;
};

export type AlarmTrigger = {
  reminder: DeadlineReminder;
  hours_before: number;
};

export type StorageSizes = {
  cache_bytes: number;
  app_bytes: number;
  cache_dir: string;
  project_root_dir: string;
};

export type AiChatMessage = {
  role: string;
  content: string;
};

export type JudgeResponse = {
  ok: boolean;
  status_code: number;
  body: string;
};