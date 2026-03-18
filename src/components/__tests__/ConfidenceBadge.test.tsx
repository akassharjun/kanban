import { render, screen } from "@testing-library/react";
import { ConfidenceBadge } from "../ConfidenceBadge";

describe("ConfidenceBadge", () => {
  it("returns null when score is null", () => {
    const { container } = render(<ConfidenceBadge score={null} />);
    expect(container.firstChild).toBeNull();
  });

  it("shows green badge for high confidence", () => {
    render(<ConfidenceBadge score={0.92} />);
    expect(screen.getByText(/0\.92/)).toBeInTheDocument();
    expect(screen.getByText(/✓/)).toBeInTheDocument();
  });

  it("shows yellow badge for medium confidence", () => {
    render(<ConfidenceBadge score={0.71} />);
    expect(screen.getByText(/0\.71/)).toBeInTheDocument();
    expect(screen.getByText(/⟳/)).toBeInTheDocument();
  });

  it("shows red badge for low confidence", () => {
    render(<ConfidenceBadge score={0.38} />);
    expect(screen.getByText(/0\.38/)).toBeInTheDocument();
    expect(screen.getByText(/✗/)).toBeInTheDocument();
  });
});
