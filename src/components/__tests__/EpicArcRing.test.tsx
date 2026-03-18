import { render, screen } from "@testing-library/react";
import { EpicArcRing } from "../EpicArcRing";

describe("EpicArcRing", () => {
  it("shows percentage for in-progress epic", () => {
    render(<EpicArcRing total={10} completed={7} />);
    expect(screen.getByText("70%")).toBeInTheDocument();
  });

  it("shows checkmark for completed epic", () => {
    render(<EpicArcRing total={4} completed={4} />);
    expect(screen.getByText("✓")).toBeInTheDocument();
  });

  it("shows 0% when no tasks completed", () => {
    render(<EpicArcRing total={5} completed={0} />);
    expect(screen.getByText("0%")).toBeInTheDocument();
  });

  it("renders at small size for inline badge", () => {
    const { container } = render(
      <EpicArcRing total={10} completed={3} size="sm" />,
    );
    const svg = container.querySelector("svg");
    expect(svg).toHaveAttribute("width", "16");
  });
});
