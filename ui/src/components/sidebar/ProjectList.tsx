import { NavLink } from "react-router";
import { useProjects } from "@/data/queries";

export function ProjectList() {
  const { data, isLoading, isError } = useProjects();
  if (isLoading) return <p className="px-2 text-xs text-neutral-500">loading…</p>;
  if (isError) return <p className="px-2 text-xs text-red-500">couldn't load projects</p>;
  if (!data?.length) return <p className="px-2 text-xs text-neutral-500">no projects yet</p>;

  return (
    <nav className="flex flex-col gap-1">
      {data.map((p) => (
        <NavLink
          key={p.id}
          to={`/p/${p.prefix}`}
          className={({ isActive }) =>
            `flex items-baseline gap-2 rounded-md px-3 py-1.5 text-sm ${
              isActive
                ? "bg-neutral-900 text-white dark:bg-white dark:text-neutral-900"
                : "text-neutral-700 hover:bg-black/5 dark:text-neutral-300 dark:hover:bg-white/10"
            }`
          }
        >
          <span className="font-mono text-xs opacity-70">{p.prefix}</span>
          <span className="truncate">{p.name}</span>
        </NavLink>
      ))}
    </nav>
  );
}
