import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { DndContext } from "@dnd-kit/core";
import { SortableContext } from "@dnd-kit/sortable";
import { describe, expect, it } from "vitest";
import { IssueCard } from "@/components/board/IssueCard";
import type { IssueDto } from "@/data/bindings";

const issue: IssueDto = {
  id: "u1",
  project_id: "p1",
  seq: 12,
  identifier: "AUTH-12",
  title: "Add OAuth",
  description: null,
  status_id: "s1",
  priority: "high",
  due_date: null,
  assignee_id: null,
  sort_key: 1,
  created_at: "",
  updated_at: "",
  labels: null,
};

describe("IssueCard", () => {
  it("renders the identifier and title", () => {
    render(
      <MemoryRouter>
        <DndContext>
          <SortableContext items={[issue.id]}>
            <IssueCard projectPrefix="AUTH" issue={issue} />
          </SortableContext>
        </DndContext>
      </MemoryRouter>,
    );
    expect(screen.getByText("AUTH-12")).toBeInTheDocument();
    expect(screen.getByText("Add OAuth")).toBeInTheDocument();
    expect(screen.getByText("high")).toBeInTheDocument();
  });
});
