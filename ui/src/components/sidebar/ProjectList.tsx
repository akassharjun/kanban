import { useProjects } from "@/data/queries";
import { ProjectRow } from "./ProjectRow";

export function ProjectList() {
  const { data, isLoading, isError } = useProjects();
  if (isLoading) return <p className="px-2 text-xs text-neutral-500">loading…</p>;
  if (isError) return <p className="px-2 text-xs text-red-500">couldn't load projects</p>;
  if (!data?.length) return <p className="px-2 text-xs text-neutral-500">no projects yet</p>;

  return (
    <nav className="flex flex-col gap-1">
      {data.map((p) => (
        <ProjectRow key={p.id} project={p} />
      ))}
    </nav>
  );
}
