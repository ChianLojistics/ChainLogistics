import { render } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { axe } from "vitest-axe";
import type { AxeResults } from "vitest-axe";
import { describe, it, expect } from "vitest";

import { Input } from "@/components/ui/input";
import { FormStepIndicator } from "@/components/forms/FormStepIndicator";
import EventTypeSelector from "@/components/forms/EventTypeSelector";

function expectNoViolations(results: AxeResults) {
  const violations = results.violations;
  if (violations.length > 0) {
    const messages = violations.map(
      (v) => `${v.id}: ${v.description} (${v.nodes.length} node(s))`
    );
    throw new Error(
      `Expected no accessibility violations but found ${violations.length}:\n${messages.join("\n")}`
    );
  }
}

describe("Accessibility", () => {
  describe("Input component", () => {
    it("associates label with input via htmlFor and id", () => {
      const { getByLabelText } = render(
        <Input label="Email" placeholder="Enter email" />
      );
      const input = getByLabelText("Email");
      expect(input).toBeInTheDocument();
      expect(input.tagName).toBe("INPUT");
    });

    it("links error message via aria-describedby", () => {
      const { getByLabelText, getByRole } = render(
        <Input label="Email" error="Required field" />
      );
      const input = getByLabelText("Email");
      expect(input).toHaveAttribute("aria-invalid", "true");

      const alert = getByRole("alert");
      expect(alert).toHaveTextContent("Required field");

      const describedBy = input.getAttribute("aria-describedby");
      expect(describedBy).toBeTruthy();
      expect(document.getElementById(describedBy!)).toBe(alert);
    });

    it("has no axe violations", async () => {
      const { container } = render(
        <Input label="Username" placeholder="Enter username" />
      );
      const results = await axe(container);
      expectNoViolations(results);
    });

    it("has no axe violations with error state", async () => {
      const { container } = render(
        <Input label="Username" error="This field is required" />
      );
      const results = await axe(container);
      expectNoViolations(results);
    });
  });

  describe("EventTypeSelector component", () => {
    it("renders as a radiogroup with radio options", () => {
      const { getByRole, getAllByRole } = render(
        <EventTypeSelector value="" onChange={() => {}} />
      );
      expect(getByRole("radiogroup")).toBeInTheDocument();
      const radios = getAllByRole("radio");
      expect(radios.length).toBeGreaterThan(0);
    });

    it("marks selected option with aria-checked", () => {
      const { getAllByRole } = render(
        <EventTypeSelector value="HARVEST" onChange={() => {}} />
      );
      const radios = getAllByRole("radio");
      const selected = radios.find(
        (r) => r.getAttribute("aria-checked") === "true"
      );
      expect(selected).toBeTruthy();
    });

    it("supports keyboard activation with Enter and Space", async () => {
      const user = userEvent.setup();
      let selectedValue = "";
      const { getAllByRole } = render(
        <EventTypeSelector
          value=""
          onChange={(val) => { selectedValue = val; }}
        />
      );
      const radios = getAllByRole("radio");
      radios[0].focus();
      await user.keyboard("{Enter}");
      expect(selectedValue).toBe("HARVEST");

      selectedValue = "";
      radios[1].focus();
      await user.keyboard(" ");
      expect(selectedValue).toBe("PROCESS");
    });

    it("has no axe violations", async () => {
      const { container } = render(
        <EventTypeSelector value="HARVEST" onChange={() => {}} />
      );
      const results = await axe(container);
      expectNoViolations(results);
    });
  });

  describe("FormStepIndicator component", () => {
    const steps = [
      { id: 1, name: "Basic Info" },
      { id: 2, name: "Details" },
      { id: 3, name: "Review" },
    ];

    it("marks current step with aria-current", () => {
      const { getAllByRole } = render(
        <FormStepIndicator steps={steps} currentStep={2} />
      );
      const items = getAllByRole("listitem");
      const currentItem = items.find(
        (item) => item.getAttribute("aria-current") === "step"
      );
      expect(currentItem).toBeTruthy();
    });

    it("has no axe violations", async () => {
      const { container } = render(
        <FormStepIndicator steps={steps} currentStep={1} />
      );
      const results = await axe(container);
      expectNoViolations(results);
    });
  });
});
