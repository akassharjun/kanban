import { useParams } from "react-router";
import { useProjects } from "@/data/queries";
import { Board } from "@/components/board/Board";
import { IssuePanel } from "@/components/detail/IssuePanel";

export default function IssueRoute() {
  const { prefix = "" } = useParams();
  const { data: projects } = useProjects();
  const project = projects?.find((p) => p.prefix === prefix);
  if (!project) return null;
  return (
    <div className="relative h-full">
      <Board projectPrefix={prefix} projectId={project.id} />
      <IssuePanel />
    </div>
  );
}
