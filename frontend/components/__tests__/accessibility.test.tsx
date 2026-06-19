import { render } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { axe, AxeCore } from "vitest-axe";
import { describe, it, expect } from "vitest";

import { Input } from "@/components/ui/input";
import { FormStepIndicator } from "@/components/forms/FormStepIndicator";
import EventTypeSelector from "@/components/forms/EventTypeSelector";
import { Button } from "@/components/ui/button";
import { Navigation } from "@/components/layouts/Navigation";
import { Footer } from "@/components/layouts/Footer";
import { SkipToContentLink } from "@/components/SkipToContentLink";
import { Features } from "@/components/layouts/Features";
import { ProblemStats } from "@/components/layouts/ProblemStats";
import { HowItWorks } from "@/components/layouts/HowItWorks";
import { UseCases } from "@/components/layouts/UseCases";
import { TrustBlockchain } from "@/components/layouts/TrustBlockchain";
import { Hero } from "@/components/layouts/Hero";
import { CTA } from "@/components/layouts/CTA";

function expectNoViolations(results: AxeCore.AxeResults) {
  const violations = results.violations;
  if (violations.length > 0) {
    const messages = violations.map(
      (v: AxeCore.Result) => `${v.id}: ${v.description} (${v.nodes.length} node(s))`
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

  describe("Button component", () => {
    it("renders with accessible role", () => {
      const { getByRole } = render(<Button>Submit</Button>);
      expect(getByRole("button", { name: "Submit" })).toBeInTheDocument();
    });

    it("handles disabled state with aria-disabled", () => {
      const { getByRole } = render(<Button disabled>Disabled</Button>);
      const button = getByRole("button", { name: "Disabled" });
      expect(button).toBeDisabled();
    });

    it("has no axe violations", async () => {
      const { container } = render(<Button>Accessible Button</Button>);
      const results = await axe(container);
      expectNoViolations(results);
    });
  });

  describe("Navigation component", () => {
    it("has aria-label on nav element", () => {
      const { getByRole } = render(<Navigation />);
      const nav = getByRole("navigation");
      expect(nav).toHaveAttribute("aria-label", "Main navigation");
    });

    it("has accessible hamburger button on mobile", () => {
      const { getByLabelText } = render(<Navigation />);
      const menuButton = getByLabelText("Open menu");
      expect(menuButton).toHaveAttribute("aria-expanded", "false");
      expect(menuButton).toHaveAttribute("aria-label", "Open menu");
    });

    it("toggles mobile menu on hamburger click", async () => {
      const user = userEvent.setup();
      const { getByLabelText } = render(<Navigation />);

      // Initially closed
      const openButton = getByLabelText("Open menu");
      expect(openButton).toHaveAttribute("aria-expanded", "false");

      // Click to open
      await user.click(openButton);
      const closeButton = getByLabelText("Close menu");
      expect(closeButton).toHaveAttribute("aria-expanded", "true");

      // Mobile nav links are now visible — look for Features link in the mobile panel
      const mobileLinks = closeButton.closest("nav")?.querySelectorAll("a");
      const featuresLink = Array.from(mobileLinks || []).find(
        (el) => el.textContent?.trim() === "Features" && el.closest(".space-y-1")
      );
      expect(featuresLink).toBeTruthy();

      // Click to close
      await user.click(closeButton);
      expect(getByLabelText("Open menu")).toHaveAttribute("aria-expanded", "false");
    });

    it("has no axe violations", async () => {
      const { container } = render(<Navigation />);
      const results = await axe(container);
      expectNoViolations(results);
    });
  });

  describe("Footer component", () => {
    it("has aria-labels on social media links", () => {
      const { getByLabelText } = render(<Footer />);
      expect(getByLabelText("GitHub")).toBeInTheDocument();
      expect(getByLabelText("Twitter")).toBeInTheDocument();
      expect(getByLabelText("Discord")).toBeInTheDocument();
    });

    it("has no axe violations", async () => {
      const { container } = render(<Footer />);
      const results = await axe(container);
      expectNoViolations(results);
    });
  });

  describe("SkipToContentLink component", () => {
    it("has sr-only class for visibility", () => {
      const { getByText } = render(<SkipToContentLink />);
      const link = getByText(/skip to main/i);
      expect(link).toHaveAttribute("href", "#main-content");
      expect(link.className).toContain("sr-only");
    });

    it("has no axe violations", async () => {
      const { container } = render(<SkipToContentLink />);
      const results = await axe(container);
      expectNoViolations(results);
    });
  });

  describe("Layout components — axe audits", () => {
    it("Hero has no axe violations", async () => {
      const { container } = render(<Hero />);
      const results = await axe(container);
      expectNoViolations(results);
    });

    it("Features has no axe violations", async () => {
      const { container } = render(<Features />);
      const results = await axe(container);
      expectNoViolations(results);
    });

    it("ProblemStats has no axe violations", async () => {
      const { container } = render(<ProblemStats />);
      const results = await axe(container);
      expectNoViolations(results);
    });

    it("HowItWorks has no axe violations", async () => {
      const { container } = render(<HowItWorks />);
      const results = await axe(container);
      expectNoViolations(results);
    });

    it("UseCases has no axe violations", async () => {
      const { container } = render(<UseCases />);
      const results = await axe(container);
      expectNoViolations(results);
    });

    it("TrustBlockchain has no axe violations", async () => {
      const { container } = render(<TrustBlockchain />);
      const results = await axe(container);
      expectNoViolations(results);
    });

    it("CTA has no axe violations", async () => {
      const { container } = render(<CTA />);
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
