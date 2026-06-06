import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { commands } from "./bindings";
import { asApiError } from "./client";
import type {
  IssueDto,
  LabelDto,
  MemberDto,
  ProjectDto,
  Settings,
  StatusDto,
  ThemeChoice,
} from "./bindings";

/** Query-key factory — keeps invalidation keys consistent across hooks. */
export const qk = {
  projects: () => ["projects"] as const,
  project: (prefix: string) => ["projects", prefix] as const,
  issues: (prefix: string) => ["projects", prefix, "issues"] as const,
  issue: (key: string) => ["issues", key] as const,
  statuses: (prefix: string) => ["projects", prefix, "statuses"] as const,
  labels: (prefix: string) => ["projects", prefix, "labels"] as const,
  members: (prefix: string) => ["projects", prefix, "members"] as const,
  settings: () => ["settings"] as const,
} as const;

type CmdResult<T> = { status: "ok"; data: T } | { status: "error"; error: unknown };

/** Unwrap a tauri-specta `Result<T, E>` into `T`, throwing a normalized ApiError on failure. */
async function unwrap<T>(p: Promise<CmdResult<T>>): Promise<T> {
  const r = await p;
  if (r.status === "ok") return r.data;
  throw asApiError(r.error);
}

export const useProjects = () =>
  useQuery({ queryKey: qk.projects(), queryFn: () => unwrap<ProjectDto[]>(commands.listProjects()) });

export const useIssues = (prefix: string) =>
  useQuery({
    queryKey: qk.issues(prefix),
    queryFn: () => unwrap<IssueDto[]>(commands.listIssues(prefix)),
    enabled: !!prefix,
  });

export const useIssue = (key: string) =>
  useQuery({
    queryKey: qk.issue(key),
    queryFn: () => unwrap<IssueDto>(commands.getIssue(key)),
    enabled: !!key,
  });

export const useStatuses = (prefix: string) =>
  useQuery({
    queryKey: qk.statuses(prefix),
    queryFn: () => unwrap<StatusDto[]>(commands.listStatuses(prefix)),
    enabled: !!prefix,
  });

export const useLabels = (prefix: string) =>
  useQuery({
    queryKey: qk.labels(prefix),
    queryFn: () => unwrap<LabelDto[]>(commands.listLabels(prefix)),
    enabled: !!prefix,
  });

export const useMembers = (prefix: string) =>
  useQuery({
    queryKey: qk.members(prefix),
    queryFn: () => unwrap<MemberDto[]>(commands.listMembers(prefix)),
    enabled: !!prefix,
  });

export const useSettings = () =>
  useQuery({ queryKey: qk.settings(), queryFn: () => unwrap<Settings>(commands.getSettings()) });

export const useUpdateSettings = () => {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (theme: ThemeChoice) => unwrap<Settings>(commands.updateSettings(theme)),
    onSuccess: (settings) => {
      qc.setQueryData(qk.settings(), settings);
    },
  });
};
