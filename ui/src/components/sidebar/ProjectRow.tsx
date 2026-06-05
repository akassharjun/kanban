import { useState } from "react";
import { NavLink, useNavigate, useParams } from "react-router";
import type { ProjectDto } from "@/data/bindings";
import { useApply } from "@/data/mutations";
import { ops } from "@/data/ops";

export interface ProjectRowProps {
  project: ProjectDto;
}

export function ProjectRow({ project }: ProjectRowProps) {
  const apply = useApply();
  const navigate = useNavigate();
  const { prefix: activePrefix } = useParams();
  const isActive = activePrefix === project.prefix;

  const [menuOpen, setMenuOpen] = useState(false);
  const [renaming, setRenaming] = useState(false);
  const [confirmingDelete, setConfirmingDelete] = useState(false);

  const closeMenu = () => {
    setMenuOpen(false);
    setConfirmingDelete(false);
  };

  const commitRename = (raw: string) => {
    const trimmed = raw.trim();
    if (trimmed && trimmed !== project.name) {
      apply.mutate(ops.updateProject({ id: project.id, name: trimmed }));
    }
    setRenaming(false);
  };

  const onArchive = () => {
    apply.mutate(ops.archiveProject({ id: project.id }));
    closeMenu();
    if (isActive) navigate("/");
  };

  const onDelete = () => {
    apply.mutate(ops.deleteProject({ id: project.id }));
    closeMenu();
    if (isActive) navigate("/");
  };

  if (renaming) {
    return (
      <div className="flex items-center px-3 py-1.5">
        <input
          aria-label="Rename project"
          autoFocus
          defaultValue={project.name}
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              e.preventDefault();
              commitRename(e.currentTarget.value);
            } else if (e.key === "Escape") {
              e.preventDefault();
              setRenaming(false);
            }
          }}
          onBlur={(e) => commitRename(e.currentTarget.value)}
          className="w-full rounded-md border border-black/10 bg-white px-2 py-1 text-sm dark:border-white/10 dark:bg-neutral-800"
        />
      </div>
    );
  }

  return (
    <div className="group relative flex items-center">
      <NavLink
        to={`/p/${project.prefix}`}
        className={({ isActive: linkActive }) =>
          `flex flex-1 items-baseline gap-2 rounded-md px-3 py-1.5 text-sm ${
            linkActive
              ? "bg-neutral-900 text-white dark:bg-white dark:text-neutral-900"
              : "text-neutral-700 hover:bg-black/5 dark:text-neutral-300 dark:hover:bg-white/10"
          }`
        }
      >
        <span className="font-mono text-xs opacity-70">{project.prefix}</span>
        <span className="truncate">{project.name}</span>
      </NavLink>

      <button
        type="button"
        aria-label={`Actions for ${project.prefix}`}
        aria-haspopup="menu"
        aria-expanded={menuOpen}
        onClick={() => {
          setMenuOpen((o) => !o);
          setConfirmingDelete(false);
        }}
        className="ml-1 rounded px-1.5 text-neutral-500 hover:bg-black/5 dark:hover:bg-white/10"
      >
        ⋯
      </button>

      {menuOpen && (
        <div className="absolute right-0 top-full z-10 mt-1 flex w-36 flex-col rounded-md border border-black/10 bg-white py-1 text-sm shadow-lg dark:border-white/10 dark:bg-neutral-900">
          <button
            type="button"
            onClick={() => {
              setRenaming(true);
              closeMenu();
            }}
            className="px-3 py-1.5 text-left hover:bg-black/5 dark:hover:bg-white/10"
          >
            Rename
          </button>
          <button
            type="button"
            onClick={onArchive}
            className="px-3 py-1.5 text-left hover:bg-black/5 dark:hover:bg-white/10"
          >
            Archive
          </button>
          {confirmingDelete ? (
            <button
              type="button"
              onClick={onDelete}
              className="px-3 py-1.5 text-left text-red-600 hover:bg-red-500/10"
            >
              Confirm delete
            </button>
          ) : (
            <button
              type="button"
              onClick={() => setConfirmingDelete(true)}
              className="px-3 py-1.5 text-left text-red-600 hover:bg-red-500/10"
            >
              Delete
            </button>
          )}
        </div>
      )}
    </div>
  );
}
