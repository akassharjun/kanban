import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { NewProjectDialog } from "@/components/sidebar/NewProjectDialog";

const applyFn = vi.hoisted(() => vi.fn());
vi.mock("@/data/mutations", () => ({
  useApply: () => ({ mutate: applyFn, mutateAsync: applyFn, isPending: false }),
}));

describe("NewProjectDialog", () => {
  it("validates prefix client-side and surfaces an error", async () => {
    applyFn.mockReset();
    render(<NewProjectDialog open onOpenChange={() => {}} />);
    await userEvent.type(screen.getByLabelText(/name/i), "Auth Service");
    await userEvent.type(screen.getByLabelText(/prefix/i), "lower");
    await userEvent.click(screen.getByRole("button", { name: /create/i }));
    expect(await screen.findByText(/2-8 uppercase letters/i)).toBeInTheDocument();
    expect(applyFn).not.toHaveBeenCalled();
  });

  it("submits a CreateProject Operation when valid", async () => {
    applyFn.mockReset();
    render(<NewProjectDialog open onOpenChange={() => {}} />);
    await userEvent.type(screen.getByLabelText(/name/i), "Auth Service");
    await userEvent.type(screen.getByLabelText(/prefix/i), "AUTH");
    await userEvent.click(screen.getByRole("button", { name: /create/i }));
    expect(applyFn).toHaveBeenCalledTimes(1);
    const op = applyFn.mock.calls[0][0] as { op: string; args: { prefix: string } };
    expect(op.op).toBe("CreateProject");
    expect(op.args.prefix).toBe("AUTH");
  });
});
