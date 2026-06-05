import { useParams } from "react-router";
import { Board } from "@/components/board/Board";
import { useProjects } from "@/data/queries";

export default function ProjectRoute() {
  const { prefix = "" } = useParams();
  const { data: projects } = useProjects();
  const project = projects?.find((p) => p.prefix === prefix);
  if (!project) return <div className="p-6 text-sm text-neutral-500">project not found</div>;
  return <Board projectPrefix={prefix} projectId={project.id} />;
}
